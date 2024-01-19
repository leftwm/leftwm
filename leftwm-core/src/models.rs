//! Objects (such as windows) used to develop `LeftWM`.
mod dock_area;
mod focus_manager;
mod gutter;
mod manager;
mod margins;
mod mode;
mod scratchpad;
mod screen;
mod size;
mod tag;
mod window;
mod window_change;
mod window_state;
mod window_type;
mod workspace;
mod xyhw;
mod xyhw_change;

pub mod dto;

pub use dock_area::DockArea;
pub use focus_manager::FocusBehaviour;
pub use focus_manager::FocusManager;
pub use gutter::Gutter;
pub use gutter::Side;
pub use manager::Manager;
pub use margins::Margins;
pub use mode::Mode;
pub use scratchpad::{ScratchPad, ScratchPadName};
pub use screen::{BBox, Screen};
pub use size::Size;
pub use window::Window;
pub use window::WindowHandle;
pub use window::Handle;
pub(crate) use window::MockHandle;
pub use window_change::WindowChange;
pub use window_state::WindowState;
pub use window_type::WindowType;
pub use workspace::Workspace;
pub use xyhw::Xyhw;
pub use xyhw::XyhwBuilder;
pub use xyhw_change::XyhwChange;

pub use tag::Tag;
pub use tag::Tags;

pub type TagId = usize;
pub type WorkspaceId = usize;
type MaybeWindowHandle<H> = Option<WindowHandle<H>>;
