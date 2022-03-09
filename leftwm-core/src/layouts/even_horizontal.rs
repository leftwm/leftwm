use crate::models::Window;
use crate::models::Workspace;

/// Layout which gives each window full height, but splits the workspace width among them all.
pub fn update(workspace: &Workspace, windows: &mut [&mut Window]) {
    let window_count = windows.len();
    let width_f = workspace.width_limited(window_count) as f32 / windows.len() as f32;
    let width = width_f.floor() as i32;
    let mut x = 0;
    for w in windows.iter_mut() {
        w.set_height(workspace.height());
        w.set_width(width);
        w.set_x(workspace.x_limited(window_count) + x);
        w.set_y(workspace.y());
        x += width;
    }
}
