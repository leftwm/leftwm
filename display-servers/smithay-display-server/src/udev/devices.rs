use std::{borrow::BorrowMut, collections::HashMap, os::fd::FromRawFd, path::Path};

use leftwm_core::{
    models::{BBox, Screen},
    DisplayEvent,
};
use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{
            compositor::DrmCompositor, CreateDrmNodeError, DrmDevice, DrmDeviceFd, DrmError,
            DrmEvent, DrmNode,
        },
        egl::{self, EGLDevice, EGLDisplay},
        session::{libseat, Session},
    },
    output::{Mode as WlMode, Output, PhysicalProperties, Subpixel},
    reexports::{
        drm::{
            control::{connector, crtc, ModeTypeFlags},
            Device,
        },
        nix::fcntl::OFlag,
    },
    utils::{DeviceFd, Logical, Rectangle, Size},
};

use smithay_drm_extras::{
    drm_scanner::{DrmScanEvent, DrmScanner},
    edid::EdidInfo,
};
use tracing::{error, info, warn};

use crate::{
    drawing::border::BorderRenderer,
    state::{CalloopData, SmithayState},
    udev::{get_surface_dmabuf_feedback, Surface, UdevOutputId},
};

use super::Device as UdevDevice;

pub const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];

pub const SUPPORTED_FORMATS_8BIT: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];

#[derive(Debug)]
pub enum DeviceAddError {
    DeviceOpen(libseat::Error),
    DrmDevice(DrmError),
    GbmDevice(std::io::Error),
    DrmNode(CreateDrmNodeError),
    AddNode(egl::Error),
}

impl SmithayState {
    pub fn device_added(&mut self, node: DrmNode, path: &Path) -> Result<(), DeviceAddError> {
        // Try to open the device
        let fd = self
            .udev_data
            .session
            .open(
                path,
                OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NOCTTY | OFlag::O_NONBLOCK,
            )
            .map_err(DeviceAddError::DeviceOpen)?;

        let fd = DrmDeviceFd::new(unsafe { DeviceFd::from_raw_fd(fd) });

        let (drm, notifier) =
            DrmDevice::new(fd.clone(), true).map_err(DeviceAddError::DrmDevice)?;
        let gbm = GbmDevice::new(fd).map_err(DeviceAddError::GbmDevice)?;

        let registration_token = self
            .loop_handle
            .insert_source(
                notifier,
                move |event, _, data: &mut CalloopData| match event {
                    DrmEvent::VBlank(crtc) => {
                        let device = data.state.udev_data.devices.get_mut(&node).unwrap();
                        let surface = device.surfaces.get_mut(&crtc).unwrap();
                        surface.compositor.frame_submitted().unwrap();
                        data.state.render(node, crtc, None).unwrap();
                    }
                    DrmEvent::Error(error) => {
                        error!("{:?}", error);
                    }
                },
            )
            .unwrap();

        let render_node = EGLDevice::device_for_display(&EGLDisplay::new(gbm.clone()).unwrap())
            .ok()
            .and_then(|x| x.try_get_render_node().ok().flatten())
            .unwrap_or(node);

        self.udev_data
            .gpu_manager
            .as_mut()
            .add_node(render_node, gbm.clone())
            .map_err(DeviceAddError::AddNode)?;

        self.udev_data.devices.insert(
            node,
            UdevDevice {
                surfaces: HashMap::new(),
                gbm,
                drm,
                drm_scanner: DrmScanner::new(),
                render_node,
                registration_token,
            },
        );

        self.device_changed(node);

        Ok(())
    }

