use super::{focus_handler, Manager, Window, WindowChange, WindowType, Workspace};
use crate::child_process::exec_shell;
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::models::WindowHandle;
use crate::utils::helpers;

/// Process a collection of events, and apply them changes to a manager.
/// Returns true if changes need to be rendered.
pub fn created(mut manager: &mut Manager, mut window: Window, x: i32, y: i32) -> bool {
    //don't add the window if the manager already knows about it
    for w in &manager.windows {
        if w.handle == window.handle {
            return false;
        }
    }

    //When adding a window we add to the workspace under the cursor, This isn't necessarily the
    //focused workspace. If the workspace is empty, it might not have received focus. This is so
    //the workspace that has windows on its is still active not the empty workspace.
    let ws: Option<&Workspace> = manager
        .workspaces
        .iter()
        .find(|ws| ws.xyhw.contains_point(x, y))
        .or_else(|| manager.focused_workspace()); //backup plan

    //Random value
    let mut layout: Layout = Layout::MainAndVertStack;
    if let Some(ws) = ws {
        window.tags = ws.tags.clone();
        layout = ws.layout.clone();
        //if dialog, center in workspace
        if window.type_ == WindowType::Dialog {
            window.set_floating(true);
            let new_float_exact = ws.center_halfed();
            window.normal = ws.xyhw;
            window.set_floating_exact(new_float_exact);
        }
        if window.type_ == WindowType::Splash {
            if let Some(requested) = window.requested {
                window.normal = ws.xyhw;
                requested.update_window_floating(&mut window);
                let mut xhyw = window.get_floating_offsets().unwrap_or_default();
                xhyw.center_relative(ws.xyhw, window.border, window.requested);
                window.set_floating_offsets(Some(xhyw));
            } else {
                window.set_floating(true);
                let new_float_exact = ws.center_halfed();
                window.normal = ws.xyhw;
                window.set_floating_exact(new_float_exact);
            }
        }
    } else {
        window.tags = vec![manager.tags[0].id.clone()]
    }
    // if there is a window with an applied margin multiplier present use this multiplier
    // in any other case use default margins (multiplier '1.0')
    if window.type_ == WindowType::Normal {
        let margin_multiplier = match manager.focused_window() {
            Some(w) => w.margin_multiplier(),
            _ => 1.0,
        };
        window.apply_margin_multiplier(margin_multiplier);
    }

    if let Some(trans) = &window.transient {
        if let Some(parent) = find_parent_window(manager, &trans) {
            window.set_floating(true);
            let new_float_exact = parent.calculated_xyhw().center_halfed();
            window.normal = parent.normal;
            window.set_floating_exact(new_float_exact);
        }
    }

    window.update_for_theme(&manager.theme_setting);

    //Dirty
    if (Layout::Monocle == layout || Layout::MainAndDeck == layout)
        && window.type_ == WindowType::Normal
    {
        let for_active_workspace = |x: &Window| -> bool {
            helpers::intersect(&window.tags, &x.tags) && x.type_ != WindowType::Dock
        };

        let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
        if Layout::Monocle == layout {
            to_reorder.insert(0, window.clone());
        } else {
            to_reorder.insert(1, window.clone());
        }
        manager.windows.append(&mut to_reorder);
    } else {
        manager.windows.push(window.clone());
    }

    //let the DS know we are managing this window
    let act = DisplayAction::AddedWindow(window.handle);
    manager.actions.push_back(act);

    //let the DS know the correct desktop to find this window
    if !window.tags.is_empty() {
        let act = DisplayAction::SetWindowTags(window.handle, window.tags[0].clone());
        manager.actions.push_back(act);
    }

    // tell the WM the new display order of the windows
    //new windows should be on the top of the stack
    manager.sort_windows();

    focus_handler::focus_window(manager, &window.handle);

    if let Some(cmd) = &manager.theme_setting.on_new_window_cmd.clone() {
        exec_shell(&cmd, &mut manager);
    }

    true
}

/// Process a collection of events, and apply them changes to a manager.
/// Returns true if changes need to be rendered.
pub fn destroyed(manager: &mut Manager, handle: &WindowHandle) -> bool {
    manager.windows = manager
        .windows
        .iter()
        .filter(|w| &w.handle != handle)
        .cloned()
        .collect();

    //make sure the workspaces do not draw on the docks
    update_workspace_avoid_list(manager);

    //make sure focus is recalculated
    let act = DisplayAction::FocusWindowUnderCursor;
    manager.actions.push_back(act);

    true
}

pub fn changed(manager: &mut Manager, change: WindowChange) -> bool {
    for w in &mut manager.windows {
        if w.handle == change.handle {
            log::debug!("WINDOW CHANGED {:?} {:?}", &w, change);
            //let old_type = w.type_.clone();
            let changed = change.update(w);
            if w.type_ == WindowType::Dock {
                update_workspace_avoid_list(manager);
                //don't left changes from docks re-render the worker. This will result in an
                //infinite loop. Just be patient a rerender will occur.
                //return true;
            }
            return changed;
        }
    }
    false
}

fn find_window<'w>(manager: &'w Manager, handle: &WindowHandle) -> Option<&'w Window> {
    for win in &manager.windows {
        if &win.handle == handle {
            let r: &Window = win;
            return Some(r);
        }
    }
    None
}

fn find_parent_window<'w>(manager: &'w Manager, handle: &WindowHandle) -> Option<&'w Window> {
    let mut w = find_window(manager, handle);
    while w.is_some() && w.unwrap().transient.is_some() {
        let tran = w.unwrap().transient.clone().unwrap();
        w = find_window(manager, &tran);
    }
    w
}

pub fn update_workspace_avoid_list(manager: &mut Manager) {
    let mut avoid = vec![];
    for w in &manager.windows {
        if w.type_ == WindowType::Dock && w.strut.is_some() {
            if let Some(to_avoid) = w.strut {
                log::debug!("AVOID STRUT:[{:?}] {:?}", w.handle, to_avoid);
                avoid.push(to_avoid);
            }
        }
    }
    for w in &mut manager.workspaces {
        w.avoid = avoid.clone();
        w.update_avoided_areas();
    }
}

pub fn snap_to_workspace(window: &mut Window, workspace: &Workspace) -> bool {
    window.debugging = true;
    window.set_floating(false);

    //we are reparenting
    if window.tags != workspace.tags {
        window.debugging = true;
        window.tags = workspace.tags.clone();
        let mut offset = window.get_floating_offsets().unwrap_or_default();
        let mut start_loc = window.start_loc.unwrap_or_default();
        let x = offset.x() + window.normal.x();
        let y = offset.y() + window.normal.y();
        offset.set_x(x - workspace.xyhw.x());
        offset.set_y(y - workspace.xyhw.y());
        window.set_floating_offsets(Some(offset));

        let x = start_loc.x() + window.normal.x();
        let y = start_loc.y() + window.normal.y();
        start_loc.set_x(x - workspace.xyhw.x());
        start_loc.set_y(y - workspace.xyhw.y());
        window.start_loc = Some(start_loc);
    }
    true
}
