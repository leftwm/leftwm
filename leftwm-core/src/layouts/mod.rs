use super::models::Window;
use super::models::Workspace;
use crate::models::Tag;
use thiserror::Error;

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

/*
pub struct NewLayout {}

impl NewLayout {
    pub fn update_windows(&self, workspace: &Workspace, windows: &mut [&mut Window], tag: &Tag) {
        // leftwm_layouts::apply()
        println!("test")
    }
}*/

#[derive(Debug, Error)]
#[error("Could not parse layout: {0}")]
pub struct ParseLayoutError(String);

#[cfg(test)]
mod tests {
    
}
