use crate::models::Window;
use crate::models::Workspace;

/// Fibonacci layout, which divides the workspace in subsequent halves and assignes them to the windows
/// ```text
/// +-----------+-----------+
/// |           |           |
/// |           |     2     |
/// |           |           |
/// |     1     +-----+-----+
/// |           |     |  4  |
/// |           |  3  +--+--+
/// |           |     | 5|-.|
/// +-----------+-----+-----+
/// ```
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut Window>) {
    let mut x = workspace.x();
    let mut y = workspace.y();
    let mut height = workspace.height() as i32;
    let mut width = workspace.width() as i32;
    let window_count = windows.len();

    for i in 0..window_count {
        if i % 2 != 0 {
            continue;
        }

        let half_width = (width as f32 / 2.0).floor() as i32;
        let half_height = (height as f32 / 2.0).floor() as i32;
        let (main_x, alt_x);
        if workspace.flipped_horizontal() {
            main_x = x + half_width;
            alt_x = x;
        } else {
            main_x = x;
            alt_x = x + half_width;
        }
        let (new_y, alt_y);
        if workspace.flipped_vertical() {
            new_y = y;
            alt_y = y + half_height;
        } else {
            new_y = y + half_height;
            alt_y = y;
        }
        match window_count - i {
            1 => setter(&mut windows[i], height, width, x, y),
            2 => {
                setter(&mut windows[i], height, half_width, main_x, y);
                setter(&mut windows[i + 1], height, half_width, alt_x, y);
            }
            _ => {
                setter(&mut windows[i], height, half_width, main_x, y);
                setter(&mut windows[i + 1], half_height, half_width, alt_x, alt_y);

                x = alt_x;
                y = new_y;
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
