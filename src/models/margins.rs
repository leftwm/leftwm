use serde::{Deserialize, Serialize};

// TODO custom serialize/deserialize
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Margins {
    #[must_use]
    pub fn into_vec(self) -> Vec<u32> {
        vec![self.top, self.right, self.bottom, self.left]
    }

    pub fn new(size: u32) -> Self {
        Self {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }

    pub fn new_from_pair(top_and_bottom: u32, left_and_right: u32) -> Self {
        Self {
            top: top_and_bottom,
            right: left_and_right,
            bottom: top_and_bottom,
            left: left_and_right,
        }
    }
}
