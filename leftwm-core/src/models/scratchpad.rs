use crate::models::Size;
use serde::{Deserialize, Serialize};

use super::{Xyhw, XyhwBuilder};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScratchPad {
    pub name: ScratchPadName,
    pub args: Option<Vec<String>>,
    pub value: String,
    // relative x of scratchpad, 25 means 25% of workspace x
    pub x: Option<Size>,
    // relative y of scratchpad, 25 means 25% of workspace y
    pub y: Option<Size>,
    // relative height of scratchpad, 50 means 50% of workspace height
    pub height: Option<Size>,
    // relative width of scratchpad, 50 means 50% of workspace width
    pub width: Option<Size>,
}

impl ScratchPad {
    // Get size and position of scratchpad from config and workspace size.
    pub fn xyhw(&self, xyhw: &Xyhw) -> Xyhw {
        let x_sane = sane_dimension(self.x, 0.25, xyhw.w());
        let y_sane = sane_dimension(self.y, 0.25, xyhw.h());
        let height_sane = sane_dimension(self.height, 0.50, xyhw.h());
        let width_sane = sane_dimension(self.width, 0.50, xyhw.w());

        XyhwBuilder {
            x: xyhw.x() + x_sane,
            y: xyhw.y() + y_sane,
            h: height_sane,
            w: width_sane,
            ..XyhwBuilder::default()
        }
        .into()
    }
}

/// Newtype used as the name for a scratchpad, can be seen as some sort of symbol in languages like
/// Lisp/Scheme/...
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(from = "String")]
#[serde(into = "String")]
pub struct ScratchPadName(String);

impl From<String> for ScratchPadName {
    fn from(other: String) -> Self {
        Self(other)
    }
}

impl From<ScratchPadName> for String {
    fn from(other: ScratchPadName) -> Self {
        other.0
    }
}

impl From<&str> for ScratchPadName {
    fn from(other: &str) -> Self {
        Self(other.to_string())
    }
}

impl PartialEq<&str> for ScratchPadName {
    fn eq(&self, other: &&str) -> bool {
        &self.0.as_str() == other
    }
}

fn sane_dimension(config_value: Option<Size>, default_ratio: f32, max_pixel: i32) -> i32 {
    match config_value {
        Some(size @ Size::Ratio(r)) if (0.0..=1.0).contains(&r) => size.into_absolute(max_pixel),
        Some(Size::Pixel(pixel)) if (0..=max_pixel).contains(&pixel) => pixel,
        _ => Size::Ratio(default_ratio).into_absolute(max_pixel),
    }
}
