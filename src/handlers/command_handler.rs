use super::*;
use crate::display_action::DisplayAction;
use crate::utils::helpers;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag if val.is_none() => false,
        Command::MoveToTag if !is_num(&val) => false,
        Command::MoveToTag if to_num(&val) > manager.tags.len() => false,
        Command::MoveToTag if to_num(&val) < 1 => false,
        Command::MoveToTag => {
            let tag_num = to_num(&val);
            let tag = manager.tags[tag_num - 1].clone();
            if let Some(window) = manager.focused_window_mut() {
                window.clear_tags();
                window.set_floating(false);
                window.tag(tag.clone());
                let act = DisplayAction::SetWindowTags(window.handle.clone(), tag.clone());
                manager.actions.push_back(act);
                return true;
            }
            false
        }

        Command::GotoTag if val.is_none() => false,
        Command::GotoTag if !is_num(&val) => false,
        Command::GotoTag => goto_tag_handler::process(manager, to_num(&val)),

        Command::FocusNextTag => {
            let current = manager.focused_tag();
            let current = current.unwrap();
            let mut index = match manager.tags.iter().position(|x| x == &current) {
                Some(x) => x + 1,
                None => {
                    return false;
                }
            };

            index += 1;

            if index > manager.tags.len() {
                index = 1;
            }

            goto_tag_handler::process(manager, index)
        }

        Command::FocusPreviousTag => {
            let current = manager.focused_tag();
            let current = current.unwrap();
            let mut index = match manager.tags.iter().position(|x| x == &current) {
                Some(x) => x + 1,
                None => {
                    return false;
                }
            };

            index -= 1;
            if index < 1 {
                index = manager.tags.len();
            }

            goto_tag_handler::process(manager, index)
        }

        Command::Execute if val.is_none() => false,
        Command::Execute => {
            use std::process::{Command, Stdio};
            let _ = Command::new("sh")
                .arg("-c")
                .arg(&val.unwrap())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .map(|child| manager.children.insert(child));
            false
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

        Command::MoveWindowMaster => {
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
            // helpers::reorder_vec(&mut to_reorder, is_handle, -3);
            // set first in line
            let list = &mut to_reorder;
            let len = list.len() as i32;
            let (index, item) = match list.iter().enumerate().find(|&x| is_handle(&x.1)) {
                Some(x) => (x.0, x.1.clone()),
                None => {
                    return false;
                }
            };
            list.remove(index);
            let mut new_index: i32;
            match index {
                0 => new_index = 1,
                _ => {
                    new_index = 0;
                }
            }
            if new_index < 0 {
                new_index += len
            }
            if new_index >= len {
                new_index -= len
            }
            list.insert(new_index as usize, item);

            // set first in line
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

        Command::FocusWorkspaceNext => {
            let current = manager.focused_workspace();
            if current.is_none() {
                return false;
            }
            let current = current.unwrap();
            let mut index = match manager
                .workspaces
                .iter()
                .enumerate()
                .find(|&x| x.1 == current)
            {
                Some(x) => x.0,
                None => {
                    return false;
                }
            };
            index += 1;
            if index >= manager.workspaces.len() {
                index = 0;
            }
            let workspace = manager.workspaces[index].clone();
            focus_handler::focus_workspace(manager, &workspace);
            let act = DisplayAction::MoveMouseOverPoint(workspace.xyhw.center());
            manager.actions.push_back(act);
            if let Some(window) = manager
                .windows
                .iter()
                .find(|w| workspace.is_displaying(w) && w.type_ == WindowType::Normal)
            {
                let window = window.clone();
                focus_handler::focus_window(manager, &window, &window.x() + 1, &window.y() + 1);
                let act = DisplayAction::MoveMouseOver(window.handle);
                manager.actions.push_back(act);
            }
            true
        }

        Command::FocusWorkspacePrevious => {
            let current = manager.focused_workspace();
            if current.is_none() {
                return false;
            }
            let current = current.unwrap();
            let mut index = match manager
                .workspaces
                .iter()
                .enumerate()
                .find(|&x| x.1 == current)
            {
                Some(x) => x.0 as i32,
                None => {
                    return false;
                }
            };
            index -= 1;
            if index < 0 {
                index = (manager.workspaces.len() as i32) - 1;
            }
            let workspace = manager.workspaces[index as usize].clone();
            focus_handler::focus_workspace(manager, &workspace);
            let act = DisplayAction::MoveMouseOverPoint(workspace.xyhw.center());
            manager.actions.push_back(act);
            if let Some(window) = manager
                .windows
                .iter()
                .find(|w| workspace.is_displaying(w) && w.type_ == WindowType::Normal)
            {
                let window = window.clone();
                focus_handler::focus_window(manager, &window, &window.x() + 1, &window.y() + 1);
                let act = DisplayAction::MoveMouseOver(window.handle);
                manager.actions.push_back(act);
            }
            true
        }

        Command::MouseMoveWindow => false,

        Command::SoftReload => {
            manager.soft_reload();
            false
        }
        Command::HardReload => {
            manager.hard_reload();
            false
        }
    }
}

/// Is the string passed in a valid number
fn is_num(val: &Option<String>) -> bool {
    match val {
        Some(num) => num.parse::<usize>().is_ok(),
        None => false,
    }
}

/// Convert the option string to a number
fn to_num(val: &Option<String>) -> usize {
    val.as_ref()
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or_default()
}
