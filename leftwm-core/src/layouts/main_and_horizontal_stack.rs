use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two rows, gives one window all of the upper row,
/// and divides the lower row among all the other windows.
pub fn update(workspace: &Workspace, tag: &Tag, windows: &mut [&mut Window]) {
    let window_count = windows.len();
    if window_count == 0 {
        return;
    }

    let column_count = match window_count {
        1 => 1,
        _ => 2,
    };
    let workspace_width = workspace.width_limited(column_count);
    let workspace_x = workspace.x_limited(column_count);

    let height = match window_count {
        1 => workspace.height() as i32,
        _ => (workspace.height() as f32 / 100.0 * tag.main_width_percentage()).floor() as i32,
    };

    let mut main_y = workspace.y();
    let mut stack_y = workspace.y() + height;
    if tag.flipped_vertical {
        main_y = match window_count {
            1 => main_y,
            _ => main_y + height,
        };
        stack_y = match window_count {
            1 => 0,
            _ => stack_y - height,
        };
    }

    //build the main window.
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_width(workspace_width);
            first.set_height(height);
            first.set_x(workspace_x);
            first.set_y(main_y);
        }
    }

    //stack all the others
    let width_f = workspace_width as f32 / (window_count - 1) as f32;
    let width = width_f.floor() as i32;
    let mut x = 0;
    for w in iter {
        w.set_height(workspace.height() - height);
        w.set_width(width);
        w.set_x(workspace_x + x);
        w.set_y(stack_y);
        x += width;
    }
}
