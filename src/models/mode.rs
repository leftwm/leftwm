use crate::models::WindowHandle;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Mode {
    ResizingWindow(WindowHandle),
    MovingWindow(WindowHandle),
    NormalMode,
}
