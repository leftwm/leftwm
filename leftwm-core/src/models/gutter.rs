use serde::{Deserialize, Serialize};

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
    pub id: Option<usize>,
}

impl Gutter {
    #[must_use]
    pub const fn new(side: Side, value: i32, id: Option<usize>) -> Self {
        Self { side, value, id }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            side: Side::Top,
            value: 0,
            id: None,
        }
    }
}
