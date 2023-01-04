mod layout_manager;
mod layout_mode;

use thiserror::Error;

pub use layout_manager::LayoutManager;
pub use layout_mode::LayoutMode;

pub const DEFAULT: &str = "Default";
pub const MONOCLE: &str = "Monocle";
pub const MAIN_AND_DECK: &str = "MainAndDeck";
pub const LEFT_WIDER_RIGHT_STACK: &str = "LeftWiderRightStack";
pub const RIGHT_WIDER_LEFT_STACK: &str = "RightWiderLeftStack";
pub const MAIN_AND_VERT_STACK: &str = "MainAndVertStack";
pub const MAIN_AND_HORIZONTAL_STACK: &str = "MainAndHorizontalStack";
pub const GRID_HORIZONTAL: &str = "GridHorizontal";
pub const EVEN_HORIZONTAL: &str = "EvenHorizontal";
pub const EVEN_VERTICAL: &str = "EvenVertical";
pub const FIBONACCI: &str = "Fibonacci";
pub const LEFT_MAIN: &str = "LeftMain";
pub const CENTER_MAIN: &str = "CenterMain";
pub const CENTER_MAIN_BALANCED: &str = "CenterMainBalanced";
pub const CENTER_MAIN_FLUID: &str = "CenterMainFluid";

#[derive(Debug, Error)]
#[error("Could not parse layout: {0}")]
pub struct ParseLayoutError(String);

#[cfg(test)]
mod tests {}
