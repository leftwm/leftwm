use serde::{Deserialize, Serialize};

type WorkSpaceID = i32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Gutter {
    pub side: Side,
    pub value: i32,
    pub wsid: Option<WorkSpaceID>,
}

impl Gutter {
    #[must_use]
    pub const fn new(side: Side, value: i32, wsid: Option<WorkSpaceID>) -> Self {
        Self { side, value, wsid }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            side: Side::Top,
            value: 0,
            wsid: None,
        }
    }
}
