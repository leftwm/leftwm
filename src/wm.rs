
mod layouts;
mod utils;
mod display_servers;
mod manager;
mod config;
mod event_queue;
use display_servers::*;
use event_queue::*;
use manager::Manager;


fn main() {

    let ds:XlibDisplayServer = DisplayServer::new();
    let manager: Manager<XlibDisplayServer> = Manager::new();
    let event_queue :EventQueue = event_queue::new();

    //wireup the events 
    manager.ds.watch_events( event_queue.clone() ); 

    loop{
        if let Some(event) = event_queue.lock().unwrap().pop_front() {
            println!("event");
        }
    }

}

