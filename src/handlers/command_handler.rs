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
                goto_tag_handler::process(manager, tag)
            } else {
                false
            }
        }

        Command::Execute => false,
        Command::CloseWindow => false,
        Command::SwapTags => false,
    }
}
