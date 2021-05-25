use crate::handlers::focus_handler;
use crate::models::Manager;
use crate::models::Mode;
use crate::models::WindowHandle;
use crate::utils::xkeysym_lookup::Button;
use crate::utils::xkeysym_lookup::ModMask;
use crate::{display_action::DisplayAction, models::FocusBehaviour};
use x11_dl::xlib;

pub fn process(
    manager: &mut Manager,
    modmask: ModMask,
    button: Button,
    handle: WindowHandle,
) -> bool {
    //look through the config and build a command if its defined in the config
    let act = build_action(manager, modmask, button, handle);
    if let Some(act) = act {
        //save off the info about position of the window when we started to move/resize
        manager
            .windows
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
        manager.move_to_top(&handle);
        manager.actions.push_back(act);
        return false;
    }

    true
}

fn build_action(
    manager: &mut Manager,
    mut mod_mask: ModMask,
    button: Button,
    window: WindowHandle,
) -> Option<DisplayAction> {
    match button {
        xlib::Button1 => {
            mod_mask &= !(xlib::Mod2Mask | xlib::LockMask);
            if mod_mask == xlib::ControlMask || mod_mask == (xlib::ControlMask | xlib::ShiftMask) {
                let _ = manager
                    .windows
                    .iter()
                    .find(|w| w.handle == window && w.can_move())?;
                manager.mode = Mode::MovingWindow(window);
                return Some(DisplayAction::StartMovingWindow(window));
            }
            if manager.focus_manager.behaviour == FocusBehaviour::ClickTo {
                focus_handler::focus_window(manager, &window);
            }
            None
        }
        xlib::Button3 => {
            let _ = manager
                .windows
                .iter()
                .find(|w| w.handle == window && w.can_resize())?;
            manager.mode = Mode::ResizingWindow(window);
            Some(DisplayAction::StartResizingWindow(window))
        }
        _ => None,
    }
}
