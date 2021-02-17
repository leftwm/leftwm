use super::models::Window;
use super::models::Workspace;
use serde::{Deserialize, Serialize};

mod center_main;
mod even_horizontal;
mod even_vertical;
mod fibonacci;
mod grid_horizontal;
mod main_and_horizontal_stack;
mod main_and_vert_stack;
mod monocle;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Layout {
    MainAndVertStack,
    MainAndHorizontalStack,
    GridHorizontal,
    EvenHorizontal,
    EvenVertical,
    Fibonacci,
    CenterMain,
    Monocle,
}

impl Default for Layout {
    fn default() -> Self {
        let layouts: Vec<Layout> = LAYOUTS.iter().map(|&x| x.into()).collect();
        layouts.first().unwrap().clone()
    }
}

const LAYOUTS: &[&str] = &[
    "MainAndVertStack",
    "MainAndHorizontalStack",
    "GridHorizontal",
    "EvenHorizontal",
    "EvenVertical",
    "Fibonacci",
    "CenterMain",
    "Monocle",
];

impl From<&str> for Layout {
    fn from(s: &str) -> Self {
        match s {
            "MainAndVertStack" => Self::MainAndVertStack,
            "MainAndHorizontalStack" => Self::MainAndHorizontalStack,
            "GridHorizontal" => Self::GridHorizontal,
            "EvenHorizontal" => Self::EvenHorizontal,
            "EvenVertical" => Self::EvenVertical,
            "Fibonacci" => Self::Fibonacci,
            "CenterMain" => Self::CenterMain,
            "Monocle" => Self::Monocle,
            _ => Self::MainAndVertStack,
        }
    }
}

// This is tedious, but simple and effective.
impl Layout {
    pub fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        match self {
            Self::MainAndVertStack => main_and_vert_stack::update(workspace, windows),
            Self::MainAndHorizontalStack => main_and_horizontal_stack::update(workspace, windows),
            Self::GridHorizontal => grid_horizontal::update(workspace, windows),
            Self::EvenHorizontal => even_horizontal::update(workspace, windows),
            Self::EvenVertical => even_vertical::update(workspace, windows),
            Self::Fibonacci => fibonacci::update(workspace, windows),
            Self::CenterMain => center_main::update(workspace, windows),
            Self::Monocle => monocle::update(workspace, windows),
        }
    }

    pub fn next_layout(&self) -> Self {
        let layouts: Vec<Layout> = LAYOUTS.iter().map(|&x| x.into()).collect();
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return "Fibonacci".into(),
        } + 1;
        if index >= layouts.len() as isize {
            index = 0;
        }
        layouts[index as usize].clone()
    }

    pub fn prev_layout(&self) -> Self {
        let layouts: Vec<Layout> = LAYOUTS.iter().map(|&x| x.into()).collect();
        let mut index = match layouts.iter().position(|x| x == self) {
            Some(x) => x as isize,
            None => return "Fibonacci".into(),
        } - 1;
        if index < 0 {
            index = layouts.len() as isize - 1;
        }
        layouts[index as usize].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, WindowHandle};

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
        );
        ws.xyhw.set_minh(600);
        ws.xyhw.set_minw(800);
        ws.update_avoided_areas();
        let mut w = Window::new(WindowHandle::MockHandle(1), None);
        w.border = 0;
        w.margin = 0;
        let mut windows = vec![&mut w];
        let mut windows_filters = windows.iter_mut().filter(|_f| true).collect();
        even_horizontal::update(&ws, &mut windows_filters);
        assert!(
            w.height() == 600,
            "window was not size to the correct height"
        );
        assert!(w.width() == 800, "window was not size to the correct width");
    }
}
