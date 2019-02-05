use super::DisplayServer;
use super::Screen;
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

}

impl MockDisplayServer  {

}



