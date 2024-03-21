use crate::models::Handle;
use crate::models::TagId;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::WindowState;
use crate::utils::modmask_lookup::Button;
use serde::{Deserialize, Serialize};

/// These are responses from the Window manager.
/// The display server should act on these actions.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DisplayAction<H: Handle> {
    /// Nicely ask a window if it would please close at its convenience.
    #[serde(bound = "")]
    KillWindow(WindowHandle<H>),

    /// Get triggered after a new window is discovered and WE are
    /// managing it.
    #[serde(bound = "")]
    AddedWindow(WindowHandle<H>, bool, bool),

    /// Makes sure the mouse is over a given window.
    #[serde(bound = "")]
    MoveMouseOver(WindowHandle<H>, bool),

    /// Makes sure the mouse is over a given point.
    MoveMouseOverPoint((i32, i32)),

    /// Change a windows state.
    #[serde(bound = "")]
    SetState(WindowHandle<H>, bool, WindowState),

    /// Sets the "z-index" order of the windows
    /// first in the array is top most
    #[serde(bound = "")]
    SetWindowOrder(Vec<WindowHandle<H>>),

    /// Raises a given window.
    #[serde(bound = "")]
    MoveToTop(WindowHandle<H>),

    /// Tell the DS we no longer care about the this window and other
    /// cleanup.
    #[serde(bound = "")]
    DestroyedWindow(WindowHandle<H>),

    /// Tell a window that it is to become focused.
    #[serde(bound = "")]
    WindowTakeFocus {
        window: Window<H>,
        previous_window: Option<Window<H>>,
    },

    /// Remove focus on any visible window by focusing the root window.
    #[serde(bound = "")]
    Unfocus(Option<WindowHandle<H>>, bool),

    /// To the window under the cursor to take the focus.
    FocusWindowUnderCursor,

    #[serde(bound = "")]
    ReplayClick(WindowHandle<H>, Button),

    /// Tell the DM we are ready to resize this window.
    #[serde(bound = "")]
    ReadyToResizeWindow(WindowHandle<H>),

    /// Tell the DM we are ready to move this window.
    #[serde(bound = "")]
    ReadyToMoveWindow(WindowHandle<H>),

    /// Used to let the WM know of the current displayed tag changes.
    SetCurrentTags(Option<TagId>),

    /// Used to let the WM know of the tag for a given window.
    #[serde(bound = "")]
    SetWindowTag(WindowHandle<H>, Option<TagId>),

    /// Tell the DM to return to normal mode if it is not (ie resize a
    /// window or moving a window).
    NormalMode,

    /// Configure a xlib window.
    #[serde(bound = "")]
    ConfigureXlibWindow(Window<H>),
}
