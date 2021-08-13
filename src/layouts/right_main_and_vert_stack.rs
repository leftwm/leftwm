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

pub fn update(workspace: &Workspace, windows: &mut Vec<&mut Window>, tags: &mut Vec<Tag>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let primary_width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 100.0 * workspace.main_width(tags)).floor() as i32,
    };

    let thrid_part = workspace.width() - primary_width;

    let (mut main_x, mut stack_x) = match window_count {
        1 => (workspace.x(), 0),
        _ => (workspace.x() + thrid_part, workspace.x()),
    };
    if workspace.flipped_horizontal(tags) {
        main_x = workspace.x();
        stack_x = workspace.x() + primary_width;
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
        w.set_width(thrid_part);
        w.set_x(stack_x);
        w.set_y(workspace.y() + y);
        y += height;
    }
}
