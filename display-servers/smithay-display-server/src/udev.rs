use std::collections::HashMap;

use smithay::{
    backend::{
        allocator::gbm::{GbmAllocator, GbmDevice},
        drm::{compositor::DrmCompositor, DrmDevice, DrmDeviceFd, DrmNode, NodeType},
        renderer::{
            element::texture::TextureBuffer,
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiTexture},
        },
        session::{libseat::LibSeatSession, Session},
        udev,
    },
    output::Output,
    reexports::{
        calloop::RegistrationToken, drm::control::crtc, wayland_server::backend::GlobalId,
    },
};
use smithay_drm_extras::drm_scanner::DrmScanner;

pub type GbmDrmCompositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmDevice<DrmDeviceFd>, (), DrmDeviceFd>;

pub struct UdevData {
    pub session: LibSeatSession,
    pub primary_gpu: DrmNode,
    pub gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer>>,
    pub devices: HashMap<DrmNode, Device>,
}

impl UdevData {
    pub fn seat_name(&self) -> String {
        self.session.seat()
    }
}

pub struct Device {
    pub surfaces: HashMap<crtc::Handle, Surface>,
    pub gbm: GbmDevice<DrmDeviceFd>,
    pub drm: DrmDevice,
    pub drm_scanner: DrmScanner,
    pub render_node: DrmNode,
    pub registration_token: RegistrationToken,
}

pub struct Surface {
    _device_id: DrmNode,
    _render_node: DrmNode,
    global: GlobalId,
    pub compositor: GbmDrmCompositor,
    output: Output,
    pointer_texture: TextureBuffer<MultiTexture>,
}

pub fn init_udev(session: LibSeatSession) -> UdevData {
    let primary_gpu = udev::primary_gpu(&session.seat())
        .unwrap()
        .and_then(|gpu| {
            DrmNode::from_path(gpu)
                .ok()?
                .node_with_type(NodeType::Render)?
                .ok()
        })
        .unwrap_or_else(|| {
            udev::all_gpus(&session.seat())
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
    }
}
