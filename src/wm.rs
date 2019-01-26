
mod utils;
mod display_servers;
mod manager;
use display_servers::*;


fn main() {

    let mut ds:XlibDisplayServer = DisplayServer::new();
    ds.start_event_loop();

}

