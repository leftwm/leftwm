use super::Config;
use super::DisplayEvent;
use super::DisplayServer;
use super::Screen;
use super::Window;
use crate::display_action::DisplayAction;

#[derive(Clone)]
pub struct MockDisplayServer {
    pub screens: Vec<Screen>,
}

impl DisplayServer for MockDisplayServer {
    fn new(_: &Config) -> MockDisplayServer {
        MockDisplayServer { screens: vec![] }
    }

    //testing a couple mock event
    fn get_next_events(&self) -> Vec<DisplayEvent> {
        vec![]
    }

    fn update_windows(&self, _: Vec<&Window>) {}

    fn execute_action(&mut self, _: DisplayAction) -> Option<DisplayEvent> {
        None
    }
}
