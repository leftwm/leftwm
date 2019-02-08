use super::event_queue::EventQueueItem;
use super::utils;
use super::DisplayServer;
use super::Screen;

#[derive(Clone)]
pub struct MockDisplayServer {
    pub screens: Vec<Screen>,
}

impl DisplayServer for MockDisplayServer {
    fn new() -> MockDisplayServer {
        MockDisplayServer { screens: vec![] }
    }

    //testing a couple mock event
    fn get_next_events(&self) -> Vec<EventQueueItem> {
        vec![]
    }

    fn update_windows(&self, _: Vec<&utils::window::Window>) {}
}

impl MockDisplayServer {}
