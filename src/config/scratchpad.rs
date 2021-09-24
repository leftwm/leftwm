use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScratchPad {
    pub name: String,
    pub value: String,
    pub width: i32,
    pub height: i32,
    pub x: i32,
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
