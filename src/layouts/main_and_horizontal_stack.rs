use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two columns, gives one window all of the left column,
/// and divides the right column among all the other windows.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();
    if window_count == 0 {
        return;
    }

    let height = match window_count {
        1 => workspace.height() as i32,
        _ => (workspace.height() as f32 / 100.0 * workspace.main_width()).floor() as i32,
    };

    //build build the main window.
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_width(workspace.width());
            first.set_height(height);
            first.set_x(workspace.x());
            first.set_y(workspace.y());
        }
    }

    //stack all the others
    let width_f = workspace.width() as f32 / (window_count - 1) as f32;
    let width = width_f.floor() as i32;
    let mut x = 0;
    for w in iter {
        w.set_height(workspace.height() - height);
        w.set_width(width);
        w.set_x(workspace.x() + x);
        w.set_y(workspace.y() + height);
        x += width;
    }
}
