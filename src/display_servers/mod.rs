use crate::config::{Config, ThemeSetting};
use crate::display_action::DisplayAction;
use crate::models::Manager;
use crate::models::Screen;
use crate::models::Window;
use crate::models::Workspace;
use crate::DisplayEvent;
#[cfg(test)]
mod mock_display_server;
pub mod xlib_display_server;

#[cfg(test)]
pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer<C: Config> {
    fn new(config: C) -> Self;

    fn get_next_events(&mut self) -> Vec<DisplayEvent>;

    fn update_theme_settings(&mut self, _settings: ThemeSetting) {}

    fn update_windows(
        &self,
        _windows: Vec<&Window>,
        _focused: Option<&Window>,
        _manager: &Manager,
    ) {
    }

    fn update_workspaces(&self, _windows: Vec<&Workspace>, _focused: Option<&Workspace>) {}

    fn execute_action(&mut self, _act: DisplayAction) -> Option<DisplayEvent> {
        None
    }
}
