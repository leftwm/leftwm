use super::*;
use crate::config::ThemeSetting;
use crate::utils::command_pipe::ExternalCommand;
use crate::utils::window_updater::update_windows;

pub fn process(manager: &mut Manager, command: ExternalCommand) -> bool {
    let needs_redraw = process_work(manager, command);
    if needs_redraw {
        update_windows(manager);
    }
    needs_redraw
}

fn process_work(manager: &mut Manager, command: ExternalCommand) -> bool {
    match command {
        ExternalCommand::UnloadTheme => {
            let theme = ThemeSetting::default();
            for win in &mut manager.windows {
                win.update_for_theme(&theme);
            }
            manager.theme_setting = theme;
            return true;
        }

        ExternalCommand::LoadTheme(path) => {
            let theme = ThemeSetting::load(&path);
            for win in &mut manager.windows {
                win.update_for_theme(&theme);
            }
            manager.theme_setting = theme;
            return true;
        }

        ExternalCommand::SendWorkspaceToTag(ws_index, tag_index) => {
            if ws_index < manager.workspaces.len() && tag_index < manager.tags.len() {
                let workspace = &manager.workspaces[ws_index].clone();
                focus_handler::focus_workspace(manager, workspace);
                goto_tag_handler::process(manager, tag_index + 1);
                return true;
            }
        }

        _ => {}
    }

    false
}
