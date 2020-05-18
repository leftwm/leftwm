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
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len() as i32;

    // choose the number of columns so that we get close to an even NxN grid.
    let num_cols = (window_count as f32).sqrt().ceil() as i32;

    let mut iter = windows.iter_mut().enumerate().peekable();
    for col in 0..num_cols {
        let remaining_windows = window_count - iter.peek().unwrap().0 as i32;
        let remaining_columns = num_cols - col;
        let num_rows_in_this_col = remaining_windows / remaining_columns;

        let win_height = workspace.height() / num_rows_in_this_col;
        let win_width = workspace.width() / num_cols;

        for row in 0..num_rows_in_this_col {
            let (_idx, win) = iter.next().unwrap();
            win.set_height(win_height);
            win.set_width(win_width);
            win.set_x(workspace.x() + win_width * col);
            win.set_y(workspace.y() + win_height * row);
        }
    }
}
