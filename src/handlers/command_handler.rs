use super::*;
use crate::display_action::DisplayAction;
use crate::utils::logging::*;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag => {
            if let Some(tag) = val {
                if let Some(window) = manager.focused_window_mut() {
                    window.clear_tags();
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
                use std::process::Command;
                let _ = Command::new("sh").arg("-c").arg(&cmd).spawn();
                log_info("EXECUTE", &cmd);
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
    }
}
