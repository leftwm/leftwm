mod config;
mod display_servers;
mod event_queue;
mod layouts;
mod manager;
mod utils;
use display_servers::*;
use event_queue::*;
use manager::Manager;
use std::{thread, time};

fn main() {
    let mut manager: Manager<XlibDisplayServer> = Manager::new();
    let event_queue: EventQueue = event_queue::new();

    //wireup the events
    manager.ds.watch_events(event_queue.clone());

    loop {
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        let mut q = event_queue.lock().unwrap();
        while q.len() > 0 {
            if let Some(event) = q.pop_front() {
                manager.on_event(event)
            }
        }
    }
}
