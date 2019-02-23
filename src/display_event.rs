use super::{models::Screen, models::Window, models::WindowHandle, ModMask, XKeysym};

#[derive(Debug)]
pub enum DisplayEvent {
    KeyCombo(ModMask, XKeysym),
    WindowCreate(Window),
    WindowDestroy(WindowHandle),
    FocusedWindow(WindowHandle),
    ScreenCreate(Screen),
}
