use super::*;
use crate::display_action::DisplayAction;

/*
 * process a collection of events, and apply them changes to a manager
 * returns true if changes need to be rendered
 */
pub fn created(manager: &mut Manager, a_window: Window) -> bool {
    //don't add the window if the manager already knows about it
    for w in &manager.windows {
        if w.handle == a_window.handle {
            return false;
        }
    }
    let mut window = a_window;
    if let Some(ws) = manager.focused_workspace() {
        window.tags = ws.tags.clone();
    } else {
        window.tags = vec![manager.tags[0].clone()]
    }
    //window.margin = 25;
    manager.windows.push(window.clone());
    focus_handler::focus_window(manager, &window, window.x() + 1, window.y() + 1);

    //let the DS know we are managing this window
    let act = DisplayAction::AddedWindow(window.handle.clone());
    manager.actions.push_back(act);
    true
}

/*
 * process a collection of events, and apply them changes to a manager
 * returns true if changes need to be rendered
 */
pub fn destroyed(manager: &mut Manager, handle: &WindowHandle) -> bool {
    let start_size = manager.windows.len();
    manager.windows = manager
        .windows
        .iter()
        .filter(|w| &w.handle != handle)
        .map(|w| w.clone())
        .collect();
    //if we removed the focused window, focus the last window
    focus_handler::focus_last_window_that_exists(manager);
    start_size != manager.windows.len()
}


pub fn changed(manager: &mut Manager, change: WindowChange) -> bool {
    for w in manager.windows.iter_mut() {
        if w.handle == change.handle {
            change.update(w);
            return true;
        }
    }
    false
}
