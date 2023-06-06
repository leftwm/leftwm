pub mod devices;
mod rendering;

use std::collections::{HashMap, HashSet};

use smithay::{
    backend::{
        allocator::{
            dmabuf::{AnyError, Dmabuf, DmabufAllocator},
            gbm::{GbmAllocator, GbmDevice},
            vulkan::{ImageUsageFlags, VulkanAllocator},
            Allocator,
        },
        drm::{compositor::DrmCompositor, DrmDevice, DrmDeviceFd, DrmNode, NodeType},
        renderer::{
            element::texture::TextureBuffer,
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiTexture},
            ImportDma, ImportEgl, ImportMemWl,
        },
        session::{libseat::LibSeatSession, Session},
        udev::{self, UdevBackend},
        vulkan::{version::Version, Instance, PhysicalDevice},
    },
    desktop::utils::OutputPresentationFeedback,
    output::Output,
    reexports::{
        ash::vk::ExtPhysicalDeviceDrmFn,
        calloop::RegistrationToken,
        drm::control::crtc,
        wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1,
        wayland_server::{backend::GlobalId, protocol::wl_surface, Display},
    },
    wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
};
use smithay_drm_extras::drm_scanner::DrmScanner;
use tracing::{error, info, warn};
use xcursor::parser::Image;

use crate::{drawing::PointerElement, pointer, state::SmithayState};

use self::devices::DeviceAddError;

pub type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmDevice<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd,
>;

pub struct UdevData {
    pub session: LibSeatSession,
    pub primary_gpu: DrmNode,
    pub gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer>>,
    pub devices: HashMap<DrmNode, Device>,
    pub allocator: Option<Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError>>>,
    pub dmabuf_state: Option<(DmabufState, DmabufGlobal)>,

    pub pointer_image: crate::pointer::Cursor,
    pub pointer_images: Vec<(Image, TextureBuffer<MultiTexture>)>,
    pub pointer_element: PointerElement<MultiTexture>,
}

impl UdevData {
    pub fn seat_name(&self) -> String {
        self.session.seat()
    }

