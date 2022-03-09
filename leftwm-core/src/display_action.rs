use crate::config::Keybind;
use crate::models::TagId;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::WindowState;
use serde::{Deserialize, Serialize};

/// These are responses from the Window manager.
/// The display server should act on these actions.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DisplayAction {
    /// Nicely ask a window if it would please close at its convenience.
    KillWindow(WindowHandle),

    /// Get triggered after a new window is discovered and WE are
    /// managing it.
    AddedWindow(WindowHandle, bool, bool),

    /// Makes sure the mouse is over a given window.
    MoveMouseOver(WindowHandle, bool),

    /// Makes sure the mouse is over a given point.
    MoveMouseOverPoint((i32, i32)),

    /// Change a windows state.
    SetState(WindowHandle, bool, WindowState),

    /// Sets the "z-index" order of the windows
    /// first in the array is top most
    SetWindowOrder(Vec<Window>),

    /// Raises a given window.
    MoveToTop(WindowHandle),

    /// Tell the DS we no longer care about the this window and other
    /// cleanup.
    DestroyedWindow(WindowHandle),

    /// Tell a window that it is to become focused.
    WindowTakeFocus {
        window: Window,
        previous_window: Option<Window>,
    },

    /// Remove focus on any visible window by focusing the root window.
    Unfocus(Option<WindowHandle>, bool),

    /// To the window under the cursor to take the focus.
    FocusWindowUnderCursor,

    /// Tell the DM we are ready to resize this window.
    ReadyToResizeWindow(WindowHandle),

    /// Tell the DM we are ready to move this window.
    ReadyToMoveWindow(WindowHandle),

    /// Used to let the WM know of the current displayed tag changes.
    SetCurrentTags(Vec<TagId>),

    /// Used to let the WM know of the tag for a given window.
    SetWindowTags(WindowHandle, Vec<TagId>),

    /// Tell the DM to return to normal mode if it is not (ie resize a
    /// window or moving a window).
    NormalMode,

    /// SoftReload keygrabs, needed when keyboard changes.
    ReloadKeyGrabs(Vec<Keybind>),

    /// Configure a xlib window.
    ConfigureXlibWindow(Window),
}
