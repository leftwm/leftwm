use crate::models::Window;
use crate::models::WindowHandle;

//*********************************************************
// * these are responses from the Window manager.
// * the display server should act on these actions
//*********************************************************

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DisplayAction {
    KillWindow(WindowHandle), // nicely ask a window if it would please close at its convenience
    AddedWindow(WindowHandle), // get triggered after a new window is discovered and WE are managing it
    MoveMouseOver(WindowHandle), // makes sure the mouse is over a given window
    MoveMouseOverPoint((i32, i32)), // makes sure the mouse is over a given point
    MoveToTop(WindowHandle),   // send a window to the top move location
    DestroyedWindow(WindowHandle), // tell the DS we no longer care about the this window and other cleanup
    WindowTakeFocus(Window),       // tell a window that it is to become focused
    StartResizingWindow(WindowHandle), // tell the DM we are going to resize a window and only send that type of events
    StartMovingWindow(WindowHandle), // tell the DM we are going to move a window and only send that type of events
    SetCurrentTags(String),          // Used to let the WM know of the current displayed tag changes
    SetWindowTags(WindowHandle, String), // Used to let the WM know of the tag for a given window
    NormalMode, // tell the DM to return to normal mode if it is not (ie resize a window or moving a window)
}
