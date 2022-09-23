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
    pub output: Option<String>,
    pub num: Option<usize>,
}

impl Gutter {
    #[must_use]
    pub const fn new(side: Side, value: i32, output: Option<String>, num: Option<usize>) -> Self {
        Self {
            side,
            value,
            output,
            num,
        }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            side: Side::Top,
            value: 0,
            output: None,
            num: None,
        }
    }
}
