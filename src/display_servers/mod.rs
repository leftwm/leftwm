use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Manager;
use crate::models::Window;
use crate::models::Workspace;
use crate::DisplayEvent;
#[cfg(test)]
mod mock_display_server;
pub mod xlib_display_server;

#[cfg(test)]
pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer<CMD> {
    fn new(config: &impl Config<CMD>) -> Self;

    fn get_next_events(&mut self) -> Vec<DisplayEvent<CMD>>;

    fn update_theme_settings(&mut self, _config: &impl Config<CMD>) {}

    fn update_windows<C: Config<CMD>>(
        &self,
        _windows: Vec<&Window>,
        _focused: Option<&Window>,
        _manager: &Manager<C, CMD>,
    ) {
    }

    fn update_workspaces(&self, _windows: Vec<&Workspace>, _focused: Option<&Workspace>) {}

    fn execute_action(&mut self, _act: DisplayAction) -> Option<DisplayEvent<CMD>> {
        None
    }
}
