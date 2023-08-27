use crate::display_action::DisplayAction;
use crate::models::Mode;
use crate::models::WindowHandle;
use crate::state::State;
use crate::utils;
use crate::utils::modmask_lookup::Button;
use crate::utils::modmask_lookup::ModMask;
use x11_dl::xlib;

impl State {
    pub fn mouse_combo_handler(
        &mut self,
        modmask: ModMask,
        button: Button,
        handle: WindowHandle,
        x: i32,
        y: i32,
    ) -> bool {
        if let Some(window) = self.windows.iter().find(|w| w.handle == handle) {
            if !self.disable_tile_drag || window.floating() {
                let modifier = utils::modmask_lookup::into_modmask(&self.mousekey);
                let bottom_right = (window.x() + window.width(), window.y() + window.height());
                // Build the display to say whether we are ready to move/resize.
                let act = self.build_action(modmask, button, handle, modifier);
                if let Some(act) = act {
                    if self.reposition_cursor_on_resize {
                        if let DisplayAction::ReadyToResizeWindow(_) = act {
                            let move_act = DisplayAction::MoveMouseOverPoint(bottom_right);
                            self.actions.push_back(move_act);
                        }
                    }
                    self.actions.push_back(act);
                    return false;
                }
            }
        } else if self.focus_manager.behaviour.is_clickto() {
            if let xlib::Button1 | xlib::Button3 = button {
                if self.screens.iter().any(|s| s.root == handle) {
                    self.focus_workspace_with_point(x, y);
                    return false;
                }
            }
        }
        true
    }

    fn build_action(
        &mut self,
        mod_mask: ModMask,
        button: Button,
        window: WindowHandle,
        modifier: ModMask,
    ) -> Option<DisplayAction> {
        let is_mouse_key = mod_mask == modifier || mod_mask == (modifier | xlib::ShiftMask);
        match button {
            xlib::Button1 if is_mouse_key => {
                _ = self
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_move())?;
                self.mode = Mode::ReadyToMove(window);
                Some(DisplayAction::ReadyToMoveWindow(window))
            }
            xlib::Button3 if is_mouse_key => {
                _ = self
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_resize())?;
                self.mode = Mode::ReadyToResize(window);
                Some(DisplayAction::ReadyToResizeWindow(window))
            }
            xlib::Button1 | xlib::Button3 if self.focus_manager.behaviour.is_clickto() => {
                self.focus_window(&window);
                Some(DisplayAction::ReplayClick(window, button))
            }
            _ => None,
        }
    }
}
