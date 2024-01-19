use super::{Manager, Window, WindowHandle, Workspace};
use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::{Xyhw, Handle};

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    pub fn window_move_handler(
        &mut self,
        handle: &WindowHandle<H>,
        offset_x: i32,
        offset_y: i32,
    ) -> bool {
        let disable_snap = &self.config.disable_window_snap();
        match self.state.windows.iter_mut().find(|w| w.handle == *handle) {
            Some(w) => {
                process_window(w, offset_x, offset_y);
                if !disable_snap && snap_to_workspace(w, &self.state.workspaces) {
                    self.state.sort_windows();
                }
                true
            }
            None => false,
        }
    }
}

fn process_window<H: Handle>(window: &mut Window<H>, offset_x: i32, offset_y: i32) {
    let mut offset = window.get_floating_offsets().unwrap_or_default();
    let start = window.start_loc.unwrap_or_default();
    offset.set_x(start.x() + offset_x);
    offset.set_y(start.y() + offset_y);
    window.set_floating_offsets(Some(offset));
}

// Update the window for the workspace it is currently on.
fn snap_to_workspace<H: Handle>(window: &mut Window<H>, workspaces: &[Workspace]) -> bool {
    // Check that the workspace contains the window.
    let loc = window.calculated_xyhw();
    let (x, y) = loc.center();

    if let Some(workspace) = workspaces.iter().find(|ws| ws.contains_point(x, y)) {
        return should_snap(window, workspace, loc);
    }
    false
}

// To be snapable, the window must be inside the workspace AND the a side must be close to
// the workspaces edge.
fn should_snap<H: Handle>(window: &mut Window<H>, workspace: &Workspace, loc: Xyhw) -> bool {
    if window.must_float() {
        return false;
    }
    // Get window sides.
    let win_left = loc.x();
    let win_right = win_left + window.width();
    let win_top = loc.y();
    let win_bottom = win_top + window.height();
    // Check for close edge.
    let dist = 10;
    let ws_left = workspace.x();
    let ws_right = workspace.x() + workspace.width();
    let ws_top = workspace.y();
    let ws_bottom = workspace.y() + workspace.height();
    if [
        win_top - ws_top,
        win_bottom - ws_bottom,
        win_left - ws_left,
        win_right - ws_right,
    ]
    .iter()
    .any(|x| x.abs() < dist)
    {
        return window.snap_to_workspace(workspace);
    }
    false
}
