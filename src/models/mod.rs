mod dock_area;
mod manager;
mod mode;
mod screen;
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
pub use manager::Manager;
pub use mode::Mode;
pub use screen::{BBox, Screen};
pub use window::Window;
pub use window::WindowHandle;
pub use window_change::WindowChange;
pub use window_state::WindowState;
pub use window_type::WindowType;
pub use workspace::Workspace;
pub use xyhw::XYHWBuilder;
pub use xyhw::XYHW;
pub use xyhw_change::XYHWChange;

pub type Tag = String;
