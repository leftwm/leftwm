#[cfg_attr(
    feature = "clippy",
    allow(clippy::cast_ptr_alignment, clippy::cast_lossless)
)]
mod command;
mod command_builder;
pub mod config;
mod display_action;
mod display_event;
pub mod display_servers;
pub mod errors;
mod handlers;
mod layouts;
pub mod models;
pub mod state;
pub mod utils;

use utils::xkeysym_lookup::Button;
use utils::xkeysym_lookup::ModMask;
use utils::xkeysym_lookup::XKeysym;

pub use command::Command;
pub use display_event::DisplayEvent;
pub use display_servers::xlib_display_server::XWrap;
pub use display_servers::DisplayServer;
pub use display_servers::XlibDisplayServer;
pub use handlers::display_event_handler::DisplayEventHandler;
pub use handlers::external_command_handler;
pub use models::Manager;
pub use models::Mode;
pub use models::Window;
pub use models::Workspace;
pub use utils::child_process;
pub use utils::command_pipe::CommandPipe;
pub use utils::state_socket::StateSocket;
