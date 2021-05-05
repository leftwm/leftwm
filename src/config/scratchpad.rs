use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScratchPad {
    pub name: String,
    pub value: String,
}
