use super::{focus_handler, Manager, Window, WindowChange, WindowType, Workspace};
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::models::WindowHandle;
use crate::utils::helpers;
use crate::{child_process::exec_shell, models::FocusBehaviour};

/// Process a collection of events, and apply them changes to a manager.
/// Returns true if changes need to be rendered.
pub fn created(mut manager: &mut Manager, mut window: Window, x: i32, y: i32) -> bool {
    //don't add the window if the manager already knows about it
    if manager.windows.iter().any(|w| w.handle == window.handle) {
        return false;
    }

    let mut is_first = false;
    //Random value
    let mut layout: Layout = Layout::MainAndVertStack;
    let is_scratchpad = is_scratchpad(manager, &window);
    setup_window(
        manager,
        &mut window,
        x,
        y,
        is_scratchpad,
        &mut layout,
        &mut is_first,
    );
    insert_window(manager, &mut window, is_scratchpad, &layout);

    let follow_mouse = manager.focus_manager.focus_new_windows
        || manager.focus_manager.behaviour == FocusBehaviour::Sloppy;
    //let the DS know we are managing this window
    let act = DisplayAction::AddedWindow(window.handle, follow_mouse);
    manager.actions.push_back(act);

    //let the DS know the correct desktop to find this window
    if !window.tags.is_empty() {
        let act = DisplayAction::SetWindowTags(window.handle, window.tags[0].clone());
        manager.actions.push_back(act);
    }

    // tell the WM the new display order of the windows
    //new windows should be on the top of the stack
    manager.sort_windows();

    if manager.focus_manager.focus_new_windows || is_first {
        focus_handler::focus_window(manager, &window.handle);
    }

    if let Some(cmd) = &manager.theme_setting.on_new_window_cmd.clone() {
        exec_shell(cmd, &mut manager);
    }

    true
}

fn setup_window(
    manager: &mut Manager,
    window: &mut Window,
    x: i32,
    y: i32,
    is_scratchpad: bool,
    layout: &mut Layout,
    is_first: &mut bool,
) {
    //When adding a window we add to the workspace under the cursor, This isn't necessarily the
    //focused workspace. If the workspace is empty, it might not have received focus. This is so
    //the workspace that has windows on its is still active not the empty workspace.
    let ws: Option<&Workspace> = manager
        .workspaces
        .iter()
        .find(|ws| {
            ws.xyhw.contains_point(x, y)
                && manager.focus_manager.behaviour == FocusBehaviour::Sloppy
        })
        .or_else(|| manager.focused_workspace()); //backup plan

    if let Some(ws) = ws {
        let for_active_workspace =
            |x: &Window| -> bool { helpers::intersect(&ws.tags, &x.tags) && !x.is_unmanaged() };
        *is_first = !manager.windows.iter().any(|w| for_active_workspace(w));
        window.tags = ws.tags.clone();
        *layout = ws.layout.clone();

        if is_scratchpad {
            window.set_floating(true);
            let new_float_exact = ws.center_halfed();
            window.normal = ws.xyhw;
            window.set_floating_exact(new_float_exact);
        }
        if window.type_ == WindowType::Normal {
            window.apply_margin_multiplier(ws.margin_multiplier);
        }
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
                requested.update_window_floating(window);
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
        window.tags = vec![manager.tags[0].id.clone()];
        if is_scratchpad {
            window.tag("NSP");
            window.set_floating(true);
        }
    }

    if let Some(parent) = find_transient_parent(manager, window) {
        window.set_floating(true);
        let new_float_exact = parent.calculated_xyhw().center_halfed();
        window.normal = parent.normal;
        window.set_floating_exact(new_float_exact);
    }

    window.update_for_theme(&manager.theme_setting);
}
fn insert_window(manager: &mut Manager, window: &mut Window, is_scratchpad: bool, layout: &Layout) {
    // If the tag contains a fullscreen window, minimize it
    let for_active_workspace =
        |x: &Window| -> bool { helpers::intersect(&window.tags, &x.tags) && !x.is_unmanaged() };
    let mut was_fullscreen = false;
    if let Some(fsw) = manager
        .windows
        .iter_mut()
        .find(|w| for_active_workspace(w) && w.is_fullscreen())
    {
        if let Some(act) = fsw.toggle_fullscreen() {
            manager.actions.push_back(act);
            was_fullscreen = true;
        }
    }

    if (&Layout::Monocle == layout || &Layout::MainAndDeck == layout)
        && window.type_ == WindowType::Normal
    {
        let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
        if &Layout::Monocle == layout || to_reorder.is_empty() {
            if was_fullscreen {
                if let Some(act) = window.toggle_fullscreen() {
                    manager.actions.push_back(act);
                }
            }
            to_reorder.insert(0, window.clone());
        } else {
            to_reorder.insert(1, window.clone());
        }
        manager.windows.append(&mut to_reorder);
    } else if window.type_ == WindowType::Dialog
        || window.type_ == WindowType::Splash
        || is_scratchpad
    {
        //Slow
        manager.windows.insert(0, window.clone());
    } else {
        manager.windows.push(window.clone());
    }
}

