//! Various leftwm features.
// We deny clippy pedantic lints, primarily to keep code as correct as possible
// Remember, the goal of LeftWM is to do one thing and to do that one thing
// well: Be a window manager.
#![warn(clippy::pedantic)]
// Each of these lints are globally allowed because they otherwise make a lot
// of noise. However, work to ensure that each use of one of these is correct
// would be very much appreciated.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::default_trait_access
)]
mod command;
mod command_builder;
pub mod config;
mod display_action;
mod display_event;
pub mod display_servers;
pub mod errors;
mod event_loop;
mod handlers;
pub mod layouts;
pub mod models;
pub mod utils;

use utils::xkeysym_lookup::Button;
use utils::xkeysym_lookup::ModMask;
use utils::xkeysym_lookup::XKeysym;

pub use command::Command;
pub use config::Config;
pub use display_action::DisplayAction;
pub use display_event::DisplayEvent;
pub use display_servers::xlib_display_server::XWrap;
pub use display_servers::DisplayServer;
pub use display_servers::XlibDisplayServer;
pub use handlers::external_command_handler;
pub use models::Manager;
pub use models::Mode;
pub use models::Window;
pub use models::Workspace;
pub use utils::child_process;
pub use utils::command_pipe::CommandPipe;
pub use utils::state_socket::StateSocket;
