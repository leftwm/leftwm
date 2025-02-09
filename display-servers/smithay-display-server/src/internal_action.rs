use leftwm_core::{DisplayAction, Window};

use crate::{leftwm_config::LeftwmConfig, SmithayWindowHandle};

#[derive(Debug)]
pub enum InternalAction {
    Flush,
    GenerateVerifyFocusEvent,
    UpdateConfig(LeftwmConfig),
    UpdateWindows(Vec<Window<SmithayWindowHandle>>),
    DisplayAction(DisplayAction<SmithayWindowHandle>),
}
