use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into two columns.
/// Gives first column 2/3 workspace width on the right side, 1/3 for second column on left side.
/// Divides second column height for other windows.
///
/// Meant for ultra-wide monitors.
///
/// 1 window
/// +-----------------------------------------+
/// |                                         |
/// |                                         |
/// |                                         |
/// |                                         |
/// |                    1                    |
/// |                                         |
/// |                                         |
/// |                                         |
/// +-----------------------------------------+
/// 2 windows
/// +--------------------+--------------------+
/// |            |                            |
/// |            |                            |
/// |            |                            |
/// |     2      |             1              |
/// |            |                            |
/// |            |                            |
/// |            |                            |
/// |            |                            |
/// +--------------------+--------------------+
/// 3 windows
/// +--------------------+--------------------+
/// |             |                           |
/// |      2      |                           |
/// |             |                           |
/// +_____________+            1              |
/// |             |                           |
/// |      3      |                           |
/// |             |                           |
/// |             |                           |
/// +--------------------+--------------------+
/// 4 windows
/// +--------------------+--------------------+
/// |             |                           |
/// |      2      |                           |
/// +_____________+                           |
/// |             |             1             |
/// |      3      |                           |
/// +_____________+                           |
/// |             |                           |
/// |      4      |                           |
/// +--------------------+--------------------+

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
        1 => workspace_width,
        _ => (workspace_width as f32 / 100.0 * tag.main_width_percentage().floor()) as i32,
    };

    let third_part = workspace_width - primary_width;

    let (mut main_x, mut stack_x) = match window_count {
        1 => (workspace_x, 0),
        _ => (workspace_x + third_part, workspace_x),
    };
    if tag.flipped_horizontal {
        main_x = workspace_x;
        stack_x = workspace_x + primary_width;
    }

    let mut iter = windows.iter_mut();

    // build the primary window
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(primary_width);
            first.set_x(main_x);
            first.set_y(workspace.y());
        }
    }

    // build other windows
    let height = (workspace.height() as f32 / (window_count - 1) as f32).floor() as i32;

    let mut y = 0;

    for w in iter {
        w.set_height(height);
        w.set_width(third_part);
        w.set_x(stack_x);
        w.set_y(workspace.y() + y);
        y += height;
    }
}
