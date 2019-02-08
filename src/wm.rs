mod config;
mod display_servers;
mod event_queue;
mod layouts;
mod manager;
mod utils;
use display_servers::*;
use manager::Manager;

fn main() {
    let mut manager: Manager<XlibDisplayServer> = Manager::new();
    config::load_config(&mut manager);

    //main event loop
    loop {
        let events = manager.ds.get_next_events();
        for event in events {
            manager.on_event(event)
        }
    }
}
