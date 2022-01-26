use crate::models::WindowHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    ReadyToResize(WindowHandle),
    ReadyToMove(WindowHandle),
    ResizingWindow(WindowHandle),
    MovingWindow(WindowHandle),
    Normal,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
