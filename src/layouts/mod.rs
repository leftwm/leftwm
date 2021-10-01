use super::models::Window;
use super::models::Workspace;
use crate::models::Tag;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

mod center_main;
mod center_main_balanced;
mod even_horizontal;
mod even_vertical;
mod fibonacci;
mod grid_horizontal;
mod main_and_deck;
mod main_and_horizontal_stack;
mod main_and_vert_stack;
mod monocle;
mod right_main_and_vert_stack;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Layout {
    MainAndVertStack,
    MainAndHorizontalStack,
    MainAndDeck,
    GridHorizontal,
    EvenHorizontal,
    EvenVertical,
    Fibonacci,
    CenterMain,
    CenterMainBalanced,
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
    Layout::CenterMain,
    Layout::CenterMainBalanced,
    Layout::Monocle,
    Layout::RightWiderLeftStack,
    Layout::LeftWiderRightStack,
];

impl Default for Layout {
    fn default() -> Self {
        Layout::MainAndVertStack
    }
}

// This is tedious, but simple and effective.
impl Layout {
    pub fn new(layouts: &[Self]) -> Self {
        if let Some(layout) = layouts.first() {
            return *layout;
        }
        Self::Fibonacci
    }
    pub fn update_windows(
        &self,
        workspace: &Workspace,
        windows: &mut Vec<&mut Window>,
        tags: &mut Vec<Tag>,
    ) {
        match self {
            Self::MainAndVertStack | Self::LeftWiderRightStack => {
                main_and_vert_stack::update(workspace, windows, tags);
            }
            Self::MainAndHorizontalStack => {
                main_and_horizontal_stack::update(workspace, windows, tags);
            }
            Self::MainAndDeck => main_and_deck::update(workspace, windows, tags),
            Self::GridHorizontal => grid_horizontal::update(workspace, windows),
            Self::EvenHorizontal => even_horizontal::update(workspace, windows),
            Self::EvenVertical => even_vertical::update(workspace, windows),
            Self::Fibonacci => fibonacci::update(workspace, windows, tags),
            Self::CenterMain => center_main::update(workspace, windows, tags),
            Self::CenterMainBalanced => center_main_balanced::update(workspace, windows, tags),
            Self::Monocle => monocle::update(workspace, windows),
            Self::RightWiderLeftStack => {
                right_main_and_vert_stack::update(workspace, windows, tags);
            }
        }
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
            Self::Fibonacci => {
                [(false, false), (true, false), (true, true), (false, true)].to_vec()
            }
            //Layouts that can be flipped vertically
            Self::MainAndHorizontalStack => [(false, false), (false, true)].to_vec(),
            //Layouts that can be flipped horizontally
            _ => [(false, false), (true, false)].to_vec(),
        }
    }

    pub fn next_layout(&self, layouts: &[Self]) -> Self {
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return Self::Fibonacci,
        } + 1;
        if index >= layouts.len() as isize {
            index = 0;
        }
        layouts[index as usize]
    }

    pub fn prev_layout(&self, layouts: &[Self]) -> Self {
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return Self::Fibonacci,
        } - 1;
        if index < 0 {
            index = layouts.len() as isize - 1;
        }
        layouts[index as usize]
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
            "CenterMain" => Ok(Self::CenterMain),
            "CenterMainBalanced" => Ok(Self::CenterMainBalanced),
            "Monocle" => Ok(Self::Monocle),
            "RightWiderLeftStack" => Ok(Self::RightWiderLeftStack),
            "LeftWiderRightStack" => Ok(Self::LeftWiderRightStack),
            _ => Err(ParseLayoutError(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, Margins, WindowHandle};

    #[test]
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
            vec![],
            vec![],
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
        // let mut windows_filters: Vec<&mut Window> = windows.iter_mut().filter(|_f| true).collect();
        even_horizontal::update(&ws, &mut windows);
        assert!(
            w.height() == 600,
            "window was not size to the correct height"
        );
        assert!(w.width() == 800, "window was not size to the correct width");
    }

    #[test]
    fn test_from_str() {
        let layout_strs: [&str; 12] = [
            "MainAndVertStack",
            "MainAndHorizontalStack",
            "MainAndDeck",
            "GridHorizontal",
            "EvenHorizontal",
            "EvenVertical",
            "Fibonacci",
            "CenterMain",
            "CenterMainBalanced",
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
