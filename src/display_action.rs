use crate::models::WindowHandle;

//*********************************************************
// * these are responses from the the Window manager.
// * the display server should act on these actions
//*********************************************************

#[derive(Clone, Debug)]
pub enum DisplayAction {
    KillWindow(WindowHandle),      // nicely ask a window if it would please close at its convenience
    AddedWindow(WindowHandle),     // get triggered after a new window is discovered and WE are managing it
    DestroyedWindow(WindowHandle), // tell the DS we no longer care about the this window and other cleanup
    WindowTakeFocus(WindowHandle), // tell a window that it is to become focused
}
