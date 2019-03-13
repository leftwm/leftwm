use super::{models::Screen, models::Window, models::WindowHandle, Button, ModMask, XKeysym};
use crate::models::WindowChange;
use crate::Command;

#[derive(Debug)]
pub enum DisplayEvent {
    Movement(WindowHandle, i32, i32),
    KeyCombo(ModMask, XKeysym),
    MouseCombo(ModMask, Button, WindowHandle),
    WindowCreate(Window),
    WindowChange(WindowChange),
    WindowDestroy(WindowHandle),
    FocusedWindow(WindowHandle, i32, i32),
    MoveWindow(WindowHandle, i32, i32),
    ResizeWindow(WindowHandle, i32, i32),
    ScreenCreate(Screen),
    SendCommand(Command, Option<String>),
    ChangeToNormalMode,
}
