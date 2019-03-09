use super::{models::Screen, models::Window, models::WindowHandle, ModMask, XKeysym, Button};

#[derive(Debug)]
pub enum DisplayEvent {
    Movement(WindowHandle, i32, i32),
    KeyCombo(ModMask, XKeysym),
    MouseCombo(ModMask, Button, WindowHandle),
    WindowCreate(Window),
    WindowDestroy(WindowHandle),
    FocusedWindow(WindowHandle, i32, i32),
    MoveWindow(WindowHandle, i32, i32),
    ResizeWindow(WindowHandle, i32, i32),
    ScreenCreate(Screen),
    ChangeToNormalMode,
}
