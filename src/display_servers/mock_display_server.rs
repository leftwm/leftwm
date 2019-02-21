use super::DisplayEvent;
use super::DisplayServer;
use super::Screen;
use super::Config;
use super::Window;

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
}

impl MockDisplayServer {
    pub fn create_fake_screens(&self, count: i32) -> Vec<Screen> {
        let mut screens = vec![];
        for _ in 0..count {
            screens.push(Screen::default());
        }
        screens
    }
}
