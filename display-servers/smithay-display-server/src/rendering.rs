use smithay::{backend::drm::DrmNode, reexports::drm::control::crtc};
use tracing::error;

use crate::state::SmithayState;

impl SmithayState {
    fn render(&mut self, node: DrmNode, crtc: Option<crtc::Handle>) {
        let device_backend = match self.udev_data.devices.get_mut(&node) {
            Some(backend) => backend,
            None => {
                error!("Trying to render on non-existent backend {}", node);
                return;
            }
        };

        if let Some(crtc) = crtc {
            self.render_surface(node, crtc);
        } else {
            let crtcs: Vec<_> = device_backend.surfaces.keys().copied().collect();
            for crtc in crtcs {
                self.render_surface(node, crtc);
            }
        };
    }

    fn render_surface(&mut self, node: DrmNode, crtc: crtc::Handle) {
        let device = if let Some(device) = self.udev_data.devices.get_mut(&node) {
            device
        } else {
            return;
        };

        let surface = if let Some(surface) = device.surfaces.get_mut(&crtc) {
            surface
        } else {
            return;
        };

        // TODO get scale from the rendersurface when supporting HiDPI
        let frame = self
            .udev_data
            .pointer_image
            .get_image(1 /*scale*/, self.clock.now().try_into().unwrap());

        let render_node = surface.render_node;
        let primary_gpu = self.udev_data.primary_gpu;
        let mut renderer = if primary_gpu == render_node {
            self.udev_data.gpu_manager.single_renderer(&render_node)
        } else {
            let format = surface.compositor.format();
            self.udev_data.gpu_manager.renderer(
                &primary_gpu,
                &render_node,
                self.udev_data
                    .allocator
                    .as_mut()
                    // TODO: We could build some kind of `GLAllocator` using Renderbuffers in theory for this case.
                    //  That would work for memcpy's of offscreen contents.
                    .expect("We need an allocator for multigpu systems")
                    .as_mut(),
                format,
            )
        }
        .unwrap();

        let pointer_images = &mut self.backend_data.pointer_images;
        let pointer_image = pointer_images
            .iter()
            .find_map(|(image, texture)| {
                if image == &frame {
                    Some(texture.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                let texture = TextureBuffer::from_memory(
                    &mut renderer,
                    &frame.pixels_rgba,
                    Fourcc::Abgr8888,
                    (frame.width as i32, frame.height as i32),
                    false,
                    1,
                    Transform::Normal,
                    None,
                )
                .expect("Failed to import cursor bitmap");
                pointer_images.push((frame, texture.clone()));
                texture
            });

        let output = if let Some(output) = self.space.outputs().find(|o| {
            o.user_data().get::<UdevOutputId>()
                == Some(&UdevOutputId {
                    device_id: surface.device_id,
                    crtc,
                })
        }) {
            output.clone()
        } else {
            // somehow we got called with an invalid output
            return;
        };

        let result = render_surface(
            surface,
            &mut renderer,
            &self.space,
            &output,
            self.seat.input_method(),
            self.pointer_location,
            &pointer_image,
            &mut self.backend_data.pointer_element,
            &self.dnd_icon,
            &mut self.cursor_status.lock().unwrap(),
            &self.clock,
            self.show_window_preview,
        );
        let reschedule = match &result {
            Ok(has_rendered) => !has_rendered,
            Err(err) => {
                warn!("Error during rendering: {:?}", err);
                match err {
                    SwapBuffersError::AlreadySwapped => false,
                    SwapBuffersError::TemporaryFailure(err) => !matches!(
                        err.downcast_ref::<DrmError>(),
                        Some(&DrmError::DeviceInactive)
                            | Some(&DrmError::Access {
                                source: drm::SystemError::PermissionDenied,
                                ..
                            })
                    ),
                    SwapBuffersError::ContextLost(err) => panic!("Rendering loop lost: {}", err),
                }
            }
        };

        if reschedule {
            let output_refresh = match output.current_mode() {
                Some(mode) => mode.refresh,
                None => return,
            };
            // If reschedule is true we either hit a temporary failure or more likely rendering
            // did not cause any damage on the output. In this case we just re-schedule a repaint
            // after approx. one frame to re-test for damage.
            let reschedule_duration =
                Duration::from_millis((1_000_000f32 / output_refresh as f32) as u64);
            trace!(
                "reschedule repaint timer with delay {:?} on {:?}",
                reschedule_duration,
                crtc,
            );
            let timer = Timer::from_duration(reschedule_duration);
            self.handle
                .insert_source(timer, move |_, _, data| {
                    data.state.render(node, Some(crtc));
                    TimeoutAction::Drop
                })
                .expect("failed to schedule frame timer");
        }
    }
}
