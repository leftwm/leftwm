use super::Config;
use super::DisplayEvent;
use super::DisplayServer;
use crate::models::Handle;
use crate::models::Screen;

#[derive(Clone)]
pub struct MockDisplayServer<H: Handle> {
    pub screens: Vec<Screen<H>>,
}

impl<H: Handle> DisplayServer<H> for MockDisplayServer<H> {
    fn new(_: &impl Config) -> Self {
        Self { screens: vec![] }
    }

    // testing a couple mock event
    fn get_next_events(&mut self) -> Vec<DisplayEvent<H>> {
        vec![]
    }

    fn wait_readable(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
        unimplemented!()
    }

    fn flush(&self) {
        unimplemented!()
    }

    fn generate_verify_focus_event(&self) -> Option<DisplayEvent<H>> {
        unimplemented!()
    }
}
