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
        let modifier = utils::xkeysym_lookup::into_mod(&self.mousekey);
        //look through the config and build a command if its defined in the config
        let act = self.build_action(modmask, button, handle, modifier);
        if let Some(act) = act {
            //save off the info about position of the window when we started to move/resize
            self.windows
                .iter_mut()
                .filter(|w| w.handle == handle)
                .for_each(|w| {
                    if w.floating() {
                        let offset = w.get_floating_offsets().unwrap_or_default();
                        w.start_loc = Some(offset);
                    } else {
                        let container = w.container_size.unwrap_or_default();
                        let normal = w.normal;
                        let floating = normal - container;
                        w.set_floating_offsets(Some(floating));
                        w.start_loc = Some(floating);
                        w.set_floating(true);
                    }
                });
            self.move_to_top(&handle);
            self.actions.push_back(act);
            return false;
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
                self.mode = Mode::MovingWindow(window);
                Some(DisplayAction::StartMovingWindow(window))
            }
            xlib::Button3 if is_mouse_key => {
                let _ = self
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_resize())?;
                self.mode = Mode::ResizingWindow(window);
                Some(DisplayAction::StartResizingWindow(window))
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
