use crate::config::Keybind;
use crate::models::Window;
use crate::models::WindowHandle;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// These are responses from the Window manager.
/// The display server should act on these actions.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Derivative)]
#[derivative(Debug)]
pub enum DisplayAction<CMD> {
    /// Nicely ask a window if it would please close at its convenience.
    KillWindow(WindowHandle),

    /// Get triggered after a new window is discovered and WE are
    /// managing it.
    AddedWindow(WindowHandle, bool),

    /// Makes sure the mouse is over a given window.
    MoveMouseOver(WindowHandle),

    /// Makes sure the mouse is over a given point.
    MoveMouseOverPoint((i32, i32)),

    /// Change a windows fullscreen state
    SetFullScreen(Window, bool),

    /// Sets the "z-index" order of the windows
    /// first in the array is top most
    SetWindowOrder(Vec<WindowHandle>),

    /// Tell the DS we no longer care about the this window and other
    /// cleanup.
    DestroyedWindow(WindowHandle),

    /// Tell a window that it is to become focused.
    WindowTakeFocus(Window),

    /// Remove focus on any visible window by focusing the root window
    Unfocus,

    /// To the window under the cursor to take the focus
    FocusWindowUnderCursor,

    /// Tell the DM we are going to resize a window and only send that
    /// type of events.
    StartResizingWindow(WindowHandle),

    /// Tell the DM we are going to move a window and only send that
    /// type of events.
    StartMovingWindow(WindowHandle),

    /// Used to let the WM know of the current displayed tag changes.
    SetCurrentTags(String),

    /// Used to let the WM know of the tag for a given window.
    SetWindowTags(WindowHandle, String),

    /// Tell the DM to return to normal mode if it is not (ie resize a
    /// window or moving a window).
    NormalMode,

    /// Reload keygrabs, needed when keyboard changes
    ReloadKeyGrabs(#[derivative(Debug = "ignore")] Vec<Keybind<CMD>>),
}
