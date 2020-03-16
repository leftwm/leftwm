use crate::models::WindowHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Mode {
    ResizingWindow(WindowHandle),
    MovingWindow(WindowHandle),
    NormalMode,
}

impl Default for Mode {
    fn default() -> Self {
        Self::NormalMode
    }
}
