use super::models::Window;
use super::models::Workspace;
use crate::models::Tag;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;


#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    MainAndVertStack,
    MainAndHorizontalStack,
    MainAndDeck,
    GridHorizontal,
    EvenHorizontal,
    EvenVertical,
    Fibonacci,
    LeftMain,
    CenterMain,
    CenterMainBalanced,
    CenterMainFluid,
    Monocle,
    RightWiderLeftStack,
    LeftWiderRightStack,
}

pub const LAYOUTS: &[Layout] = &[
    Layout::MainAndVertStack,
    Layout::MainAndHorizontalStack,
    Layout::MainAndDeck,
    Layout::GridHorizontal,
    Layout::EvenHorizontal,
    Layout::EvenVertical,
    Layout::Fibonacci,
    Layout::LeftMain,
    Layout::CenterMain,
    Layout::CenterMainBalanced,
    Layout::CenterMainFluid,
    Layout::Monocle,
    Layout::RightWiderLeftStack,
    Layout::LeftWiderRightStack,
];

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


impl Default for Layout {
    fn default() -> Self {
        Self::MainAndVertStack
    }
}

pub struct NewLayout {}

impl NewLayout {
    pub fn update_windows(&self, workspace: &Workspace, windows: &mut [&mut Window], tag: &Tag) {
        // leftwm_layouts::apply()
        println!("test")
    }
}

// This is tedious, but simple and effective.
impl Layout {
    pub fn update_windows(&self, workspace: &Workspace, windows: &mut [&mut Window], tag: &Tag) {
        // todo: remove method
        /*match self {
            Self::MainAndVertStack | Self::LeftWiderRightStack => {
                main_and_vert_stack::update(workspace, tag, windows);
            }
            Self::MainAndHorizontalStack => {
                main_and_horizontal_stack::update(workspace, tag, windows);
            }
            Self::MainAndDeck => main_and_deck::update(workspace, tag, windows),
            Self::GridHorizontal => grid_horizontal::update(workspace, tag, windows),
            Self::EvenHorizontal => even_horizontal::update(workspace, windows),
            Self::EvenVertical => even_vertical::update(workspace, windows),
            Self::Fibonacci => fibonacci::update(workspace, tag, windows),
            Self::LeftMain => left_main::update(workspace, tag, windows),
            Self::CenterMain => center_main::update(workspace, tag, windows),
            Self::CenterMainBalanced => center_main_balanced::update(workspace, tag, windows),
            Self::CenterMainFluid => center_main_fluid::update(workspace, tag, windows),
            Self::Monocle => monocle::update(workspace, windows),
            Self::RightWiderLeftStack => {
                right_main_and_vert_stack::update(workspace, tag, windows);
            }
        }*/
    }

    pub const fn main_width(&self) -> u8 {
        match self {
            Self::RightWiderLeftStack | Self::LeftWiderRightStack => 75,
            _ => 50,
        }
    }

    //The possible permutations that a layout can be flipped => (flipable_horz, flipable_vert)
    pub fn rotations(&self) -> Vec<(bool, bool)> {
        match self {
            //Layouts that can be flipped both ways
            Self::Fibonacci | Self::GridHorizontal => {
                [(false, false), (true, false), (true, true), (false, true)].to_vec()
            }
            //Layouts that can be flipped vertically
            Self::MainAndHorizontalStack => [(false, false), (false, true)].to_vec(),
            //Layouts that can be flipped horizontally
            _ => [(false, false), (true, false)].to_vec(),
        }
    }
}

#[derive(Debug, Error)]
#[error("Could not parse layout: {0}")]
pub struct ParseLayoutError(String);

impl FromStr for Layout {
    type Err = ParseLayoutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MainAndVertStack" => Ok(Self::MainAndVertStack),
            "MainAndHorizontalStack" => Ok(Self::MainAndHorizontalStack),
            "MainAndDeck" => Ok(Self::MainAndDeck),
            "GridHorizontal" => Ok(Self::GridHorizontal),
            "EvenHorizontal" => Ok(Self::EvenHorizontal),
            "EvenVertical" => Ok(Self::EvenVertical),
            "Fibonacci" => Ok(Self::Fibonacci),
            "LeftMain" => Ok(Self::LeftMain),
            "CenterMain" => Ok(Self::CenterMain),
            "CenterMainBalanced" => Ok(Self::CenterMainBalanced),
            "CenterMainFluid" => Ok(Self::CenterMainFluid),
            "Monocle" => Ok(Self::Monocle),
            "RightWiderLeftStack" => Ok(Self::RightWiderLeftStack),
            "LeftWiderRightStack" => Ok(Self::LeftWiderRightStack),
            _ => Err(ParseLayoutError(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use leftwm_layouts::LayoutDefinition;

    use super::*;
    use crate::models::{BBox, Margins, WindowHandle};

    /*#[test]
    fn should_fullscreen_a_single_window() {
        //size defaults to 600x800
        let mut ws = Workspace::new(
            None,
            BBox {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            LayoutDefinition::default().name, // todo
            None,
        );
        ws.margin = Margins::new(0);
        ws.xyhw.set_minh(600);
        ws.xyhw.set_minw(800);
        ws.update_avoided_areas();
        let mut w = Window::new(WindowHandle::MockHandle(1), None, None);
        w.border = 0;
        w.margin = Margins::new(0);
        let mut windows = vec![&mut w];
        even_horizontal::update(&ws, &mut windows);
        assert!(
            w.height() == 600,
            "window was not size to the correct height"
        );
        assert!(w.width() == 800, "window was not size to the correct width");
    }*/

    #[test]
    fn test_from_str() {
        let layout_strs: [&str; 14] = [
            "MainAndVertStack",
            "MainAndHorizontalStack",
            "MainAndDeck",
            "GridHorizontal",
            "EvenHorizontal",
            "EvenVertical",
            "Fibonacci",
            "LeftMain",
            "CenterMain",
            "CenterMainBalanced",
            "CenterMainFluid",
            "Monocle",
            "RightWiderLeftStack",
            "LeftWiderRightStack",
        ];

        assert_eq!(layout_strs.len(), LAYOUTS.len());

        for (i, layout) in LAYOUTS.iter().enumerate() {
            assert_eq!(
                layout,
                &Layout::from_str(layout_strs[i]).expect("Layout String")
            );
        }
    }
}