    pub fn early_import(&mut self, surface: &wl_surface::WlSurface) {
        if let Err(err) =
            self.gpu_manager
                .early_import(Some(self.primary_gpu), self.primary_gpu, surface)
        {
            warn!("Early buffer import failed: {}", err);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UdevOutputId {
    pub device_id: DrmNode,
    pub crtc: crtc::Handle,
}

pub struct Device {
    pub surfaces: HashMap<crtc::Handle, Surface>,
    pub gbm: GbmDevice<DrmDeviceFd>,
    pub drm: DrmDevice,
    pub drm_scanner: DrmScanner,

    pub render_node: DrmNode,
    pub registration_token: RegistrationToken,
}

pub struct DrmSurfaceDmabufFeedback {
    pub render_feedback: DmabufFeedback,
    pub scanout_feedback: DmabufFeedback,
}

pub struct Surface {
    pub device_id: DrmNode,
    pub render_node: DrmNode,
    pub dmabuf_feedback: Option<DrmSurfaceDmabufFeedback>,
    global: GlobalId,
    // NOTE: Currently only doing hardware compositing, do we wanna have the oprion for software.
    pub compositor: GbmDrmCompositor,
    output: Output,
}

pub fn init_udev_stage_1(session: LibSeatSession) -> UdevData {
    let primary_gpu = udev::primary_gpu(session.seat())
        .unwrap()
        .and_then(|gpu| {
            DrmNode::from_path(gpu)
                .ok()?
                .node_with_type(NodeType::Render)?
                .ok()
        })
        .unwrap_or_else(|| {
            udev::all_gpus(session.seat())
                .unwrap()
                .into_iter()
                .find_map(|gpu| {
                    DrmNode::from_path(gpu)
                        .ok()?
                        .node_with_type(NodeType::Render)?
                        .ok()
                })
                .unwrap()
        });

    let gpu_manager = GpuManager::new(Default::default()).unwrap();

    UdevData {
        session,
        primary_gpu,
        gpu_manager,
        devices: HashMap::new(),
        allocator: None,
        dmabuf_state: None,

        pointer_image: pointer::Cursor::load(),
        pointer_images: Vec::new(),
        pointer_element: Default::default(),
    }
}
impl SmithayState {
    pub fn init_udev_stage_2(
        &mut self,
        udev_backend: UdevBackend,
        display: &Display<SmithayState>,
    ) {
        for (device_id, path) in udev_backend.device_list() {
            if let Err(err) = DrmNode::from_dev_id(device_id)
                .map_err(DeviceAddError::DrmNode)
                .and_then(|node| self.device_added(node, path))
            {
                error!("Skipping device {device_id}: {:?}", err);
            }
        }

        self.shm_state.update_formats(
            self.udev_data
                .gpu_manager
                .single_renderer(&self.udev_data.primary_gpu)
                .unwrap()
                .shm_formats(),
        );

        if let Ok(instance) = Instance::new(Version::VERSION_1_2, None) {
            if let Some(physical_device) =
                PhysicalDevice::enumerate(&instance)
                    .ok()
                    .and_then(|devices| {
                        devices
                            .filter(|phd| phd.has_device_extension(ExtPhysicalDeviceDrmFn::name()))
                            .find(|phd| {
                                phd.primary_node().unwrap() == Some(self.udev_data.primary_gpu)
                                    || phd.render_node().unwrap()
                                        == Some(self.udev_data.primary_gpu)
                            })
                    })
            {
                match VulkanAllocator::new(
                    &physical_device,
                    ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED,
                ) {
                    Ok(allocator) => {
                        self.udev_data.allocator = Some(Box::new(DmabufAllocator(allocator))
                            as Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError>>);
                    }
                    Err(err) => {
                        warn!("Failed to create vulkan allocator: {}", err);
                    }
                }
            }
        }

        let mut renderer = self
            .udev_data
            .gpu_manager
            .single_renderer(&self.udev_data.primary_gpu)
            .unwrap();

        match renderer.bind_wl_display(&display.handle()) {
            Ok(_) => info!("EGL hardware-acceleration enabled"),
            Err(err) => {
                info!(?err, "Failed to initialize EGL hardware-acceleration")
            }
        }

        let dmabuf_formats = renderer.dmabuf_formats().collect::<Vec<_>>();
        let default_feedback =
            DmabufFeedbackBuilder::new(self.udev_data.primary_gpu.dev_id(), dmabuf_formats)
                .build()
                .unwrap();
        let mut dmabuf_state = DmabufState::new();
        let global = dmabuf_state.create_global_with_default_feedback::<SmithayState>(
            &display.handle(),
            &default_feedback,
        );
        self.udev_data.dmabuf_state = Some((dmabuf_state, global));

        let gpus = &mut self.udev_data.gpu_manager;
        self.udev_data
            .devices
            .values_mut()
            .for_each(|backend_data| {
                // Update the per drm surface dmabuf feedback
                backend_data.surfaces.values_mut().for_each(|surface_data| {
                    surface_data.dmabuf_feedback =
                        surface_data.dmabuf_feedback.take().or_else(|| {
                            get_surface_dmabuf_feedback(
                                self.udev_data.primary_gpu,
                                surface_data.render_node,
                                gpus,
                                &surface_data.compositor,
                            )
                        });
                });
            });
    }
}

pub fn get_surface_dmabuf_feedback(
    primary_gpu: DrmNode,
    render_node: DrmNode,
    gpus: &mut GpuManager<GbmGlesBackend<GlesRenderer>>,
    composition: &GbmDrmCompositor,
) -> Option<DrmSurfaceDmabufFeedback> {
    let primary_formats = gpus
        .single_renderer(&primary_gpu)
        .ok()?
        .dmabuf_formats()
        .collect::<HashSet<_>>();

    let render_formats = gpus
        .single_renderer(&render_node)
        .ok()?
        .dmabuf_formats()
        .collect::<HashSet<_>>();

    let all_render_formats = primary_formats
        .iter()
        .chain(render_formats.iter())
        .copied()
        .collect::<HashSet<_>>();

    let surface = composition.surface();
    let planes = surface.planes().unwrap();
    // We limit the scan-out trache to formats we can also render from
    // so that there is always a fallback render path available in case
    // the supplied buffer can not be scanned out directly
    let planes_formats = surface
        .supported_formats(planes.primary.handle)
        .unwrap()
        .into_iter()
        .chain(
            planes
                .overlay
                .iter()
                .flat_map(|p| surface.supported_formats(p.handle).unwrap()),
        )
        .collect::<HashSet<_>>()
        .intersection(&all_render_formats)
        .copied()
        .collect::<Vec<_>>();

    let builder = DmabufFeedbackBuilder::new(primary_gpu.dev_id(), primary_formats);
    let render_feedback = builder
        .clone()
        .add_preference_tranche(render_node.dev_id(), None, render_formats.clone())
        .build()
        .unwrap();

    let scanout_feedback = builder
        .add_preference_tranche(
            surface.device_fd().dev_id().unwrap(),
            Some(zwp_linux_dmabuf_feedback_v1::TrancheFlags::Scanout),
            planes_formats,
        )
        .add_preference_tranche(render_node.dev_id(), None, render_formats)
        .build()
        .unwrap();

    Some(DrmSurfaceDmabufFeedback {
        render_feedback,
        scanout_feedback,
    })
}
