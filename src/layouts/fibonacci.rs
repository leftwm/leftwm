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
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let mut x = workspace.x();
    let mut y = workspace.y();
    let mut height = workspace.height() as i32;
    let mut width = workspace.width() as i32;
    let window_count = windows.len() as i32;

    for i in 0..window_count {
        if i % 2 != 0 {
            continue;
        }

        let half_width = (width as f32 / 2 as f32).floor() as i32;
        let half_height = (height as f32 / 2 as f32).floor() as i32;

        match window_count - 1 - i {
            0 => {
                setter(&mut windows[i as usize], height, width, x, y);
            }
            1 => {
                setter(&mut windows[i as usize], height, half_width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    height,
                    half_width,
                    x + half_width,
                    y,
                );
            }
            _ => {
                setter(&mut windows[i as usize], height, half_width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    half_height,
                    half_width,
                    x + half_width,
                    y,
                );

                x = x + half_width;
                y = y + half_height;
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
