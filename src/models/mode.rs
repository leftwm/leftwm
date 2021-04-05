use crate::models::WindowHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Mode {
    ResizingWindow(WindowHandle),
    MovingWindow(WindowHandle),
    Normal,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
