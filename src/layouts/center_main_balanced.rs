use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into three columns.
/// Gives first window all of the center column.
/// Divides the left and right columns among all other windows in a fibonacci layout.
///
/// Meant for ultra-wide monitors.
///
/// 1 window
/// ```text
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
/// ```
/// 2 windows
/// ```text
/// +--------------------+--------------------+
/// |                    |                    |
/// |                    |                    |
/// |                    |                    |
/// |         2          |         1          |
/// |                    |                    |
/// |                    |                    |
/// |                    |                    |
/// |                    |                    |
/// +--------------------+--------------------+
/// ```
/// 3 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |             |             |             |
/// |             |             |             |
/// |      2      |      1      |      3      |
/// |             |             |             |
/// |             |             |             |
/// |             |             |             |
/// |             |             |             |
/// +-------------+-------------+-------------+
/// ```
/// 4 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |             |
/// |             |             |             |
/// |             |      1      |      3      |
/// +-------------+             |             |
/// |             |             |             |
/// |      4      |             |             |
/// |             |             |             |
/// +-------------+-------------+-------------+
/// ```
/// 4 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |             |
/// |             |             |             |
/// |             |      1      |      3      |
/// +-------------+             |             |
/// |             |             |             |
/// |      4      |             |             |
/// |             |             |             |
/// +-------------+-------------+-------------+
/// ```
/// 5 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |      1      |             |
/// +-------------+             +-------------+
/// |             |             |             |
/// |      4      |             |      5      |
/// |             |             |             |
/// +-------------+-------------+-------------+
/// ```
/// 6 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |             |             |
/// +------+------+      1      +-------------+
/// |      |      |             |             |
/// |  4   |  6   |             |      5      |
/// |      |      |             |             |
/// +------+------+-------------+-------------+
/// ```
/// 7 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |             |             |
/// +------+------+      1      +------+------+
/// |      |      |             |      |      |
/// |  4   |  6   |             |  5   |  7   |
/// |      |      |             |      |      |
/// +------+------+-------------+------+------+
/// ```
/// 8 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |      1      |             |
/// +------+------+             +------+------+
/// |      |  6   |             |      |      |
/// |  4   +------+             |  5   |  7   |
/// |      |  8   |             |      |      |
/// +------+------+-------------+------+------+
/// ```
/// 9 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |      1      |             |
/// +------+------+             +------+------+
/// |      |  6   |             |      |  7   |
/// |  4   +------+             |  5   +------+
/// |      |  8   |             |      |  9   |
/// +------+------+-------------+------+------+
/// ```
/// 10 windows
/// ```text
/// +-------------+-------------+-------------+
/// |             |             |             |
/// |      2      |             |      3      |
/// |             |             |             |
/// |             |      1      |             |
/// +------+------+             +------+------+
/// |      |  6   |             |      |  7   |
/// |  4   +---+--+             |  5   +------+
/// |      |  8|10|             |      |  9   |
/// +------+---+--+-------------+------+------+
/// ```
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let primary_width = match window_count {
        1 => workspace.width() as i32,
        2 => (workspace.width() as f32 / 2.0).floor() as i32,
        _ => (workspace.width() as f32 / 3.0).floor() as i32,
    };

    let primary_x = match window_count {
        1 => 0_i32,
        2 => (workspace.width() as f32 / 2.0).floor() as i32,
        _ => (workspace.width() as f32 / 3.0).floor() as i32,
    };

    let mut iter = windows.iter_mut();

    // build primary window
    if let Some(first) = iter.next() {
        first.set_height(workspace.height());
        first.set_width(primary_width);
        first.set_x(workspace.x() + primary_x);
        first.set_y(workspace.y());
    }

    // build secondary window if there's only two windows.
    if window_count < 3 {
        if let Some(second) = iter.next() {
            second.set_height(workspace.height());
            second.set_width(primary_width);
            second.set_x(workspace.x());
            second.set_y(workspace.y());
        }
        return;
    };

    // put even numbered windows in the left column and the odd windows in the right column.
    // Distribute them in the same way as the fibonacci layout, but start with rows instead of
    // columns.
    let remaining_windows = (iter.len() - 2) as usize;
    let half_remaining = (remaining_windows as f32 / 2.0).ceil() as usize;

    let mut left_windows = Vec::with_capacity(half_remaining);
    let mut right_windows = Vec::with_capacity(half_remaining);

    for (i, window) in iter.enumerate() {
        if i % 2 == 0 {
            left_windows.push(window);
        } else {
            right_windows.push(window);
        }
    }

    update_fibonacci(
        left_windows,
        workspace.x(),
        workspace.y(),
        workspace.height(),
        primary_width,
    );
    update_fibonacci(
        right_windows,
        workspace.x() + 2 * primary_width,
        workspace.y(),
        workspace.height(),
        primary_width,
    );
}

fn update_fibonacci(
    mut windows: Vec<&mut &mut &mut Window>,
    workspace_x: i32,
    workspace_y: i32,
    workspace_height: i32,
    workspace_width: i32,
) {
    let mut x = workspace_x;
    let mut y = workspace_y;
    let mut height = workspace_height as i32;
    let mut width = workspace_width as i32;
    let window_count = windows.len() as i32;

    for i in 0..window_count {
        if i % 2 != 0 {
            continue;
        }

        let half_width = (width as f32 / 2.0).floor() as i32;
        let half_height = (height as f32 / 2.0).floor() as i32;

        match window_count - 1 - i {
            0 => {
                setter(&mut windows[i as usize], height, width, x, y);
            }
            1 => {
                setter(&mut windows[i as usize], half_height, width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    half_height,
                    width,
                    x,
                    y + half_height,
                );
            }
            _ => {
                setter(&mut windows[i as usize], half_height, width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    half_height,
                    half_width,
                    x,
                    y + half_height,
                );

                x += half_width;
                y += half_height;
                width = half_width;
                height = half_height;
            }
        }
    }
}

fn setter(window: &mut Window, height: i32, width: i32, x: i32, y: i32) {
    window.set_height(height);
    window.set_width(width);
    window.set_x(x);
    window.set_y(y);
}
