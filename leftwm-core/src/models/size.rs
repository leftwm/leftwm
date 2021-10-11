use serde::{Deserialize, Serialize};

/// Helper enum to represent a size which can be
/// an absolute pixel value or a relative percentage value
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(untagged)]
pub enum Size {
    Pixel(i32),
    Percentage(f32),
}

impl Size {
    /// Turn the size into an absolute value, the pixel value
    /// will be returned as is, the percentage value will be
    /// multiplied by the provided `whole` to calculate
    /// the absolute value
    #[must_use]
    pub fn into_absolute(self, whole: f32) -> f32 {
        match self {
            Size::Pixel(x) => x as f32,
            Size::Percentage(x) => whole * x,
        }
    }
}
