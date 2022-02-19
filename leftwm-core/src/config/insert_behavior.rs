use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum InsertBehavior {
    Top,
    Bottom,
    BeforeCurrent,
    AfterCurrent,
}
