use crate::models::Window;
use crate::models::WindowHandle;

//*********************************************************
// * these are responses from the Window manager.
// * the display server should act on these actions
//*********************************************************

#[derive(Clone, Debug)]
pub enum DisplayAction {
    KillWindow(WindowHandle), // nicely ask a window if it would please close at its convenience
    AddedWindow(WindowHandle), // get triggered after a new window is discovered and WE are managing it
    MoveToTop(WindowHandle),   // send a window to the top move location
    DestroyedWindow(WindowHandle), // tell the DS we no longer care about the this window and other cleanup
    WindowTakeFocus(Window),       // tell a window that it is to become focused
    StartResizingWindow(WindowHandle), // tell the DM we are going to resize a window and only send that type of events
    StartMovingWindow(WindowHandle), // tell the DM we are going to move a window and only send that type of events
    SetCurrentTags(String),          // Used to let the WM know of the current tag changes
    NormalMode, // tell the DM to return to normal mode if it is not (ie resize a window or moving a window)
}
