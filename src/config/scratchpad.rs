use crate::models::Size;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScratchPad {
    pub name: String,
    pub value: String,
    // relative x of scratchpad, 25 means 25% of workspace x
    pub x: Option<Size>,
    // relative y of scratchpad, 25 means 25% of workspace y
    pub y: Option<Size>,
    // relative height of scratchpad, 50 means 50% of workspace height
    pub height: Option<Size>,
    // relative width of scratchpad, 50 means 50% of workspace width
    pub width: Option<Size>,
}
