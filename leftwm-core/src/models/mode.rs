use std::fmt::Debug;

use crate::models::WindowHandle;
use serde::{Deserialize, Serialize};

use super::window::Handle;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode<H: Handle> {
    #[serde(bound = "")]
    ReadyToResize(WindowHandle<H>),
    #[serde(bound = "")]
    ReadyToMove(WindowHandle<H>),
    #[serde(bound = "")]
    ResizingWindow(WindowHandle<H>),
    #[serde(bound = "")]
    MovingWindow(WindowHandle<H>),
    Normal,
}

impl<H: Handle> Default for Mode<H> {
    fn default() -> Self {
        Self::Normal
    }
}
