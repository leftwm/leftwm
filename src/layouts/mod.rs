use super::models::Window;
use super::models::Workspace;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

pub const LAYOUTS: [Layout; 12] = [
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

// This is tedious, but simple and effective.
impl Layout {
    pub fn new(layouts: &[Layout]) -> Self {
        if let Some(layout) = layouts.first() {
            return layout.clone();
        }
        Layout::Fibonacci
    }
    pub fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut Window>) {
        match self {
            Self::MainAndVertStack | Self::LeftWiderRightStack => {
                main_and_vert_stack::update(workspace, windows)
            }
            Self::MainAndHorizontalStack => main_and_horizontal_stack::update(workspace, windows),
            Self::MainAndDeck => main_and_deck::update(workspace, windows),
            Self::GridHorizontal => grid_horizontal::update(workspace, windows),
            Self::EvenHorizontal => even_horizontal::update(workspace, windows),
            Self::EvenVertical => even_vertical::update(workspace, windows),
            Self::Fibonacci => fibonacci::update(workspace, windows),
            Self::CenterMain => center_main::update(workspace, windows),
            Self::CenterMainBalanced => center_main_balanced::update(workspace, windows),
            Self::Monocle => monocle::update(workspace, windows),
            Self::RightWiderLeftStack => right_main_and_vert_stack::update(workspace, windows),
        }
    }

    pub fn main_width(&self) -> u8 {
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

    pub fn next_layout(&self, layouts: &[Layout]) -> Self {
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return Layout::Fibonacci,
        } + 1;
        if index >= layouts.len() as isize {
            index = 0;
        }
        layouts[index as usize].clone()
    }

    pub fn prev_layout(&self, layouts: &[Layout]) -> Self {
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return Layout::Fibonacci,
        } - 1;
        if index < 0 {
            index = layouts.len() as isize - 1;
        }
        layouts[index as usize].clone()
    }
}

// TODO: Perhaps there is a more efficient way to impl FromStr using serde
impl FromStr for Layout {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MainAndVertStack" => Ok(Layout::MainAndVertStack),
            "MainAndHorizontalStack" => Ok(Layout::MainAndHorizontalStack),
            "MainAndDeck" => Ok(Layout::MainAndDeck),
            "GridHorizontal" => Ok(Layout::GridHorizontal),
            "EvenHorizontal" => Ok(Layout::EvenHorizontal),
            "EvenVertical" => Ok(Layout::EvenVertical),
            "Fibonacci" => Ok(Layout::Fibonacci),
            "CenterMain" => Ok(Layout::CenterMain),
            "CenterMainBalanced" => Ok(Layout::CenterMainBalanced),
            "Monocle" => Ok(Layout::Monocle),
            "RightWiderLeftStack" => Ok(Layout::RightWiderLeftStack),
            "LeftWiderRightStack" => Ok(Layout::LeftWiderRightStack),
            _ => Err(()),
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
            BBox {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            vec![],
            vec![],
        );
        ws.margin = Margins::Int(0);
        ws.xyhw.set_minh(600);
        ws.xyhw.set_minw(800);
        ws.update_avoided_areas();
        let mut w = Window::new(WindowHandle::MockHandle(1), None, None);
        w.border = 0;
        w.margin = Margins::Int(0);
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