    fn device_changed(&mut self, node: DrmNode) {
        let device = if let Some(device) = self.udev_data.devices.get_mut(&node) {
            device
        } else {
            return;
        };

        for event in device.drm_scanner.scan_connectors(&device.drm) {
            match event {
                DrmScanEvent::Connected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    self.connector_connected(node, connector, crtc);
                }
                DrmScanEvent::Disconnected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    self.connector_disconnected(node, connector, crtc);
                }
                _ => {}
            }
        }

        //TODO: fixup window coordinates
    }

    fn connector_connected(
        &mut self,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    ) {
        let device = if let Some(device) = self.udev_data.devices.get_mut(&node) {
            device
        } else {
            return;
        };

        let mut renderer = self
            .udev_data
            .gpu_manager
            .single_renderer(&device.render_node)
            .unwrap();
        let render_formats = renderer
            .as_mut()
            .egl_context()
            .dmabuf_render_formats()
            .clone();

        info!(
            ?crtc,
            "Trying to setup connector {:?}-{}",
            connector.interface(),
            connector.interface_id(),
        );

        let mode_id = connector
            .modes()
            .iter()
            .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or(0);

        let drm_mode = connector.modes()[mode_id];
        let wl_mode = WlMode::from(drm_mode);

        let surface = match device
            .drm
            .create_surface(crtc, drm_mode, &[connector.handle()])
        {
            Ok(surface) => surface,
            Err(err) => {
                warn!("Failed to create drm surface: {}", err);
                return;
            }
        };

        let output_name = format!(
            "{}-{}",
            connector.interface().as_str(),
            connector.interface_id()
        );

        let (make, model) = EdidInfo::for_connector(&device.drm, connector.handle())
            .map(|info| (info.manufacturer, info.model))
            .unwrap_or_else(|| ("Unknown".into(), "Unknown".into()));

        let (phys_w, phys_h) = connector.size().unwrap_or((0, 0));
        let output = Output::new(
            output_name.clone(),
            PhysicalProperties {
                size: (phys_w as i32, phys_h as i32).into(),
                subpixel: Subpixel::Unknown,
                make,
                model,
            },
        );
        let global = output.create_global::<SmithayState>(&self.display_handle);

        let x = self.outputs.iter().fold(0, |acc, (_, r)| acc + r.size.w);
        //TODO: Set pos based on config
        let position = (x, 0).into();

        output.set_preferred(wl_mode);
        output.change_current_state(Some(wl_mode), None, None, Some(position));
        let geomitry = Rectangle {
            loc: position,
            size: output_size(&output).unwrap(),
        };
        self.outputs.push((output.clone(), geomitry));

        output.user_data().insert_if_missing(|| UdevOutputId {
            crtc,
            device_id: node,
        });

        let allocator = GbmAllocator::new(
            device.gbm.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );

        let color_formats: &[Fourcc] = if std::env::var("WAYLAND_DISABLE_10BIT").is_ok() {
            SUPPORTED_FORMATS_8BIT
        } else {
            SUPPORTED_FORMATS
        };

        let compositor = {
            let driver = match device.drm.get_driver() {
                Ok(driver) => driver,
                Err(err) => {
                    warn!("Failed to query drm driver: {}", err);
                    return;
                }
            };

            let mut planes = match surface.planes() {
                Ok(planes) => planes,
                Err(err) => {
                    warn!("Failed to query surface planes: {}", err);
                    return;
                }
            };

            // Using an overlay plane on a nvidia card breaks
            if driver
                .name()
                .to_string_lossy()
                .to_lowercase()
                .contains("nvidia")
                || driver
                    .description()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("nvidia")
            {
                planes.overlay = vec![];
            }

            // Because TODO
            #[allow(clippy::let_and_return)]
            let compositor = match DrmCompositor::new(
                &output,
                surface,
                Some(planes),
                allocator,
                device.gbm.clone(),
                color_formats,
                render_formats,
                device.drm.cursor_size(),
                Some(device.gbm.clone()),
            ) {
                Ok(compositor) => compositor,
                Err(err) => {
                    warn!("Failed to create drm compositor: {}", err);
                    return;
                }
            };
            //TODO
            // compositor.set_debug_flags(self.udev_data.debug_flags);

            compositor
        };

        BorderRenderer::new(renderer.as_mut().borrow_mut());

        let dmabuf_feedback = get_surface_dmabuf_feedback(
            self.udev_data.primary_gpu,
            device.render_node,
            &mut self.udev_data.gpu_manager,
            &compositor,
        );

        let mode = if let Some(mode) = output.current_mode() {
            mode
        } else {
            *output.modes().get(0).unwrap()
        };

        let surface = Surface {
            device_id: node,
            render_node: device.render_node,
            dmabuf_feedback,
            _global: global,
            compositor,
            output,
        };

        device.surfaces.insert(crtc, surface);

        self.send_event(DisplayEvent::ScreenCreate(Screen::new(
            BBox {
                x: position.x,
                y: position.y,
                width: mode.size.w,
                height: mode.size.h,
            },
            output_name,
        )))
        .unwrap();

        // self.schedule_initial_render(node, crtc, self.loop_handle.clone());
        self.render(node, crtc, None).unwrap();
    }

    fn connector_disconnected(
        &mut self,
        node: DrmNode,
        _connector: connector::Info,
        crtc: crtc::Handle,
    ) {
        let device = if let Some(device) = self.udev_data.devices.get_mut(&node) {
            device
        } else {
            return;
        };

        device.surfaces.remove(&crtc);

        let output = self
            .outputs
            .iter()
            .find(|(o, _)| {
                o.user_data()
                    .get::<UdevOutputId>()
                    .map(|id| id.device_id == node && id.crtc == crtc)
                    .unwrap_or(false)
            })
            .cloned();

        if let Some(output) = output {
            self.outputs.retain(|o| o != &output)
        }
    }
}

fn output_size(output: &Output) -> Option<Size<i32, Logical>> {
    let transform = output.current_transform();
    output.current_mode().map(|mode| {
        transform
            .transform_size(mode.size)
            .to_f64()
            .to_logical(output.current_scale().fractional_scale())
            .to_i32_ceil::<i32>()
    })
}
