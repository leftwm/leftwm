use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Margins {
    pub const fn new(size: u32) -> Self {
        Self {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }

    pub const fn new_from_pair(top_and_bottom: u32, left_and_right: u32) -> Self {
        Self {
            top: top_and_bottom,
            right: left_and_right,
            bottom: top_and_bottom,
            left: left_and_right,
        }
    }

    pub const fn new_from_triple(top: u32, left_and_right: u32, bottom: u32) -> Self {
        Self {
            top,
            right: left_and_right,
            bottom,
            left: left_and_right,
        }
    }
}
