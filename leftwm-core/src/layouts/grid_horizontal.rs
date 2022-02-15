use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into N columns, and then splits each column into rows.
/// Example arrangement (4 windows):
/// ```text
/// +---+---+
/// |   |   |
/// +---+---+
/// |   |   |
/// +---+---+
/// ```
/// or with 8 windows:
/// ```text
/// +---+---+---+
/// |   |   |   |
/// |   +---+---+
/// +---+   |   |
/// |   +---+---+
/// |   |   |   |
/// +---+---+---+
/// ```
pub fn update(workspace: &Workspace, tag: &Tag, windows: &mut [&mut Window]) {
    let window_count = windows.len() as i32;

    // choose the number of columns so that we get close to an even NxN grid.
    let num_cols = (window_count as f32).sqrt().ceil() as i32;

    let mut iter = windows.iter_mut().enumerate().peekable();
    for col in 0..num_cols {
        let iter_peek = iter.peek().map(|x| x.0).unwrap_or_default() as i32;
        let remaining_windows = window_count - iter_peek;
        let remaining_columns = num_cols - col;
        let num_rows_in_this_col = remaining_windows / remaining_columns;

        let win_height = workspace.height() / num_rows_in_this_col;
        let win_width = workspace.width_limited(num_cols as usize) / num_cols;

        let pos_x = if tag.flipped_horizontal {
            num_cols - col - 1
        } else {
            col
        };

        for row in 0..num_rows_in_this_col {
            let (_idx, win) = match iter.next() {
                Some(x) => x,
                None => return,
            };
            win.set_height(win_height);
            win.set_width(win_width);

            let pos_y = if tag.flipped_vertical {
                num_rows_in_this_col - row - 1
            } else {
                row
            };

            win.set_x(workspace.x_limited(num_cols as usize) + win_width * pos_x);
            win.set_y(workspace.y() + win_height * pos_y);
        }
    }
}
