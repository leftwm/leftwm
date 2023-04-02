use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InsertBehavior {
    Top,
    #[default]
    Bottom,
    BeforeCurrent,
    AfterCurrent,
}
