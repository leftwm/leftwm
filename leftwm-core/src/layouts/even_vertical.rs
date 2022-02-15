use crate::models::Window;
use crate::models::Workspace;

/// Layout which gives each window full width, but splits the workspace height among them all.
pub fn update(workspace: &Workspace, windows: &mut [&mut Window]) {
    let height_f = workspace.height() as f32 / windows.len() as f32;
    let height = height_f.floor() as i32;
    let mut y = 0;
    for w in windows.iter_mut() {
        w.set_height(height);
        w.set_width(workspace.width_limited(1));
        w.set_x(workspace.x_limited(1));
        w.set_y(workspace.y() + y);
        y += height;
    }
}
