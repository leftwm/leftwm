use super::{models::Screen, models::Window, models::WindowHandle, Button, ModMask, XKeysym};
use crate::models::WindowChange;
use crate::Command;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DisplayEvent {
    Movement(WindowHandle, i32, i32),
    KeyCombo(ModMask, XKeysym),
    MouseCombo(ModMask, Button, WindowHandle),
    WindowCreate(Window),
    WindowChange(WindowChange),
    WindowDestroy(WindowHandle),
    FocusedWindow(WindowHandle, i32, i32),
    MoveWindow(WindowHandle, u64, i32, i32),
    ResizeWindow(WindowHandle, u64, i32, i32),
    ScreenCreate(Screen),
    SendCommand(Command, Option<String>),
    ChangeToNormalMode,
}
