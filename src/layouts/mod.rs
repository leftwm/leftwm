use super::utils::window::*;
use super::utils::workspace::Workspace;

pub trait Layout: LayoutClone {
    fn update_windows(&self, workspace: &Workspace, windows: Vec<&mut Window>);
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

pub type DefaultLayout = EvenHorizontal;

#[derive(Clone)]
pub struct EvenHorizontal {}
impl Layout for EvenHorizontal {
    fn update_windows(&self, workspace: &Workspace, windows: Vec<&mut Window>) {
        let width_f = workspace.width as f32 / windows.len() as f32;
        let width = width_f.floor() as i32;
        let mut x = 0;
        for w in windows {
            w.set_height(workspace.height);
            w.set_width(width);
            w.set_x(workspace.x + x);
            w.set_y(workspace.y);
            x += width;
        }
    }
}

#[test]
fn should_fullscreen_a_single_window() {
    let layout = EvenHorizontal {};
    let mut ws = Workspace::new();
    ws.height = 1000;
    ws.width = 2000;
    let mut w = Window::new(WindowHandle::MockHandle(1), None);
    let windows = vec![&mut w];
    layout.update_windows(&ws, windows);
    assert!(
        w.height == 1000,
        "window was not size to the correct height"
    );
    assert!(w.width == 2000, "window was not size to the correct width");
}
