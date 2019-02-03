
mod layouts;
mod utils;
mod display_servers;
mod manager;
mod config;
use display_servers::*;
use manager::Manager;
use std::sync::{Arc, Mutex};


fn main() {

    let ds:XlibDisplayServer = DisplayServer::new();
    let m_raw: Manager<XlibDisplayServer> = Manager::new();
    let manager = Arc::new(Mutex::new(m_raw));

    //wireup the events 
    let man_for_events = manager.clone();
    ds.watch_events(man_for_events);

    //pass ownership of the display server, and run the config
    let mut man2 = manager.clone();
    {
        let m = Arc::get_mut(&mut man2).unwrap();
        let mut man = m.lock().unwrap();
        man.ds=ds;
        config::load_config( &mut man );
    }



}