fn is_scratchpad(manager: &Manager, window: &Window) -> bool {
    manager
        .active_scratchpads
        .iter()
        .any(|(_, &id)| window.pid == id)
}

/// Process a collection of events, and apply them changes to a manager.
/// Returns true if changes need to be rendered.
pub fn destroyed(manager: &mut Manager, handle: &WindowHandle) -> bool {
    let sloppy = manager.focus_manager.behaviour == FocusBehaviour::Sloppy;
    //Find the next or previous window on the workspace
    let mut new_handle = None;
    if !sloppy {
        if let Some(ws) = manager.focused_workspace().cloned() {
            let for_active_workspace = |x: &Window| -> bool { ws.is_managed(x) };
            let mut windows = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == *handle };
            let p = helpers::relative_find(&windows, is_handle, -1, false);
            new_handle = helpers::relative_find(&windows, for_active_workspace, 1, false)
                .or(p) //Backup
                .map(|w| w.handle);
            manager.windows.append(&mut windows);
        }
    }
    manager
        .focus_manager
        .tags_last_window
        .retain(|_, h| h != handle);
    manager.windows.retain(|w| &w.handle != handle);

    //make sure the workspaces do not draw on the docks
    update_workspace_avoid_list(manager);

    let focused = manager.focus_manager.window_history.get(0);
    //make sure focus is recalculated if we closed the currently focused window
    if focused == Some(handle) {
        if sloppy {
            let act = DisplayAction::FocusWindowUnderCursor;
            manager.actions.push_back(act);
        } else if let Some(h) = new_handle {
            focus_handler::focus_window(manager, &h);
        }
    }

    true
}

pub fn changed(manager: &mut Manager, change: WindowChange) -> bool {
    let mut changed = false;
    let strut_changed = change.strut.is_some();
    if let Some(w) = manager
        .windows
        .iter_mut()
        .find(|w| w.handle == change.handle)
    {
        log::debug!("WINDOW CHANGED {:?} {:?}", &w, change);
        changed = change.update(w);
        if w.type_ == WindowType::Dock {
            update_workspace_avoid_list(manager);
            //don't left changes from docks re-render the worker. This will result in an
            //infinite loop. Just be patient a rerender will occur.
        }
    }
    if strut_changed {
        manager.update_docks();
    }
    changed
}

fn find_window<'w>(manager: &'w Manager, handle: &WindowHandle) -> Option<&'w Window> {
    manager.windows.iter().find(|w| &w.handle == handle)
}

fn find_transient_parent<'w>(manager: &'w Manager, window: &Window) -> Option<&'w Window> {
    let handle = &window.transient?;
    let mut w: &Window = find_window(manager, handle)?;
    while let Some(tran) = w.transient {
        w = find_window(manager, &tran)?;
    }
    Some(w)
}

pub fn update_workspace_avoid_list(manager: &mut Manager) {
    let mut avoid = vec![];
    manager
        .windows
        .iter()
        .filter(|w| w.type_ == WindowType::Dock && w.strut.is_some())
        .for_each(|w| {
            //unwrap() is safe as we know w.strut is_some
            let to_avoid = w.strut.unwrap();
            log::debug!("AVOID STRUT:[{:?}] {:?}", w.handle, to_avoid);
            avoid.push(to_avoid);
        });
    for ws in &mut manager.workspaces {
        let struts = avoid
            .clone()
            .into_iter()
            .filter(|s| {
                let (x, y) = s.center();
                ws.contains_point(x, y)
            })
            .collect();
        ws.avoid = struts;
        ws.update_avoided_areas();
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
