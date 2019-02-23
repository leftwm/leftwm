mod command;
mod command_builder;
mod config;
mod display_event;
mod display_servers;
mod handlers;
mod layouts;
mod models;
mod utils;

use utils::xkeysym_lookup::ModMask;
use utils::xkeysym_lookup::XKeysym;

pub use command::Command;
pub use display_event::DisplayEvent;
pub use display_servers::DisplayServer;
pub use display_servers::XlibDisplayServer;
pub use handlers::display_event_handler;
pub use models::Manager;
pub use models::Window;

#[macro_use]
extern crate serde_derive;
extern crate serde;

//#[cfg(test)]
//Mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}
