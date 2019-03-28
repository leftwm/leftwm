use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Screen;
use crate::models::Window;
use crate::models::Workspace;
use crate::DisplayEvent;
mod mock_display_server;
pub mod xlib_display_server;

pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer {
    fn new(config: &Config) -> Self;
    fn get_next_events(&self) -> Vec<DisplayEvent>;
    fn update_windows(&self, windows: Vec<&Window>, focused: Option<&Window>);
    fn update_workspaces(&self, windows: Vec<&Workspace>, focused: Option<&Workspace>);
    fn execute_action(&mut self, act: DisplayAction) -> Option<DisplayEvent>;
}
