use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertBehavior {
    Top,
    Bottom,
    BeforeCurrent,
    AfterCurrent,
}

impl Default for InsertBehavior {
    fn default() -> Self {
        InsertBehavior::Bottom
    }
}
