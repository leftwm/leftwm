use leftwm_core::{DisplayAction, Window};

use crate::leftwm_config::LeftwmConfig;

#[derive(Debug)]
pub enum InternalAction {
    Flush,
    GenerateVerifyFocusEvent,
    UpdateConfig(LeftwmConfig),
    UpdateWindows(Vec<Window>),
    DisplayAction(DisplayAction),
}
