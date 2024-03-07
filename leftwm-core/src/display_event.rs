use super::{models::Screen, models::Window, models::WindowHandle, Button, ModMask};
use crate::models::WindowChange;
use crate::Command;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum DisplayEvent {
    Movement(WindowHandle, i32, i32),
    MouseCombo(ModMask, Button, WindowHandle, i32, i32),
    WindowCreate(Window, i32, i32),
    WindowChange(WindowChange),
    WindowDestroy(WindowHandle),
    WindowTakeFocus(WindowHandle),
    HandleWindowFocus(WindowHandle),
    VerifyFocusedAt(WindowHandle), // Request focus validation for this window.
    MoveFocusTo(i32, i32),         // Focus the nearest window to this point.
    MoveWindow(WindowHandle, i32, i32),
    ResizeWindow(WindowHandle, i32, i32),
    ScreenCreate(Screen),
    ScreenUpdate(Screen),
    ScreenDelete(String),
    SendCommand(Command),
    ConfigureXlibWindow(WindowHandle),
    ChangeToNormalMode,
}
