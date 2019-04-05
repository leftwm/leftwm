use super::*;
use crate::display_action::DisplayAction;
use crate::utils::helpers;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag => {
            if let Some(tag) = val {
                if let Some(window) = manager.focused_window_mut() {
                    window.clear_tags();
                    window.set_floating(false);
                    window.tag(tag);
                    return true;
                }
            }
            false
        }

        Command::GotoTag => {
            if let Some(tag) = val {
                goto_tag_handler::process(manager, tag)
            } else {
                false
            }
        }

        Command::Execute => {
            if let Some(cmd) = val {
                use std::process::{Command, Stdio};
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .spawn();
                false
            } else {
                false
            }
        }

        Command::CloseWindow => {
            if let Some(window) = manager.focused_window() {
                let act = DisplayAction::KillWindow(window.handle.clone());
                manager.actions.push_back(act);
            }
            false
        }

        Command::SwapTags => {
            if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
                let mut a = manager.workspaces[manager.focused_workspace_history[0]].clone();
                let mut b = manager.workspaces[manager.focused_workspace_history[1]].clone();
                let swap = a.tags.clone();
                a.tags = b.tags.clone();
                b.tags = swap;
                manager.workspaces[manager.focused_workspace_history[0]] = a;
                manager.workspaces[manager.focused_workspace_history[1]] = b;
                return true;
            }
            false
        }

        Command::MoveToLastWorkspace => {
            if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
                let wp_tags = &manager.workspaces[manager.focused_workspace_history[1]]
                    .tags
                    .clone();
                if let Some(window) = manager.focused_window_mut() {
                    window.tags = vec![wp_tags[0].clone()];
                    return true;
                }
            }
            false
        }

        Command::NextLayout => {
            if let Some(workspace) = manager.focused_workspace_mut() {
                workspace.next_layout();
                return true;
            }
            false
        }
        Command::PreviousLayout => {
            if let Some(workspace) = manager.focused_workspace_mut() {
                workspace.prev_layout();
                return true;
            }
            false
        }

        Command::MoveWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            helpers::reorder_vec(&mut to_reorder, is_handle, -1);
            manager.windows.append(&mut to_reorder);
            let act = DisplayAction::MoveMouseOver(handle);
            manager.actions.push_back(act);
            true
        }

        Command::MoveWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            helpers::reorder_vec(&mut to_reorder, is_handle, 1);
            manager.windows.append(&mut to_reorder);
            let act = DisplayAction::MoveMouseOver(handle);
            manager.actions.push_back(act);
            true
        }

        Command::FocusWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut window_group = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(new_focused) = helpers::relative_find(&window_group, is_handle, -1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle.clone());
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut window_group);
            true
        }

        Command::FocusWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut window_group = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(new_focused) = helpers::relative_find(&window_group, is_handle, 1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle.clone());
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut window_group);
            true
        }

        Command::MouseMoveWindow => false,
    }
}
