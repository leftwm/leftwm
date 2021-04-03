use super::*;

pub fn process(manager: &mut Manager, handle: &WindowHandle, offset_x: i32, offset_y: i32) -> bool {
    let margin_multiplier = match manager
        .windows
        .iter()
        .filter(|other| other.has_tag(&manager.focused_tag(0).unwrap_or_default()))
        .last()
    {
        Some(w) => w.margin_multiplier(),
        _ => 1.0,
    };
    // for w in &mut manager.windows {
        // if &w.handle == handle {
            // process_window(w, offset_x, offset_y);
            // snap_to_workspaces(w, &manager.workspaces);
            // if w.type_ == WindowType::Normal {
                // w.apply_margin_multiplier(margin_multiplier);
                // log::info!("Margin multiplier applied through window snap.")
            // };
            // return true;
        // }
    // }
    // false
    match manager
        .windows
        .iter_mut()
        .find(|w| w.handle == handle.clone())
    {
        Some(w) => {
            process_window(w, offset_x, offset_y);
            w.apply_margin_multiplier(margin_multiplier);
            snap_to_workspaces(w, &manager.workspaces);
            true
        },
        None => false,
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
        .any(|workspace| should_snap(window, workspace.clone()))
}

//to be snapable, the window must be inside the workspace AND the a side must be close to
//the workspaces edge
fn should_snap(window: &mut Window, workspace: Workspace) -> bool {
    if window.must_float() {
        return false;
    }
    let loc = window.calculated_xyhw();
    //get window sides
    let win_left = loc.x();
    let win_right = win_left + window.width();
    let win_top = loc.y();
    let win_bottom = win_top + window.height();
    //check for conatins
    let center_x = loc.x() + (window.width() / 2);
    let center_y = loc.y() + (window.height() / 2);
    if !workspace.contains_point(center_x, center_y) {
        return false;
    }

    //check for close edge
    let dist = 10;
    let ws_left = workspace.x();
    let ws_right = workspace.x() + workspace.width();
    let ws_top = workspace.y();
    let ws_bottom = workspace.y() + workspace.height();
    if (win_top - ws_top).abs() < dist {
        return window_handler::snap_to_workspace(window, workspace);
    }
    if (win_bottom - ws_bottom).abs() < dist {
        return window_handler::snap_to_workspace(window, workspace);
    }
    if (win_left - ws_left).abs() < dist {
        return window_handler::snap_to_workspace(window, workspace);
    }
    if (win_right - ws_right).abs() < dist {
        return window_handler::snap_to_workspace(window, workspace);
    }
    false
}
