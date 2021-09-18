use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two columns, gives one window all of the left column,
/// and divides the right column among all the other windows.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut Window>, tags: &mut Vec<Tag>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let workspace_width = workspace.width_limited(window_count);
    let workspace_x = workspace.x_limited(window_count);

    let primary_width = match window_count {
        1 => workspace_width as i32,
        _ => (workspace_width as f32 / 100.0 * workspace.main_width(tags)).floor() as i32,
    };

    let mut main_x = workspace_x;
    let mut stack_x = workspace_x + primary_width;
    if workspace.flipped_horizontal(tags) {
        main_x = match window_count {
            1 => main_x,
            _ => main_x + workspace_width - primary_width,
        };
        stack_x = match window_count {
            1 => 0,
            _ => workspace_x,
        };
    }

    //build the main window.
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(primary_width);
            first.set_x(main_x);
            first.set_y(workspace.y());
        }
    }

    //stack all the others
    let height_f = workspace.height() as f32 / (window_count - 1) as f32;
    let height = height_f.floor() as i32;
    let mut y = 0;
    for w in iter {
        w.set_height(height);
        w.set_width(workspace_width - primary_width);
        w.set_x(stack_x);
        w.set_y(workspace.y() + y);
        y += height;
    }
}
