use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum InsertBehavior {
    Top,
    Bottom,
    BeforeCurrent,
    AfterCurrent
}
