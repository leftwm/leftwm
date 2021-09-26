use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScratchPad {
    pub name: String,
    pub value: String,
    // relative x of scratchpad, 25 means 25% of workspace x
    pub x: Option<i32>,
    // relative y of scratchpad, 25 means 25% of workspace y
    pub y: Option<i32>,
    // relative height of scratchpad, 50 means 50% of workspace height
    pub height: Option<i32>,
    // relative width of scratchpad, 50 means 50% of workspace width
    pub width: Option<i32>,
}
