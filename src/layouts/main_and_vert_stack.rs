use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two columns, gives one window all of the left column,
/// and divides the right column among all the other windows.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut Window>) {
    let window_count = windows.len();
    if window_count == 0 {
        return;
    }

    let width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 100.0 * workspace.main_width()).floor() as i32,
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
        w.set_width(workspace.width() - width);
        w.set_x(workspace.x() + width);
        w.set_y(workspace.y() + y);
        y += height;
    }
}
