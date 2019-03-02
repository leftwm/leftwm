use crate::models::WindowHandle;

//*********************************************************
// * these are responses from the the Window manager.
// * the display server should act on these actions
//*********************************************************

#[derive(Clone, Debug)]
pub enum DisplayAction {
    KillWindow(WindowHandle),
}
