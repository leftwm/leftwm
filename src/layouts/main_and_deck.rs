use crate::models::Window;
use crate::models::Workspace;
// use crate::models::WindowState;

/// Layout which gives only one window with the full desktop realestate. A monocle mode.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 100.0 * workspace.main_width()).floor() as i32,
    };

    //Display main and second window
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(width);
            first.set_x(workspace.x());
            first.set_y(workspace.y());

            first.set_visible(true);
        }
        if let Some(second) = iter.next() {
            second.set_height(workspace.height());
            second.set_width(workspace.width() - width);
            second.set_x(workspace.x() + width);
            second.set_y(workspace.y());

            second.set_visible(true);
        }
    }
    //Hide the other windows behind the second
    {
        if window_count > 2 {
            for w in iter {
                w.set_height(workspace.height());
                w.set_width(workspace.width() - width);
                w.set_x(workspace.x() + width);
                w.set_y(workspace.y());

                w.set_visible(false);
            }
        }
    }
}
