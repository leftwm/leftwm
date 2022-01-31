use serde::{Deserialize, Serialize};

/// Helper enum to represent a size which can be
/// an absolute pixel value or a relative percentage value
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(untagged)]
pub enum Size {
    Pixel(i32),
    Ratio(f32),
}

impl Size {
    /// Turn the size into an absolute value.
    ///
    /// The pixel value will be returned as is, the ratio value will be multiplied by the provided
    /// `whole` to calculate the absolute value.
    #[must_use]
    pub fn into_absolute(self, whole: i32) -> i32 {
        match self {
            Size::Pixel(x) => x,
            Size::Ratio(x) => (whole as f32 * x).floor() as i32,
        }
    }
}
