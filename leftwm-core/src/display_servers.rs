#[cfg(test)]
mod mock_display_server;

use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Handle;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::DisplayEvent;

use futures::prelude::*;
use std::pin::Pin;

#[cfg(test)]
pub use self::mock_display_server::MockDisplayServer;

pub trait DisplayServer<H: Handle> {
    fn new(config: &impl Config) -> Self;

    fn get_next_events(&mut self) -> Vec<DisplayEvent<H>>;

    fn load_config(
        &mut self,
        _config: &impl Config,
        _focused: Option<&Option<WindowHandle<H>>>,
        _windows: &[Window<H>],
    ) {
    }

    fn update_windows(&self, _windows: Vec<&Window<H>>) {}

    fn update_workspaces(&self, _focused: Option<&Workspace>) {}

    fn execute_action(&mut self, _act: DisplayAction<H>) -> Option<DisplayEvent<H>> {
        None
    }

    fn wait_readable(&self) -> Pin<Box<dyn Future<Output = ()>>>;

    fn flush(&self);

    fn generate_verify_focus_event(&self) -> Option<DisplayEvent<H>>;
}
