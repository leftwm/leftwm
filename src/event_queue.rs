use super::utils::window::Window;
use super::utils::screen::Screen;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;


pub enum EventQueueItem {
    NewWindow(Window),
    NewScreen(Screen)
}


pub type EventQueue = Arc<Mutex<VecDeque<EventQueueItem>>>;

pub fn new() -> EventQueue {
    let q:VecDeque<EventQueueItem> = VecDeque::new();
    Arc::new(Mutex::new(q))
}


