//! Objects (such as windows) used to develop `LeftWM`.
mod dock_area;
mod focus_manager;
mod gutter;
mod manager;
mod margins;
mod mode;
mod screen;
mod tag;
mod window;
mod window_change;
mod window_state;
mod window_type;
mod workspace;
mod xyhw;
mod xyhw_change;

pub mod dto;
use crate::layouts;

pub use dock_area::DockArea;
pub use focus_manager::FocusBehaviour;
pub use focus_manager::FocusManager;
pub use gutter::Gutter;
pub use gutter::Side;
pub use manager::Manager;
pub use margins::Margins;
pub use mode::Mode;
pub use screen::{BBox, Screen};
pub use window::Window;
pub use window::WindowHandle;
pub use window_change::WindowChange;
pub use window_state::WindowState;
pub use window_type::WindowType;
pub use workspace::Workspace;
pub use xyhw::Xyhw;
pub use xyhw::XyhwBuilder;
pub use xyhw_change::XyhwChange;

pub use tag::Tag;

pub type TagId = String;
type MaybeWindowHandle = Option<WindowHandle>;
