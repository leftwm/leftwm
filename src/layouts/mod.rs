use super::models::Window;
use super::models::Workspace;
use serde::{Deserialize, Serialize};

mod center_main;
mod even_horizontal;
mod even_vertical;
mod fibonacci;
mod grid_horizontal;
mod main_and_vert_stack;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Layout {
    MainAndVertStack,
    GridHorizontal,
    EvenHorizontal,
    EvenVertical,
    Fibonacci,
    CenterMain,
}

impl Default for Layout {
    fn default() -> Self {
        Self::MainAndVertStack
    }
}

// This is tedious, but simple and effective.
impl Layout {
    pub fn next_layout(&self) -> Self {
        match self {
            Self::MainAndVertStack => Self::GridHorizontal,
            Self::GridHorizontal => Self::EvenHorizontal,
            Self::EvenHorizontal => Self::EvenVertical,
            Self::EvenVertical => Self::Fibonacci,
            Self::Fibonacci => Self::CenterMain,
            Self::CenterMain => Self::MainAndVertStack,
        }
    }

    pub fn prev_layout(&self) -> Self {
        match self {
            Self::MainAndVertStack => Self::CenterMain,
            Self::GridHorizontal => Self::MainAndVertStack,
            Self::EvenHorizontal => Self::GridHorizontal,
            Self::EvenVertical => Self::EvenHorizontal,
            Self::Fibonacci => Self::EvenVertical,
            Self::CenterMain => Self::Fibonacci,
        }
    }

    pub fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        match self {
            Self::MainAndVertStack => main_and_vert_stack::update(workspace, windows),
            Self::GridHorizontal => grid_horizontal::update(workspace, windows),
            Self::EvenHorizontal => even_horizontal::update(workspace, windows),
            Self::EvenVertical => even_vertical::update(workspace, windows),
            Self::Fibonacci => fibonacci::update(workspace, windows),
            Self::CenterMain => center_main::update(workspace, windows),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, WindowHandle};

    #[test]
    fn should_fullscreen_a_single_window() {
        //size defaults to 600x800
        let mut ws = Workspace::new(BBox {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
        });
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
