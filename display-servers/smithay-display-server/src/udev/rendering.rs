mod render_elements;

use std::{borrow::BorrowMut, cell::RefMut, sync::Mutex, time::Duration};

use smithay::{
    backend::{
        allocator::Fourcc,
        drm::{compositor::RenderFrameResult, DrmError, DrmNode},
        renderer::{
            element::{
                surface::WaylandSurfaceRenderElement, texture::TextureBuffer, AsRenderElements,
                RenderElementStates,
            },
            gles::GlesTexture,
            glow::GlowRenderer,
            multigpu::{gbm::GbmGlesBackend, MultiFrame, MultiRenderer},
            Renderer,
        },
        SwapBuffersError,
    },
    desktop::{
        layer_map_for_output,
        utils::{
            surface_presentation_feedback_flags_from_states, surface_primary_scanout_output,
            OutputPresentationFeedback,
        },
        LayerMap, LayerSurface,
    },
    input::pointer::{CursorImageAttributes, CursorImageStatus},
    output::Output,
    reexports::{
        calloop::timer::{TimeoutAction, Timer},
        drm::{control::crtc, SystemError},
    },
    utils::{IsAlive, Scale, Transform},
    wayland::{compositor, dmabuf::DmabufFeedback, shell::wlr_layer::Layer},
};
use tracing::warn;

use crate::{
    drawing::{border::BorderRenderer, CLEAR_COLOR},
    managed_window::ManagedWindow,
    state::SmithayState,
    udev::UdevOutputId,
};

use self::render_elements::CustomRenderElements;

type UdevRenderer<'a, 'b> =
    MultiRenderer<'a, 'a, 'b, GbmGlesBackend<GlowRenderer>, GbmGlesBackend<GlowRenderer>>;
type UdevFrame<'a, 'b, 'frame> =
    MultiFrame<'a, 'a, 'b, 'frame, GbmGlesBackend<GlowRenderer>, GbmGlesBackend<GlowRenderer>>;

