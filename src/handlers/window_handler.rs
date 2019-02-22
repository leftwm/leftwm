use super::*;

pub struct WindowHandler {}

impl WindowHandler {
    pub fn new() -> WindowHandler {
        WindowHandler {}
    }

    /*
     * process a collection of events, and apply them changes to a manager
     * returns true if changes need to be rendered
     */
    pub fn created(&self, manager: &mut Manager, a_window: Window) -> bool {
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
        manager.windows.push(window);
        true
    }
}
