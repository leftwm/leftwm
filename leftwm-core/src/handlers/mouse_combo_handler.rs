use crate::config::Config;
use crate::models::Mode;
use crate::models::WindowHandle;
use crate::state::State;
use crate::utils::xkeysym_lookup::Button;
use crate::utils::xkeysym_lookup::ModMask;
use crate::{display_action::DisplayAction, models::FocusBehaviour};
use x11_dl::xlib;

impl<C: Config> State<C> {
    pub fn mouse_combo_handler(
        &mut self,
        modmask: ModMask,
        button: Button,
        handle: WindowHandle,
        modifier: ModMask,
    ) -> bool {
        //look through the config and build a command if its defined in the config
        let act = build_action(self, modmask, button, handle, modifier);
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
}

fn build_action<C: Config>(
    state: &mut State<C>,
    mod_mask: ModMask,
    button: Button,
    window: WindowHandle,
    modifier: ModMask,
) -> Option<DisplayAction> {
    match button {
        xlib::Button1 => {
            if mod_mask == modifier || mod_mask == (modifier | xlib::ShiftMask) {
                let _ = state
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_move())?;
                state.mode = Mode::MovingWindow(window);
                return Some(DisplayAction::StartMovingWindow(window));
            }
            if state.focus_manager.behaviour == FocusBehaviour::ClickTo {
                state.focus_window(&window);
            }
            None
        }
        xlib::Button3 => {
            let _ = state
                .windows
                .iter()
                .find(|w| w.handle == window && w.can_resize())?;
            state.mode = Mode::ResizingWindow(window);
            Some(DisplayAction::StartResizingWindow(window))
        }
        _ => None,
    }
}
