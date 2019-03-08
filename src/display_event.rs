use super::{models::Screen, models::Window, models::WindowHandle, ModMask, XKeysym};

#[derive(Debug)]
pub enum DisplayEvent {
    Movement(WindowHandle, i32, i32),
    KeyCombo(ModMask, XKeysym),
    WindowCreate(Window),
    WindowDestroy(WindowHandle),
    FocusedWindow(WindowHandle, i32, i32),
    ScreenCreate(Screen),
}
