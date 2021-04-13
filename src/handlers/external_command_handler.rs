use super::{command_handler, focus_handler, goto_tag_handler, Manager};
use crate::command::Command;
use crate::config::Config;
use crate::config::ThemeSetting;
use crate::utils::command_pipe::ExternalCommand;
use crate::utils::window_updater::update_windows;
pub fn process(manager: &mut Manager, config: &Config, command: ExternalCommand) -> bool {
    let needs_redraw = process_work(manager, config, command);
    if needs_redraw {
        update_windows(manager);
    }
    needs_redraw
}

fn process_work(manager: &mut Manager, config: &Config, command: ExternalCommand) -> bool {
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

        ExternalCommand::SendWindowToTag(tag_index) => {
            if tag_index < manager.tags.len() {
                //tag number as 1 based.
                let tag_num = format!("{}", tag_index + 1);
                return command_handler::process(
                    manager,
                    config,
                    &Command::MoveToTag,
                    Some(tag_num),
                );
            }
        }

        ExternalCommand::SetLayout(layout) => {
            command_handler::process(manager, config, &Command::SetLayout, Some(layout));
        }
        ExternalCommand::SetMarginMultiplier(margin_multiplier) => {
            command_handler::process(
                manager,
                config,
                &Command::SetMarginMultiplier,
                Some(margin_multiplier),
            );
        }

        ExternalCommand::SwapScreens => {
            return command_handler::process(manager, config, &Command::SwapTags, None);
        }

        ExternalCommand::MoveWindowToLastWorkspace => {
            return command_handler::process(manager, config, &Command::MoveToLastWorkspace, None);
        }

        ExternalCommand::FloatingToTile => {
            return command_handler::process(manager, config, &Command::FloatingToTile, None);
        }

        ExternalCommand::MoveWindowUp => {
            return command_handler::process(manager, config, &Command::MoveWindowUp, None);
        }
        ExternalCommand::MoveWindowDown => {
            return command_handler::process(manager, config, &Command::MoveWindowDown, None);
        }

        ExternalCommand::FocusWindowUp => {
            return command_handler::process(manager, config, &Command::FocusWindowUp, None);
        }
        ExternalCommand::FocusWindowDown => {
            return command_handler::process(manager, config, &Command::FocusWindowDown, None);
        }

        ExternalCommand::FocusNextTag => {
            return command_handler::process(manager, config, &Command::FocusNextTag, None);
        }
        ExternalCommand::FocusPreviousTag => {
            return command_handler::process(manager, config, &Command::FocusPreviousTag, None);
        }

        ExternalCommand::FocusWorkspaceNext => {
            return command_handler::process(manager, config, &Command::FocusWorkspaceNext, None);
        }
        ExternalCommand::FocusWorkspacePrevious => {
            return command_handler::process(
                manager,
                config,
                &Command::FocusWorkspacePrevious,
                None,
            );
        }
        ExternalCommand::NextLayout => {
            return command_handler::process(manager, config, &Command::NextLayout, None);
        }
        ExternalCommand::PreviousLayout => {
            return command_handler::process(manager, config, &Command::PreviousLayout, None);
        }

        ExternalCommand::CloseWindow => {
            return command_handler::process(manager, config, &Command::CloseWindow, None);
        }

        ExternalCommand::Reload | ExternalCommand::MoveWindowTop => {}
    }

    false
}
