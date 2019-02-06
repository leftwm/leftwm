use super::DisplayServer;
use super::Screen;
use super::utils;
use super::event_queue::EventQueue;

#[derive(Clone)]
pub struct MockDisplayServer{
    pub screens: Vec<Screen>
}

impl DisplayServer for MockDisplayServer  {

    fn new() -> MockDisplayServer {
        MockDisplayServer{
            screens: vec![]
        }
    }

    //testing a couple mock event
    fn watch_events(&self, _: EventQueue){
    }

    fn update_windows(&self, windows: Vec<&utils::window::Window> ){
    }

}

impl MockDisplayServer  {

}



