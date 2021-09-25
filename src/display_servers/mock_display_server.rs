use super::Config;
use super::DisplayEvent;
use super::DisplayServer;
use crate::models::Screen;
use std::sync::Arc;

#[derive(Clone)]
pub struct MockDisplayServer {
    pub screens: Vec<Screen>,
}

impl DisplayServer for MockDisplayServer {
    fn new(_: &impl Config) -> Self {
        Self { screens: vec![] }
    }

    //testing a couple mock event
    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        vec![]
    }

    fn wait_readable(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn verify_focused_window(&self) -> Vec<DisplayEvent> {
        todo!()
    }
}
