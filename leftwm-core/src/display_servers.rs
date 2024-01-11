#[cfg(test)]
mod mock_display_server;

use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::DisplayEvent;

use futures::prelude::*;
use std::pin::Pin;

#[cfg(test)]
pub use self::mock_display_server::MockDisplayServer;

pub trait DisplayServer {
    fn new(config: &impl Config) -> Self;

    fn get_next_events(&mut self) -> Vec<DisplayEvent>;

    fn reload_config(
        &mut self,
        config: &impl Config,
        focused: Option<WindowHandle>,
        windows: &[Window],
    );

    fn update_windows(&self, _windows: Vec<&Window>) {}

    fn update_workspaces(&self, _focused: Option<&Workspace>) {}

    fn execute_action(&mut self, _act: DisplayAction) -> Option<DisplayEvent> {
        None
    }

    fn wait_readable(&self) -> Pin<Box<dyn Future<Output = ()>>>;

    fn flush(&self);

    fn generate_verify_focus_event(&self) -> Option<DisplayEvent>;
}