impl SmithayState {
    pub fn render(&mut self, node: DrmNode, crtc: crtc::Handle) -> Result<bool, SwapBuffersError> {
        let device = self.udev_data.devices.get_mut(&node).unwrap();
        let surface = device.surfaces.get_mut(&crtc).unwrap();
        let mut renderer = self
            .udev_data
            .gpu_manager
            .single_renderer(&device.render_node)
            .unwrap();
        let (output, output_geometry) = self
            .outputs
            .iter()
            .find(|(o, _)| {
                o.user_data()
                    .get::<UdevOutputId>()
                    .map(|id| id.device_id == node && id.crtc == crtc)
                    .unwrap_or(false)
            })
            .unwrap();
        let scale = Scale::from(output.current_scale().fractional_scale());

        let mut elements: Vec<CustomRenderElements<MultiRenderer<_, _>>> = Vec::new();

        let mut cursor_status = self.cursor_status.lock().unwrap();

        if output_geometry.to_f64().contains(self.pointer_location) {
            let frame = self
                .udev_data
                .pointer_image
                .get_image(1, self.clock.now().try_into().unwrap());

            let pointer_images = &mut self.udev_data.pointer_images;
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

            let cursor_hotspot = if let CursorImageStatus::Surface(ref surface) = *cursor_status {
                compositor::with_states(surface, |states| {
                    states
                        .data_map
                        .get::<Mutex<CursorImageAttributes>>()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .hotspot
                })
            } else {
                (0, 0).into()
            };
            let cursor_pos =
                self.pointer_location - output_geometry.loc.to_f64() - cursor_hotspot.to_f64();
            let cursor_pos_scaled = cursor_pos.to_physical(scale).to_i32_round();

            // set cursor
            self.udev_data
                .pointer_element
                .set_texture(pointer_image.clone());

            // draw the cursor as relevant
            // reset the cursor if the surface is no longer alive
            let mut reset = false;
            if let CursorImageStatus::Surface(ref surface) = *cursor_status {
                reset = !surface.alive();
            }
            if reset {
                *cursor_status = CursorImageStatus::Default;
            }

            self.udev_data
                .pointer_element
                .set_status(cursor_status.clone());

            elements.extend(self.udev_data.pointer_element.render_elements(
                &mut renderer,
                cursor_pos_scaled,
                scale,
                1.0,
            ));

            // draw the dnd icon if applicable
            // {
            //     if let Some(wl_surface) = dnd_icon.as_ref() {
            //         if wl_surface.alive() {
            //             custom_elements.extend(
            //                 AsRenderElements::<UdevRenderer<'a, 'b>>::render_elements(
            //                     &SurfaceTree::from_surface(wl_surface),
            //                     renderer,
            //                     cursor_pos_scaled,
            //                     scale,
            //                     1.0,
            //                 ),
            //             );
            //         }
            //     }
            // }
        }
        let layer_map = layer_map_for_output(output);
        let (lower, upper): (Vec<&LayerSurface>, Vec<&LayerSurface>) = layer_map
            .layers()
            .rev()
            .partition(|s| matches!(s.layer(), Layer::Background | Layer::Bottom));

        elements.extend(
            upper
                .into_iter()
                .filter_map(|surface| {
                    layer_map
                        .layer_geometry(surface)
                        .map(|geo| (geo.loc, surface))
                })
                .flat_map(|(loc, surface)| {
                    AsRenderElements::<MultiRenderer<_, _>>::render_elements::<
                        WaylandSurfaceRenderElement<MultiRenderer<_, _>>,
                    >(
                        surface,
                        &mut renderer,
                        loc.to_physical_precise_round(1.0),
                        Scale::from(1.0),
                        1.0,
                    )
                    .into_iter()
                    .map(CustomRenderElements::Surface)
                }),
        );

        let windows = self.window_registry.windows_in_rect(output_geometry);

        elements.extend(windows.iter().flat_map(|w| {
            w.render_elements(
                &mut renderer,
                &self.focused_window,
                &self.config.borders,
                w.data.read().unwrap().geometry.unwrap().loc.to_physical(1),
                Scale::from(1.0),
                1.0,
            )
        }));

        elements.extend(
            lower
                .into_iter()
                .filter_map(|surface| {
                    layer_map
                        .layer_geometry(surface)
                        .map(|geo| (geo.loc, surface))
                })
                .flat_map(|(loc, surface)| {
                    AsRenderElements::<MultiRenderer<_, _>>::render_elements::<
                        WaylandSurfaceRenderElement<MultiRenderer<_, _>>,
                    >(
                        surface,
                        &mut renderer,
                        loc.to_physical_precise_round(1.0),
                        Scale::from(1.0),
                        1.0,
                    )
                    .into_iter()
                    .map(CustomRenderElements::Surface)
                }),
        );

        let mut frame_result: Result<RenderFrameResult<_, _, _>, SwapBuffersError> = surface
            .compositor
            .render_frame::<_, _, GlesTexture>(&mut renderer, &elements, CLEAR_COLOR)
            .map_err(|err| match err {
                smithay::backend::drm::compositor::RenderFrameError::PrepareFrame(err) => {
                    err.into()
                }
                smithay::backend::drm::compositor::RenderFrameError::RenderFrame(
                    smithay::backend::renderer::damage::Error::Rendering(err),
                ) => err.into(),
                _ => unreachable!(),
            });

        //TODO: screencopy

        if let Ok(result) = &frame_result {
            if result.damage.is_some() {
                let output_presentation_feedback =
                    take_presentation_feedback(output, &windows, layer_map, &result.states);
                let queue_result = surface
                    .compositor
                    .queue_frame(Some(output_presentation_feedback))
                    .map_err(Into::<SwapBuffersError>::into);
                if let Err(queue_result) = queue_result {
                    frame_result = Err(queue_result);
                }
            }
        }

        let reschedule = match &frame_result {
            Ok(has_rendered) => !has_rendered.damage.is_some(),
            Err(err) => {
                warn!("Error rendering frame: {:?}", err);
                match err {
                    SwapBuffersError::AlreadySwapped => false,
                    SwapBuffersError::TemporaryFailure(err) => !matches!(
                        err.downcast_ref::<DrmError>(),
                        Some(&DrmError::DeviceInactive)
                            | Some(&DrmError::Access {
                                source: SystemError::PermissionDenied,
                                ..
                            })
                    ),
                    SwapBuffersError::ContextLost(err) => {
                        panic!("Rendering loop lost: {}", err)
                    }
                }
            }
        };

        if reschedule {
            let output_refresh = match output.current_mode() {
                Some(mode) => mode.refresh,
                None => return frame_result.map(|frame_result| frame_result.damage.is_some()),
            };

            let duration = Duration::from_millis((1_000_000f32 / output_refresh as f32) as u64);
            let timer = Timer::from_duration(duration);
            self.loop_handle
                .insert_source(timer, move |_, _, data| {
                    data.state.render(node, crtc).unwrap();
                    TimeoutAction::Drop
                })
                .unwrap();
        }

        for window in windows {
            window.send_frame(
                output,
                self.start_time.elapsed(),
                Some(Duration::ZERO),
                |_, _| Some(output.clone()),
            );
        }

        BorderRenderer::cleanup(&mut renderer.as_mut().borrow_mut());

        frame_result.map(|frame_result| frame_result.damage.is_some())
    }
}

pub trait AsGlowRenderer
where
    Self: Renderer,
{
    fn glow_renderer(&self) -> &GlowRenderer;
    fn glow_renderer_mut(&mut self) -> &mut GlowRenderer;
}

impl AsGlowRenderer for GlowRenderer {
    fn glow_renderer(&self) -> &GlowRenderer {
        self
    }

    fn glow_renderer_mut(&mut self) -> &mut GlowRenderer {
        self
    }
}

impl<'a, 'b> AsGlowRenderer for UdevRenderer<'a, 'b> {
    fn glow_renderer(&self) -> &GlowRenderer {
        self.as_ref()
    }

    fn glow_renderer_mut(&mut self) -> &mut GlowRenderer {
        self.as_mut()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SurfaceDmabufFeedback<'a> {
    pub render_feedback: &'a DmabufFeedback,
    pub scanout_feedback: &'a DmabufFeedback,
}

pub fn take_presentation_feedback(
    output: &Output,
    windows: &Vec<ManagedWindow>,
    layer_map: RefMut<LayerMap>,
    render_element_states: &RenderElementStates,
) -> OutputPresentationFeedback {
    let mut output_presentation_feedback = OutputPresentationFeedback::new(output);

    for window in windows {
        window.take_presentation_feedback(
            &mut output_presentation_feedback,
            surface_primary_scanout_output,
            |surface, _| {
                surface_presentation_feedback_flags_from_states(surface, render_element_states)
            },
        );
    }
    for layer_surface in layer_map.layers() {
        layer_surface.take_presentation_feedback(
            &mut output_presentation_feedback,
            surface_primary_scanout_output,
            |surface, _| {
                surface_presentation_feedback_flags_from_states(surface, render_element_states)
            },
        );
    }

    output_presentation_feedback
}
