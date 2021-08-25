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
            load_theme(manager, theme)
        }
        ExternalCommand::LoadTheme(path) => {
            let theme = ThemeSetting::load(&path);
            load_theme(manager, theme)
        }
        ExternalCommand::ToggleScratchPad(name) => {
            command_handler::process(manager, config, &Command::ToggleScratchPad, &Some(name))
        }
        ExternalCommand::ToggleFullScreen => {
            command_handler::process(manager, config, &Command::ToggleFullScreen, &None)
        }
        ExternalCommand::SendWorkspaceToTag(ws_index, tag_index) => {
            send_workspace_to_tag(manager, ws_index, tag_index)
        }
        ExternalCommand::SendWindowToTag(tag_index) => {
            send_window_to_tag(manager, config, tag_index)
        }
        ExternalCommand::SetLayout(layout) => {
            command_handler::process(manager, config, &Command::SetLayout, &Some(layout))
        }
        ExternalCommand::SetMarginMultiplier(margin_multiplier) => command_handler::process(
            manager,
            config,
            &Command::SetMarginMultiplier,
            &Some(margin_multiplier),
        ),
        ExternalCommand::SwapScreens => {
            command_handler::process(manager, config, &Command::SwapTags, &None)
        }

        ExternalCommand::MoveWindowToLastWorkspace => {
            command_handler::process(manager, config, &Command::MoveToLastWorkspace, &None)
        }
        ExternalCommand::FloatingToTile => {
            command_handler::process(manager, config, &Command::FloatingToTile, &None)
        }
        ExternalCommand::MoveWindowUp => {
            command_handler::process(manager, config, &Command::MoveWindowUp, &None)
        }
        ExternalCommand::MoveWindowTop => {
            command_handler::process(manager, config, &Command::MoveWindowTop, &None)
        }
        ExternalCommand::MoveWindowDown => {
            command_handler::process(manager, config, &Command::MoveWindowDown, &None)
        }
        ExternalCommand::FocusWindowUp => {
            command_handler::process(manager, config, &Command::FocusWindowUp, &None)
        }
        ExternalCommand::FocusWindowDown => {
            command_handler::process(manager, config, &Command::FocusWindowDown, &None)
        }
        ExternalCommand::FocusNextTag => {
            command_handler::process(manager, config, &Command::FocusNextTag, &None)
        }
        ExternalCommand::FocusPreviousTag => {
            command_handler::process(manager, config, &Command::FocusPreviousTag, &None)
        }
        ExternalCommand::FocusWorkspaceNext => {
            command_handler::process(manager, config, &Command::FocusWorkspaceNext, &None)
        }
        ExternalCommand::FocusWorkspacePrevious => {
            command_handler::process(manager, config, &Command::FocusWorkspacePrevious, &None)
        }
        ExternalCommand::NextLayout => {
            command_handler::process(manager, config, &Command::NextLayout, &None)
        }
        ExternalCommand::PreviousLayout => {
            command_handler::process(manager, config, &Command::PreviousLayout, &None)
        }
        ExternalCommand::RotateTag => {
            command_handler::process(manager, config, &Command::RotateTag, &None)
        }
        ExternalCommand::CloseWindow => {
            command_handler::process(manager, config, &Command::CloseWindow, &None)
        }
        ExternalCommand::Reload => {
            command_handler::process(manager, config, &Command::SoftReload, &None)
        }
    }
}

fn load_theme(manager: &mut Manager, theme: ThemeSetting) -> bool {
    for win in &mut manager.windows {
        win.update_for_theme(&theme);
    }
    for ws in &mut manager.workspaces {
        ws.update_for_theme(&theme);
    }
    manager.theme_setting = theme;
    true
}

fn send_workspace_to_tag(manager: &mut Manager, ws_index: usize, tag_index: usize) -> bool {
    if ws_index < manager.workspaces.len() && tag_index < manager.tags.len() {
        let workspace = &manager.workspaces[ws_index].clone();
        focus_handler::focus_workspace(manager, workspace);
        goto_tag_handler::process(manager, tag_index + 1);
        return true;
    }
    false
}

fn send_window_to_tag(manager: &mut Manager, config: &Config, tag_index: usize) -> bool {
    if tag_index < manager.tags.len() {
        //tag number as 1 based.
        let tag_num = format!("{}", tag_index + 1);
        return command_handler::process(manager, config, &Command::MoveToTag, &Some(tag_num));
    }
    false
}
