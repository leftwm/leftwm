use super::{models::Screen, models::Window, models::WindowHandle, Button, ModMask};
use crate::models::{WindowChange, Handle};
use crate::Command;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum DisplayEvent<H: Handle> {
    Movement(WindowHandle<H>, i32, i32),
    MouseCombo(ModMask, Button, WindowHandle<H>, i32, i32),
    WindowCreate(Window<H>, i32, i32),
    WindowChange(WindowChange<H>),
    WindowDestroy(WindowHandle<H>),
    WindowTakeFocus(WindowHandle<H>),
    HandleWindowFocus(WindowHandle<H>),
    VerifyFocusedAt(WindowHandle<H>), // Request focus validation for this window.
    MoveFocusTo(i32, i32),         // Focus the nearest window to this point.
    MoveWindow(WindowHandle<H>, i32, i32),
    ResizeWindow(WindowHandle<H>, i32, i32),
    ScreenCreate(Screen<H>),
    SendCommand(Command<H>),
    ConfigureXlibWindow(WindowHandle<H>), // TODO: check if this has backend specific code
    ChangeToNormalMode,
}
