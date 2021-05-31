use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Gutter {
    pub side: Side,
    pub value: i32,
}

impl Gutter {
    #[must_use]
    pub fn new(side: Side, value: i32) -> Gutter {
        Gutter { side, value }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Gutter {
            side: Side::Top,
            value: 0,
        }
    }
}
