use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Screen;
use crate::models::Window;
use crate::DisplayEvent;
mod display_server_mode;
mod mock_display_server;
pub mod xlib_display_server;

pub use self::display_server_mode::DisplayServerMode;
pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer {
    fn new(config: &Config) -> Self;
    fn get_next_events(&self) -> Vec<DisplayEvent>;
    fn update_windows(&self, windows: Vec<&Window>);
    fn execute_action(&mut self, act: DisplayAction) -> Option<DisplayEvent>;
}
