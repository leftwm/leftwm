use crate::models::WindowHandle;

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayServerMode {
    ResizingWindow(WindowHandle),
    MovingWindow(WindowHandle),
    NormalMode,
}
