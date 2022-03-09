use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;
// use crate::models::WindowState;

/// Layout which gives only one window with the full desktop realestate. A monocle mode.
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

    let primary_width = match window_count {
        1 => workspace_width as i32,
        _ => (workspace_width as f32 / 100.0 * tag.main_width_percentage()).floor() as i32,
    };

    let mut main_x = workspace_x;
    let stack_x = if tag.flipped_horizontal {
        main_x = match window_count {
            1 => main_x,
            _ => main_x + workspace_width - primary_width,
        };
        match window_count {
            1 => 0,
            _ => workspace_x,
        }
    } else {
        workspace_x + primary_width
    };

    //Display main and second window
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(primary_width);
            first.set_x(main_x);
            first.set_y(workspace.y());

            first.set_visible(true);
        }
        if let Some(second) = iter.next() {
            second.set_height(workspace.height());
            second.set_width(workspace_width - primary_width);
            second.set_x(stack_x);
            second.set_y(workspace.y());

            second.set_visible(true);
        }
    }
    //Hide the other windows behind the second
    {
        if window_count > 2 {
            for w in iter {
                w.set_height(workspace.height());
                w.set_width(workspace_width - primary_width);
                w.set_x(stack_x);
                w.set_y(workspace.y());

                w.set_visible(false);
            }
        }
    }
}
