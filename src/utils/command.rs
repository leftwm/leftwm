use super::config::Config;
use x11_dl::xlib::XKeyEvent;
//use super::Manager;
//use super::DisplayServer;

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    OpenTerminal,
    CloseWindow,
    SwapWorkspaces,
    GotoWorkspace,
    MovetoWorkspace,
}

pub struct CommandBuilder {
    config: Config,
}

impl CommandBuilder {
    pub fn new(config: Config) -> CommandBuilder {
        CommandBuilder { config }
    }
    pub fn try_from_xkeyevent(&self, event: XKeyEvent) -> Option<Command> {
        None
    }
}

#[test]
fn should_be_able_to_build_a_goto_workspace_command() {
    let builder = CommandBuilder::new(Config::default());
    let ev = XKeyEvent{};
    builder.from_xkeyevent(ev);
}
