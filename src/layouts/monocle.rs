use crate::models::Window;
use crate::models::Workspace;

/// Layout which gives only one window with the full desktop realestate. A monocle mode.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let height = workspace.height() as i32;
    let mut y = 0;
    for w in windows.iter_mut() {
        w.set_height(height);
        w.set_width(workspace.width());
        w.set_x(workspace.x());
        w.set_y(workspace.y() + y);
        y += height;
    }
}
