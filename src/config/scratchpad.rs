use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScratchPad {
    pub name: String,
    pub value: String,
    // relative width of scratchpad, 50 means 50% of workspace width
    #[serde(default)]
    pub width: i32,
    // relative height of scratchpad, 50 means 50% of workspace height
    #[serde(default)]
    pub height: i32,
    // relative x of scratchpad, 25 means 25% of workspace x
    #[serde(default)]
    pub x: i32,
    // relative y of scratchpad, 25 means 25% of workspace y
    #[serde(default)]
    pub y: i32,
}

impl Default for ScratchPad {
    fn default() -> Self {
        Self {
            name: "st".to_string(),
            value: "st".to_string(),
            width: 50,
            height: 50,
            x: 25,
            y: 25,
        }
    }
}
