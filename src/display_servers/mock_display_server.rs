use super::Config;
use super::DisplayEvent;
use super::DisplayServer;
use super::ThemeSetting;
use crate::models::Screen;
use std::sync::Arc;

#[derive(Clone)]
pub struct MockDisplayServer {
    pub screens: Vec<Screen>,
}

impl<C: Config, SERVER: DisplayServer> DisplayServer<C, SERVER> for MockDisplayServer {
    fn new(_: C, _: Arc<ThemeSetting>) -> Self {
        Self { screens: vec![] }
    }

    //testing a couple mock event
    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        vec![]
    }
}
