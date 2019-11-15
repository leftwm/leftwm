use super::models::Window;
use super::models::Workspace;
use std::collections::VecDeque;

pub trait Layout: LayoutClone {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>);
}

pub trait LayoutClone {
    fn clone_box(&self) -> Box<Layout>;
}

impl<T: 'static + Layout + Clone> LayoutClone for T {
    fn clone_box(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Layout> {
    fn clone(&self) -> Box<Layout> {
        self.clone_box()
    }
}

pub fn get_all_layouts() -> VecDeque<Box<Layout>> {
    let mut layouts = VecDeque::new();
    layouts.push_back(Box::new(MainAndVertStack {}) as Box<Layout>);
    layouts.push_back(Box::new(EvenHorizontal {}) as Box<Layout>);
    layouts.push_back(Box::new(EvenVertical {}) as Box<Layout>);
    layouts
}

#[derive(Clone, Debug)]
pub struct EvenHorizontal {}
impl Layout for EvenHorizontal {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        let width_f = workspace.width() as f32 / windows.len() as f32;
        let width = width_f.floor() as i32;
        let mut x = 0;
        for w in windows.iter_mut() {
            w.set_height(workspace.height());
            w.set_width(width);
            w.set_x(workspace.x() + x);
            w.set_y(workspace.y());
            x += width;
        }
    }
}

#[derive(Clone, Debug)]
pub struct EvenVertical {}
impl Layout for EvenVertical {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        let height_f = workspace.height() as f32 / windows.len() as f32;
        let height = height_f.floor() as i32;
        let mut y = 0;
        for w in windows.iter_mut() {
            w.set_height(height);
            w.set_width(workspace.width());
            w.set_x(workspace.x());
            w.set_y(workspace.y() + y);
            y += height;
        }
    }
}

#[derive(Clone, Debug)]
pub struct MainAndVertStack {}
impl Layout for MainAndVertStack {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        let window_count = windows.len();
        if window_count == 0 {
            return;
        }

        let width = match window_count {
            1 => workspace.width() as i32,
            _ => (workspace.width() as f32 / 2 as f32).floor() as i32,
        };

        //build build the main window.
        let mut iter = windows.iter_mut();
        {
            if let Some(first) = iter.next() {
                first.set_height(workspace.height());
                first.set_width(width);
                first.set_x(workspace.x());
                first.set_y(workspace.y());
            }
        }

        //stack all the others
        let height_f = workspace.height() as f32 / (window_count - 1) as f32;
        let height = height_f.floor() as i32;
        let mut y = 0;
        for w in iter {
            w.set_height(height);
            w.set_width(width);
            w.set_x(workspace.x() + width);
            w.set_y(workspace.y() + y);
            y += height;
        }
    }
}

#[test]
fn should_fullscreen_a_single_window() {
    use super::models::WindowHandle;
    let layout = EvenHorizontal {};
    //size defaults to 600x800
    let mut ws = Workspace::new();
    ws.xyhw.set_minh(600);
    ws.xyhw.set_minw(800);
    ws.update_avoided_areas();
    let mut w = Window::new(WindowHandle::MockHandle(1), None);
    w.border = 0;
    w.margin = 0;
    let mut windows = vec![&mut w];
    let mut windows_filters = windows.iter_mut().filter(|_f| true).collect();
    layout.update_windows(&ws, &mut windows_filters);
    assert!(
        w.height() == 600,
        "window was not size to the correct height"
    );
    assert!(w.width() == 800, "window was not size to the correct width");
}
