use super::Config;
use super::DisplayEvent;
use super::DisplayServer;
use super::Screen;

#[derive(Clone)]
pub struct MockDisplayServer {
    pub screens: Vec<Screen>,
}

impl<C: Config> DisplayServer<C> for MockDisplayServer {
    fn new(_: C) -> MockDisplayServer {
        MockDisplayServer { screens: vec![] }
    }

    //testing a couple mock event
    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        vec![]
    }
}
