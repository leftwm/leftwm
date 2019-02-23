use super::*;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag => false,

        Command::GotoTag => {
            if let Some(tag) = val {
                if let Some(workspace) = manager.focused_workspace() {
                    workspace.show_tag(tag.clone() );
                    focus_handler::focus_tag( manager, &tag );
                    return true;
                }
            }
            false
        },

        Command::Execute => false,
        Command::CloseWindow => false,
        Command::SwapTags => false,
    }
}
