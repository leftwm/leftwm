use super::models::Window;
use super::models::Workspace;
use std::collections::VecDeque;

pub trait Layout: LayoutClone {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut Window>);
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
    layouts.push_back(Box::new(EvenHorizontal {}) as Box<Layout>);
    layouts.push_back(Box::new(EvenVertical {}) as Box<Layout>);
    layouts
}

//pub type DefaultLayout = EvenVertical;
//pub type DefaultLayout = EvenHorizontal;

#[derive(Clone, Debug)]
pub struct EvenHorizontal {}
impl Layout for EvenHorizontal {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut Window>) {
        let width_f = workspace.width as f32 / windows.len() as f32;
        let width = width_f.floor() as i32;
        let mut x = 0;
        for w in windows.iter_mut() {
            w.set_height(workspace.height);
            w.set_width(width);
            w.set_x(workspace.x + x);
            w.set_y(workspace.y);
            x += width;
        }
    }
}

#[derive(Clone, Debug)]
pub struct EvenVertical {}
impl Layout for EvenVertical {
    fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut Window>) {
        let height_f = workspace.height as f32 / windows.len() as f32;
        let height = height_f.floor() as i32;
        let mut y = 0;
        for w in windows.iter_mut() {
            w.set_height(height);
            w.set_width(workspace.width);
            w.set_x(workspace.x);
            w.set_y(workspace.y + y);
            y += height;
        }
    }
}

#[test]
fn should_fullscreen_a_single_window() {
    use super::models::WindowHandle;
    let layout = EvenHorizontal {};
    let mut ws = Workspace::new();
    ws.height = 1000;
    ws.width = 2000;
    let mut w = Window::new(WindowHandle::MockHandle(1), None);
    w.border = 0;
    w.margin = 0;
    let mut windows = vec![&mut w];
    layout.update_windows(&ws, &mut windows);
    assert!(
        w.height() == 1000,
        "window was not size to the correct height"
    );
    assert!(
        w.width() == 2000,
        "window was not size to the correct width"
    );
}
