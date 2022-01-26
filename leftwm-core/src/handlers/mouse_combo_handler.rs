use crate::models::Mode;
use crate::models::WindowHandle;
use crate::state::State;
use crate::utils;
use crate::utils::xkeysym_lookup::Button;
use crate::utils::xkeysym_lookup::ModMask;
use crate::{display_action::DisplayAction, models::FocusBehaviour};
use x11_dl::xlib;

impl State {
    pub fn mouse_combo_handler(
        &mut self,
        modmask: ModMask,
        button: Button,
        handle: WindowHandle,
    ) -> bool {
        if let Some(window) = self.windows.iter().find(|w| w.handle == handle) {
            if !self.disable_tile_drag || window.floating() {
                let modifier = utils::xkeysym_lookup::into_modmask(&self.mousekey);
                // Build the display to say whether we are ready to move/resize.
                let act = self.build_action(modmask, button, handle, modifier);
                if let Some(act) = act {
                    self.actions.push_back(act);
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
                let _ = self
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_move())?;
                self.mode = Mode::ReadyToMove(window);
                Some(DisplayAction::ReadyToMoveWindow(window))
            }
            xlib::Button3 if is_mouse_key => {
                let _ = self
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_resize())?;
                self.mode = Mode::ReadyToResize(window);
                Some(DisplayAction::ReadyToResizeWindow(window))
            }
            xlib::Button1 | xlib::Button3
                if self.focus_manager.behaviour == FocusBehaviour::ClickTo =>
            {
                self.focus_window(&window);
                None
            }
            _ => None,
        }
    }
}
