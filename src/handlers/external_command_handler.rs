use super::{Command, Manager};
use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::utils::command_pipe::ExternalCommand;

impl<C: Config<CMD>, SERVER: DisplayServer<CMD>, CMD> Manager<C, CMD, SERVER> {
    pub fn external_command_handler(&mut self, command: ExternalCommand) -> bool {
        let needs_redraw = process_work(self, command);
        if needs_redraw {
            self.update_windows();
        }
        needs_redraw
    }
}

fn process_work<C: Config<CMD>, SERVER: DisplayServer<CMD>, CMD>(
    manager: &mut Manager<C, CMD, SERVER>,
    command: ExternalCommand,
) -> bool {
    match command {
        ExternalCommand::ToggleScratchPad(name) => {
            manager.command_handler(&Command::ToggleScratchPad, Some(&name))
        }
        ExternalCommand::ToggleFullScreen => {
            manager.command_handler(&Command::ToggleFullScreen, None)
        }
        ExternalCommand::SendWorkspaceToTag(ws_index, tag_index) => {
            send_workspace_to_tag(manager, ws_index, tag_index)
        }
        ExternalCommand::SendWindowToTag(tag_index) => send_window_to_tag(manager, tag_index),
        ExternalCommand::SetLayout(layout) => {
            manager.command_handler(&Command::SetLayout, Some(&layout))
        }
        ExternalCommand::SetMarginMultiplier(margin_multiplier) => {
            manager.command_handler(&Command::SetMarginMultiplier, Some(&margin_multiplier))
        }
        ExternalCommand::SwapScreens => manager.command_handler(&Command::SwapTags, None),
        ExternalCommand::MoveWindowToLastWorkspace => {
            manager.command_handler(&Command::MoveToLastWorkspace, None)
        }
        ExternalCommand::FloatingToTile => manager.command_handler(&Command::FloatingToTile, None),
        ExternalCommand::MoveWindowUp => manager.command_handler(&Command::MoveWindowUp, None),
        ExternalCommand::MoveWindowTop => manager.command_handler(&Command::MoveWindowTop, None),
        ExternalCommand::MoveWindowDown => manager.command_handler(&Command::MoveWindowDown, None),
        ExternalCommand::FocusWindowUp => manager.command_handler(&Command::FocusWindowUp, None),
        ExternalCommand::FocusWindowDown => {
            manager.command_handler(&Command::FocusWindowDown, None)
        }
        ExternalCommand::FocusNextTag => manager.command_handler(&Command::FocusNextTag, None),
        ExternalCommand::FocusPreviousTag => {
            manager.command_handler(&Command::FocusPreviousTag, None)
        }
        ExternalCommand::FocusWorkspaceNext => {
            manager.command_handler(&Command::FocusWorkspaceNext, None)
        }
        ExternalCommand::FocusWorkspacePrevious => {
            manager.command_handler(&Command::FocusWorkspacePrevious, None)
        }
        ExternalCommand::NextLayout => manager.command_handler(&Command::NextLayout, None),
        ExternalCommand::PreviousLayout => manager.command_handler(&Command::PreviousLayout, None),
        ExternalCommand::RotateTag => manager.command_handler(&Command::RotateTag, None),
        ExternalCommand::CloseWindow => manager.command_handler(&Command::CloseWindow, None),
        ExternalCommand::Reload => manager.command_handler(&Command::SoftReload, None),
    }
}

fn send_workspace_to_tag<C: Config<CMD>, SERVER: DisplayServer<CMD>, CMD>(
    manager: &mut Manager<C, CMD, SERVER>,
    ws_index: usize,
    tag_index: usize,
) -> bool {
    if ws_index < manager.state.workspaces.len() && tag_index < manager.tags.len() {
        let workspace = &manager.state.workspaces[ws_index].clone();
        manager.focus_workspace(workspace);
        manager.goto_tag_handler(tag_index + 1);
        return true;
    }
    false
}

fn send_window_to_tag<C: Config<CMD>, SERVER: DisplayServer<CMD>, CMD>(
    manager: &mut Manager<C, CMD, SERVER>,
    tag_index: usize,
) -> bool {
    if tag_index < manager.tags.len() {
        //tag number as 1 based.
        let tag_num = format!("{}", tag_index + 1);
        return manager.command_handler(&Command::MoveToTag, Some(&tag_num));
    }
    false
}
