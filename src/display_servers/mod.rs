use super::config::Config;
use super::models::Window;
use super::models::WindowHandle;
use super::models::Screen;
use super::utils;
use super::DisplayEvent;
mod mock_display_server;
mod xlib_display_server;

pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer {
    fn new(config: &Config) -> Self;
    fn get_next_events(&self) -> Vec<DisplayEvent>;
    fn update_windows(&self, windows: Vec<&Window>);
}
