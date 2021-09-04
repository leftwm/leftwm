use super::{Command, Manager};
use crate::config::Config;
use crate::config::ThemeLoader;
use crate::config::ThemeSetting;
use crate::state::State;
use crate::utils::command_pipe::ExternalCommand;
use std::sync::Arc;

impl<CMD> Manager<CMD> {
    pub fn external_command_handler(
        &mut self,
        state: &impl State,
        config: &impl Config,
        theme_loader: &impl ThemeLoader,
        command: ExternalCommand,
    ) -> bool {
        let needs_redraw = process_work(self, state, config, theme_loader, command);
        if needs_redraw {
            self.update_windows();
        }
        needs_redraw
    }
}

fn process_work<CMD>(
    manager: &mut Manager<CMD>,
    state: &impl State,
    config: &impl Config,
    theme_loader: &impl ThemeLoader,
    command: ExternalCommand,
) -> bool {
    match command {
        ExternalCommand::UnloadTheme => {
            let theme = theme_loader.default();
            load_theme(manager, theme)
        }
        ExternalCommand::LoadTheme(path) => {
            let theme = theme_loader.load(&path);
            load_theme(manager, theme)
        }
        ExternalCommand::ToggleScratchPad(name) => {
            manager.command_handler(state, config, &Command::ToggleScratchPad, &Some(name))
        }
        ExternalCommand::ToggleFullScreen => {
            manager.command_handler(state, config, &Command::ToggleFullScreen, &None)
        }
        ExternalCommand::SendWorkspaceToTag(ws_index, tag_index) => {
            send_workspace_to_tag(manager, ws_index, tag_index)
        }
        ExternalCommand::SendWindowToTag(tag_index) => {
            send_window_to_tag(manager, state, config, tag_index)
        }
        ExternalCommand::SetLayout(layout) => {
            manager.command_handler(state, config, &Command::SetLayout, &Some(layout))
        }
        ExternalCommand::SetMarginMultiplier(margin_multiplier) => manager.command_handler(
            state,
            config,
            &Command::SetMarginMultiplier,
            &Some(margin_multiplier),
        ),
        ExternalCommand::SwapScreens => {
            manager.command_handler(state, config, &Command::SwapTags, &None)
        }
        ExternalCommand::MoveWindowToLastWorkspace => {
            manager.command_handler(state, config, &Command::MoveToLastWorkspace, &None)
        }
        ExternalCommand::FloatingToTile => {
            manager.command_handler(state, config, &Command::FloatingToTile, &None)
        }
        ExternalCommand::MoveWindowUp => {
            manager.command_handler(state, config, &Command::MoveWindowUp, &None)
        }
        ExternalCommand::MoveWindowTop => {
            manager.command_handler(state, config, &Command::MoveWindowTop, &None)
        }
        ExternalCommand::MoveWindowDown => {
            manager.command_handler(state, config, &Command::MoveWindowDown, &None)
        }
        ExternalCommand::FocusWindowUp => {
            manager.command_handler(state, config, &Command::FocusWindowUp, &None)
        }
        ExternalCommand::FocusWindowDown => {
            manager.command_handler(state, config, &Command::FocusWindowDown, &None)
        }
        ExternalCommand::FocusNextTag => {
            manager.command_handler(state, config, &Command::FocusNextTag, &None)
        }
        ExternalCommand::FocusPreviousTag => {
            manager.command_handler(state, config, &Command::FocusPreviousTag, &None)
        }
        ExternalCommand::FocusWorkspaceNext => {
            manager.command_handler(state, config, &Command::FocusWorkspaceNext, &None)
        }
        ExternalCommand::FocusWorkspacePrevious => {
            manager.command_handler(state, config, &Command::FocusWorkspacePrevious, &None)
        }
        ExternalCommand::NextLayout => {
            manager.command_handler(state, config, &Command::NextLayout, &None)
        }
        ExternalCommand::PreviousLayout => {
            manager.command_handler(state, config, &Command::PreviousLayout, &None)
        }
        ExternalCommand::RotateTag => {
            manager.command_handler(state, config, &Command::RotateTag, &None)
        }
        ExternalCommand::CloseWindow => {
            manager.command_handler(state, config, &Command::CloseWindow, &None)
        }
        ExternalCommand::Reload => {
            manager.command_handler(state, config, &Command::SoftReload, &None)
        }
    }
}

fn load_theme<CMD>(manager: &mut Manager<CMD>, theme: ThemeSetting) -> bool {
    for win in &mut manager.windows {
        win.update_for_theme(&theme);
    }
    for ws in &mut manager.workspaces {
        ws.update_for_theme(&theme);
    }
    manager.theme_setting = Arc::new(theme);
    true
}

fn send_workspace_to_tag<CMD>(
    manager: &mut Manager<CMD>,
    ws_index: usize,
    tag_index: usize,
) -> bool {
    if ws_index < manager.workspaces.len() && tag_index < manager.tags.len() {
        let workspace = &manager.workspaces[ws_index].clone();
        manager.focus_workspace(workspace);
        manager.goto_tag_handler(tag_index + 1);
        return true;
    }
    false
}

fn send_window_to_tag<CMD>(
    manager: &mut Manager<CMD>,
    state: &impl State,
    config: &impl Config,
    tag_index: usize,
) -> bool {
    if tag_index < manager.tags.len() {
        //tag number as 1 based.
        let tag_num = format!("{}", tag_index + 1);
        return manager.command_handler(state, config, &Command::MoveToTag, &Some(tag_num));
    }
    false
}
