use super::utils::screen::Screen;
use super::utils::window;

pub enum EventQueueItem {
    WindowCreate(window::Window),
    WindowDelete(window::WindowHandle),
    ScreenCreate(Screen),
}
