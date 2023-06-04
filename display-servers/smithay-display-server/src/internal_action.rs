use leftwm_core::DisplayAction;

#[derive(Debug)]
pub enum InternalAction {
    Flush,
    GenerateVerifyFocusEvent,
    DisplayAction(DisplayAction),
}
