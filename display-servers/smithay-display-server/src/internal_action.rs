use leftwm_core::{DisplayAction, Window};

#[derive(Debug)]
pub enum InternalAction {
    Flush,
    GenerateVerifyFocusEvent,
    UpdateWindows(Vec<Window>),
    DisplayAction(DisplayAction),
}
