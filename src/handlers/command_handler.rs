#![allow(clippy::wildcard_imports)]
use super::*;
use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::utils::{child_process::exec_shell, helpers};
use std::str::FromStr;

/* Please also update src/bin/leftwm-check if any of the following apply after your update:
 * - a command now requires a value
 * - a command no longer requires a value
 * - a new command is introduced that requires a value
 *  */

pub fn process(
    mut manager: &mut Manager,
    config: &Config,
    command: &Command,
    val: Option<String>,
) -> bool {
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
                window.tag(&tag.id);
                let act = DisplayAction::SetWindowTags(window.handle, tag.id.clone());
                manager.actions.push_back(act);
                manager.sort_windows();
            }
            true
        }

        Command::GotoTag if val.is_none() => false,
        Command::GotoTag if !is_num(&val) => false,
        Command::GotoTag => {
            let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
            let previous_tag = manager.tag_index(&manager.focused_tag(1).unwrap_or_default());
            let input_tag = to_num(&val);
            let mut destination_tag = input_tag;

            if !config.disable_current_tag_swap {
                destination_tag = match (current_tag, previous_tag, input_tag) {
                    (Some(curr_tag), Some(prev_tag), inp_tag) if curr_tag + 1 == inp_tag => {
                        prev_tag + 1
                    } // if current tag is the same as the destination tag, go to the previous tag instead
                    (_, _, _) => input_tag, // go to the input tag tag
                };
            }
            goto_tag_handler::process(manager, destination_tag)
        }

        Command::FocusNextTag => {
            let current = manager.focused_tag(0);
            let current = current.unwrap();
            let mut index = match manager.tags.iter().position(|x| x.id == current) {
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
            let current = manager.focused_tag(0);
            let current = current.unwrap();
            let mut index = match manager.tags.iter().position(|x| x.id == current) {
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
            exec_shell(&val.unwrap(), &mut manager);
            false
        }

        Command::CloseWindow => {
            if let Some(window) = manager.focused_window() {
                if window.type_ != WindowType::Dock {
                    let act = DisplayAction::KillWindow(window.handle);
                    manager.actions.push_back(act);
                }
            }
            false
        }

        Command::SwapTags => {
            // If multi workspace, swap with other workspace
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
            // If single workspace, swap width previous tag
            if manager.workspaces.len() == 1 {
                let last = manager
                    .focused_tag_history
                    .get(1)
                    .map(std::string::ToString::to_string);
                if let Some(last) = last {
                    let tag_index = match manager.tags.iter().position(|x| x.id == last) {
                        Some(x) => x + 1,
                        None => return false,
                    };
                    return goto_tag_handler::process(manager, tag_index);
                }
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

        Command::SetLayout if val.is_none() => false,
        Command::SetLayout => {
            if let Some(layout) = to_layout(&val) {
                if let Some(workspace) = manager.focused_workspace_mut() {
                    workspace.set_layout(layout);
                    return true;
                }
            } // TODO: Print error message about invalid layout name
            false
        }

        Command::FloatingToTile => {
            let workspace = manager.focused_workspace().unwrap().clone();
            if let Some(window) = manager.focused_window_mut() {
                if window.must_float() {
                    return false;
                }
                //Not ideal as is_floating and must_float are connected so have to check
                //them separately
                if !window.floating() {
                    return false;
                }
                window_handler::snap_to_workspace(window, &workspace);
                let act = DisplayAction::MoveMouseOver(window.handle);
                manager.actions.push_back(act);
                return true;
            }
            false
        }

        Command::MoveWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle,
                None => {
                    return false;
                }
            };
            let (tags, layout) = match manager.focused_workspace() {
                Some(w) => (w.tags.clone(), Some(w.layout.clone())),
                None => {
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
            let mut act = DisplayAction::MoveMouseOver(handle);
            if let Some(crate::layouts::Layout::Monocle) = layout {
                // For Monocle we want to also move windows up/down
                // Not the best solution but results
                // in desired behaviour
                let new_handle = match helpers::relative_find(&to_reorder, is_handle, 1) {
                    Some(h) => h.handle,
                    None => {
                        return false;
                    }
                };
                act = DisplayAction::MoveMouseOver(new_handle);
            }
            manager.actions.push_back(act);
            true
        }

        Command::MoveWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle,
                None => {
                    return false;
                }
            };
            let (tags, layout) = match manager.focused_workspace() {
                Some(w) => (w.tags.clone(), Some(w.layout.clone())),
                None => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };

            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            let mut act = DisplayAction::MoveMouseOver(handle);
            helpers::reorder_vec(&mut to_reorder, is_handle, 1);
            manager.windows.append(&mut to_reorder);
            if let Some(crate::layouts::Layout::Monocle) = layout {
                // For Monocle we want to also move windows up/down
                // Not the best solution but results
                // in desired behaviour
                let new_handle = match helpers::relative_find(&to_reorder, is_handle, 1) {
                    Some(h) => h.handle,
                    None => {
                        return false;
                    }
                };
                act = DisplayAction::MoveMouseOver(new_handle);
            }

            manager.actions.push_back(act);
            true
        }
        // Moves the selected window at index 0 of the window list.
        // If the selected window is already at index 0, it is sent to index 1.
        Command::MoveWindowTop => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle,
                None => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                None => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            let list = &mut to_reorder;
            let len = list.len();
            let (index, item) = match list.iter().enumerate().find(|&x| is_handle(&x.1)) {
                Some(x) => (x.0, x.1.clone()),
                None => {
                    return false;
                }
            };
            list.remove(index);
            let mut new_index: usize = match index {
                0 => 1,
                _ => 0,
            };
            if new_index >= len {
                new_index -= len
            }
            list.insert(new_index, item);

            manager.windows.append(&mut to_reorder);
            // focus follows the window if it was not already on top of the stack
            if index > 0 {
                let act = DisplayAction::MoveMouseOver(handle);
                manager.actions.push_back(act);
            }
            true
        }

        Command::FocusWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle,
                None => {
                    return false;
                }
            };
            let (tags, layout) = match manager.focused_workspace() {
                Some(w) => (w.tags.clone(), Some(w.layout.clone())),
                None => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };

            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(crate::layouts::Layout::Monocle) = layout {
                let new_handle = match helpers::relative_find(&to_reorder, is_handle, 1) {
                    Some(h) => h.handle,
                    None => {
                        return false;
                    }
                };
                helpers::reorder_vec(&mut to_reorder, is_handle, -1);
                let act = DisplayAction::MoveMouseOver(new_handle);
                manager.actions.push_back(act);
            } else if let Some(new_focused) = helpers::relative_find(&to_reorder, is_handle, -1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle);
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut to_reorder);
            true
        }

        Command::FocusWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle,
                None => {
                    return false;
                }
            };
            let (tags, layout) = match manager.focused_workspace() {
                Some(w) => (w.tags.clone(), Some(w.layout.clone())),
                None => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };

            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(crate::layouts::Layout::Monocle) = layout {
                // For Monocle we want to also move windows up/down
                // Not the best solution but results in desired behaviour
                let new_handle = match helpers::relative_find(&to_reorder, is_handle, 1) {
                    Some(h) => h.handle,
                    None => {
                        return false;
                    }
                };
                helpers::reorder_vec(&mut to_reorder, is_handle, 1);
                let act = DisplayAction::MoveMouseOver(new_handle);
                manager.actions.push_back(act);
            } else if let Some(new_focused) = helpers::relative_find(&to_reorder, is_handle, -1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle);
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut to_reorder);
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
                focus_handler::move_cursor_over(manager, &window);
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
                focus_handler::move_cursor_over(manager, &window);
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

        Command::IncreaseMainWidth if val.is_none() => false,
        Command::IncreaseMainWidth => {
            let workspace = manager.focused_workspace_mut();
            if workspace.is_none() {
                return false;
            }
            let workspace = workspace.unwrap();
            let delta: u8 = (&val.unwrap()).parse().unwrap();
            workspace.increase_main_width(delta);
            true
        }
        Command::DecreaseMainWidth if val.is_none() => false,
        Command::DecreaseMainWidth => {
            let workspace = manager.focused_workspace_mut();
            if workspace.is_none() {
                return false;
            }
            let workspace = workspace.unwrap();
            let delta: u8 = (&val.unwrap()).parse().unwrap();
            workspace.decrease_main_width(delta);
            true
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

fn to_layout(val: &Option<String>) -> Option<Layout> {
    Layout::from_str(val.as_ref()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TagModel;
    #[test]
    fn go_to_tag_should_return_false_if_no_screen_is_created() {
        let mut manager = Manager::default();
        let config = Config::default();
        // no screen creation here
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("6".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("2".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("15".to_string())
            ),
            false
        );
    }

    #[test]
    fn go_to_tag_should_create_at_least_one_tag_per_screen_no_more() {
        let mut manager = Manager::default();
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        // no tag creation here but one tag per screen is created
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("2".to_string())
        ));
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("1".to_string())
        ));
        // we only have one tag per screen created automatically
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("3".to_string())
            ),
            false
        );
    }

    #[test]
    fn go_to_tag_should_return_false_on_invalid_input() {
        let mut manager = Manager::default();
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        manager.tags = vec![
            TagModel::new("A15"),
            TagModel::new("B24"),
            TagModel::new("C"),
            TagModel::new("6D4"),
            TagModel::new("E39"),
            TagModel::new("F67"),
        ];
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("abc".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                Some("ab45c".to_string())
            ),
            false
        );
        assert_eq!(
            process(&mut manager, &config, &Command::GotoTag, None),
            false
        );
    }

    #[test]
    fn go_to_tag_should_go_to_tag_and_set_history() {
        let mut manager = Manager {
            tags: vec![
                TagModel::new("A15"),
                TagModel::new("B24"),
                TagModel::new("C"),
                TagModel::new("6D4"),
                TagModel::new("E39"),
                TagModel::new("F67"),
            ],
            ..Manager::default()
        };
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());

        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("6".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(5));
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("2".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(1));

        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("3".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(2));

        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            Some("4".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(3));
        assert_eq!(
            manager.tag_index(&manager.focused_tag(1).unwrap_or_default()),
            Some(2)
        );
        assert_eq!(
            manager.tag_index(&manager.focused_tag(2).unwrap_or_default()),
            Some(1)
        );
        assert_eq!(
            manager.tag_index(&manager.focused_tag(3).unwrap_or_default()),
            Some(5)
        );
    }
}
