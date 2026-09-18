use super::{Manager, Window, WindowHandle};
use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::Handle;
use crate::models::ResizeCorner;

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    pub fn window_resize_handler(
        &mut self,
        handle: &WindowHandle<H>,
        offset_w: i32,
        offset_h: i32,
    ) -> bool {
        let corner = self.state.resize_corner;
        if let Some(w) = self.state.windows.iter_mut().find(|w| &w.handle == handle) {
            process_window(w, offset_w, offset_h, corner);
            return true;
        }
        false
    }
}

fn process_window<H: Handle>(
    window: &mut Window<H>,
    offset_w: i32,
    offset_h: i32,
    corner: ResizeCorner,
) {
    window.set_floating(true);
    let mut offset = window.get_floating_offsets().unwrap_or_default();
    let start = window.start_loc.unwrap_or_default();
    // offset.clear_minmax();

    // An anchored edge follows the pointer while the opposite one stays put,
    // so the size changes by as much as the origin does, in the opposite
    // direction. Shrinking is capped at the minimum size the window would be
    // clamped to anyway: past that point only the anchored edge would keep
    // moving, dragging the window across the screen instead of resizing it.
    if corner.anchors_left() {
        let delta = offset_w.min(shrink_limit(
            window.normal.w() + start.w(),
            window.border,
            min_width(window),
        ));
        offset.set_x(start.x() + delta);
        offset.set_w(start.w() - delta);
    } else {
        offset.set_w(start.w() + offset_w);
    }

    if corner.anchors_top() {
        let delta = offset_h.min(shrink_limit(
            window.normal.h() + start.h(),
            window.border,
            min_height(window),
        ));
        offset.set_y(start.y() + delta);
        offset.set_h(start.h() - delta);
    } else {
        offset.set_h(start.h() + offset_h);
    }

    window.set_floating_offsets(Some(offset));
}

/// How far an anchored edge may travel inwards before the window reaches its
/// minimum size. Mirrors the clamping in [`Window::width`] and
/// [`Window::height`].
const fn shrink_limit(frame: i32, border: i32, minimum: i32) -> i32 {
    frame - border * 2 - minimum
}

fn min_width<H: Handle>(window: &Window<H>) -> i32 {
    match window.requested {
        Some(requested) if requested.minw() > 0 => requested.minw(),
        _ => 100,
    }
}

fn min_height<H: Handle>(window: &Window<H>) -> i32 {
    match window.requested {
        Some(requested) if requested.minh() > 0 => requested.minh(),
        _ => 100,
    }
}
