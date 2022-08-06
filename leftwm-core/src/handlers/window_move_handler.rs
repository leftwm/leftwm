use super::{Manager, Window, WindowHandle, Workspace};
use crate::config::Config;
use crate::display_servers::DisplayServer;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    pub fn window_move_handler(
        &mut self,
        handle: &WindowHandle,
        offset_x: i32,
        offset_y: i32,
    ) -> bool {
        let margin_multiplier = match self
            .state
            .windows
            .iter()
            .find(|other| other.has_tag(&self.state.focus_manager.tag(0).unwrap_or_default()))
        {
            Some(w) => w.margin_multiplier(),
            None => 1.0,
        };
        let disable_swap = &self.config.disable_window_snap();
        match self.state.windows.iter_mut().find(|w| w.handle == *handle) {
            Some(w) => {
                process_window(w, offset_x, offset_y);
                w.apply_margin_multiplier(margin_multiplier);
                if !disable_swap && snap_to_workspaces(w, &self.state.workspaces) {
                    self.state.sort_windows();
                }
                true
            }
            None => false,
        }
    }
}

fn process_window(window: &mut Window, offset_x: i32, offset_y: i32) {
    window.set_floating(true);
    let mut offset = window.get_floating_offsets().unwrap_or_default();
    let start = window.start_loc.unwrap_or_default();
    offset.set_x(start.x() + offset_x);
    offset.set_y(start.y() + offset_y);
    window.set_floating_offsets(Some(offset));
}

//if the windows is really close to a workspace, snap to it
fn snap_to_workspaces(window: &mut Window, workspaces: &[Workspace]) -> bool {
    workspaces
        .iter()
        .any(|workspace| should_snap(window, workspace))
}

//to be snapable, the window must be inside the workspace AND the a side must be close to
//the workspaces edge
fn should_snap(window: &mut Window, workspace: &Workspace) -> bool {
    if window.must_float() {
        return false;
    }
    let loc = window.calculated_xyhw();
    // Get window sides.
    let win_left = loc.x();
    let win_right = win_left + window.width();
    let win_top = loc.y();
    let win_bottom = win_top + window.height();
    // Check for conatins.
    let (center_x, center_y) = loc.center();
    if !workspace.contains_point(center_x, center_y) {
        return false;
    }

    // Check for close edge.
    let dist = 10;
    let ws_left = workspace.x();
    let ws_right = workspace.x() + workspace.width();
    let ws_top = workspace.y();
    let ws_bottom = workspace.y() + workspace.height();
    if vec![
        win_top - ws_top,
        win_bottom - ws_bottom,
        win_left - ws_left,
        win_right - ws_right,
    ]
    .iter()
    .map(|x| x.abs())
    .any(|x| x < dist)
    {
        return window.snap_to_workspace(workspace);
    }
    false
}
