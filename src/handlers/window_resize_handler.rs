use super::{Manager, Window, WindowHandle};

pub fn process(manager: &mut Manager, handle: &WindowHandle, offset_w: i32, offset_h: i32) -> bool {
    for w in &mut manager.windows {
        if &w.handle == handle {
            process_window(w, offset_w, offset_h);
            return true;
        }
    }
    false
}

fn process_window(window: &mut Window, offset_w: i32, offset_h: i32) {
    window.set_floating(true);
    let mut offset = window.get_floating_offsets().unwrap_or_default();
    let start = window.start_loc.unwrap_or_default();
    //offset.clear_minmax();
    offset.set_w(start.w() + offset_w);
    offset.set_h(start.h() + offset_h);
    window.set_floating_offsets(Some(offset));
}
