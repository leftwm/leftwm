use super::*;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag => {
            if let Some(tag) = val {
                if let Some(window) = manager.focused_window() {
                    window.clear_tags();
                    window.tag(tag);
                    return true;
                }
            }
            false
        }

        Command::GotoTag => {
            if let Some(tag) = val {
                if !manager.tags.contains(&tag) {
                    manager.tags.push(tag.clone());
                }
                if let Some(workspace) = manager.focused_workspace() {
                    workspace.show_tag(tag.clone());
                    focus_handler::focus_tag(manager, &tag);
                    return true;
                }
            }
            true
        }

        Command::Execute => false,
        Command::CloseWindow => false,
        Command::SwapTags => false,
    }
}
