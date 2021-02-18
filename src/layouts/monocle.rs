use crate::models::Window;
use crate::models::Workspace;
// use crate::models::WindowState;

/// Layout which gives only one window with the full desktop realestate. A monocle mode.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let mut iter = windows.iter_mut();

    //maximize primary window
    {
        if let Some(monowin) = iter.next() {
            monowin.set_height(workspace.height());
            monowin.set_width(workspace.width());
            monowin.set_x(workspace.x());
            monowin.set_y(workspace.y());

            monowin.set_visible(true);
        }
    }

    //hide all other windows
    {
        if window_count > 1 {
            for w in iter {
                w.set_height(workspace.height());
                w.set_width(workspace.width());
                w.set_x(workspace.x());
                w.set_y(workspace.y());

                w.set_visible(false);
            }
        }
    }
}
