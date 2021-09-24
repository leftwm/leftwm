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
