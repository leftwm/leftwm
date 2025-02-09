use leftwm_core::{
    models::{WindowHandle, WindowType},
    DisplayAction, DisplayEvent,
};
use smithay::{
    reexports::wayland_server::Display,
    utils::{Logical, Point, Rectangle},
};
use tracing::{info, warn};

use crate::{internal_action::InternalAction, state::SmithayState, SmithayWindowHandle};

impl SmithayState {
    pub fn handle_action(&mut self, action: InternalAction, display: &mut Display<Self>) {
        info!("Received action: {:?}", action);
        match action {
            InternalAction::Flush => display.flush_clients().unwrap(),
            InternalAction::GenerateVerifyFocusEvent => {
                if let Some(handle) = self.focused_window {
                    self.send_event(DisplayEvent::VerifyFocusedAt(WindowHandle(
                        SmithayWindowHandle(handle),
                    )))
                    .unwrap();
                }
            } //NOTE: We should probably send an event too when nothing is focused
            InternalAction::UpdateConfig(config) => self.config = config,
            InternalAction::UpdateWindows(windows) => {
                info!("Received window update: {:#?}", windows);
                for window in windows {
                    let WindowHandle(SmithayWindowHandle(handle)) = window.handle;
                    if window.r#type == WindowType::WlrSurface {
                        warn!("LeftWM is trying to manage a surface, discarding")
                    } else {
                        let Some(managed_window) = self.window_registry.get_mut(handle) else {
                            warn!("LeftWM is trying to manage a nonexistent window, discarding");
                            return;
                        };

                        let border_width = self.config.borders.border_width;
                        // let border_width = 0;
                        let loc = (window.x() + border_width, window.y() + border_width).into();
                        let size = (
                            window.width() - 2 * border_width,
                            window.height() - 2 * border_width,
                        )
                            .into();
                        managed_window.set_geometry(Rectangle { loc, size });

                        let mut managed_window_data = managed_window.data.write().unwrap();

                        managed_window_data.floating = window.floating();
                        managed_window_data.visible = window.visible();

                        managed_window
                            .get_window()
                            .unwrap()
                            .toplevel()
                            .with_pending_state(|state| {
                                state.size = Some((window.width(), window.height()).into());
                            });
                        managed_window
                            .get_window()
                            .unwrap()
                            .toplevel()
                            .send_configure();
                    }
                }
            }
            InternalAction::DisplayAction(DisplayAction::KillWindow(handle)) => {
                let WindowHandle(SmithayWindowHandle(handle)) = handle;
                let window = self.window_registry.get_mut(handle);
                //NOTE: Nothing happens if the window doesnt exist;
                window.map(|w| w.send_close());
            }
            InternalAction::DisplayAction(DisplayAction::AddedWindow(handle, floating, focus)) => {
                let WindowHandle(SmithayWindowHandle(handle)) = handle;
                let window = self.window_registry.get_mut(handle).unwrap();
                let mut window_data = window.data.write().unwrap();
                window_data.floating = floating;
                window_data.managed = true;
                drop(window_data);
                if focus {
                    self.focus_window(handle, true);
                }
            }
            InternalAction::DisplayAction(DisplayAction::MoveMouseOver(handle, force)) => {
                let WindowHandle(SmithayWindowHandle(handle)) = handle;
                if Some(handle) != self.focused_window || force {
                    let window = self.window_registry.get(handle).unwrap();
                    let geometry = window.data.read().unwrap().geometry.unwrap();
                    let center =
                        Point::<i32, Logical>::from((geometry.size.w / 2, geometry.size.h / 2));
                    self.pointer_location = geometry.loc.to_f64() + center.to_f64();
                }
            }
            InternalAction::DisplayAction(DisplayAction::MoveMouseOverPoint(point)) => {
                self.pointer_location = Point::from(point).to_f64();
            }
            InternalAction::DisplayAction(DisplayAction::SetState(_, _, _)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::SetWindowOrder(_)) => {
                //TODO: no `todo!()` here because crash
            }
            InternalAction::DisplayAction(DisplayAction::MoveToTop(_)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::DestroyedWindow(_)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::WindowTakeFocus {
                window,
                previous_window: _,
            }) => {
                let WindowHandle(SmithayWindowHandle(handle)) = window.handle;
                // NOTE: Should we never move the cursor??
                self.focus_window(handle, false);
            }
            InternalAction::DisplayAction(DisplayAction::Unfocus(_, _)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::FocusWindowUnderCursor) => {
                self.focus_window_under();
            }
            InternalAction::DisplayAction(DisplayAction::ReplayClick(_, _)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::ReadyToResizeWindow(_)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::ReadyToMoveWindow(_)) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::SetCurrentTags(_)) => {
                //TODO: no `todo!()` here because crash
            }
            InternalAction::DisplayAction(DisplayAction::SetWindowTag(..)) => {}
            InternalAction::DisplayAction(DisplayAction::NormalMode) => {
                todo!()
            }
            InternalAction::DisplayAction(DisplayAction::ConfigureXlibWindow(_)) => {
                todo!()
            }
        }
    }
}
