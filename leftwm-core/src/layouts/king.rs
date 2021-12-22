use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two columns but keeps the main 
/// window over the columns, like a King sat in the throne.
/// 1 window
/// ```text
/// +-----------------------+
/// |                       |
/// |                       |
/// |           1           |
/// |                       |
/// |                       |
/// +-----------------------+
/// ```
/// 2 windows
/// ```text
/// +-----------+-----------+
/// |           |           |
/// |      +----+-----+     |
/// |   2  |    1     |     |
/// |      +----+-----+     |
/// |           |           |
/// +-----------+-----------+
/// ```
/// 3 windows
/// ```text
/// +-----------+-----------+
/// |           |           |
/// |      +----+-----+     |
/// |   2  |    1     |  3  |
/// |      +----+-----+     |
/// |           |           |
/// +-----------+-----------+
/// ```
pub fn update(workspace: &Workspace, tag: &Tag, windows: &mut Vec<&mut Window>) {
    let window_count = windows.len();
    if window_count == 0 {
        return;
    }

    let main_height = match window_count {
        1 => workspace.height() as i32,
        _ => (workspace.height() as f32 / 100.0 * tag.main_width_percentage()).floor() as i32,
    };
    let main_width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 100.0 * tag.main_width_percentage()).floor() as i32,
    };
    let width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 100.0 * tag.main_width_percentage()).floor() as i32,
    };
    let (x_main, y_main) = match window_count {
        1 => (workspace.x(), workspace.y()),
        _ => (
            workspace.x() + (workspace.width() - main_width) / 2,
            (workspace.y() + workspace.height() - main_height) / 2,
        ),
    };

    //build build the main window.
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_height(main_height);
            first.set_width(main_width);
            first.set_x(x_main);
            first.set_y(y_main);
        }
    }

    //stack all the others
    let divisor = (window_count - if window_count % 2 == 0 { 0 } else { 1 }) / 2;
    let height_f = workspace.height() as f32 / divisor as f32;
    let height = height_f.floor() as i32;
    let mut y = 0;
    let mut x = 0;
    for w in iter {
        w.set_height(height);
        w.set_width(workspace.width() - width);
        w.set_x(workspace.x() + x);
        w.set_y(workspace.y() + y);
        if x == 0 {
            x = width;
        } else {
            x = 0;
            y += height;
        }
    }
}
