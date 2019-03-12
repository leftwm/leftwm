use super::*;

pub fn process(manager: &mut Manager, handle: &WindowHandle, offset_x: i32, offset_y: i32) -> bool {
    for w in &mut manager.windows {
        if &w.handle == handle {
            process_window(w, offset_x, offset_y);
            return true;
        }
    }
    true
}

fn process_window(window: &mut Window, offset_x: i32, offset_y: i32) {
    window.set_floating(true);
    if window.floating_size.is_none() {
        window.floating_size = Some((window.width(), window.height()));
    }
    if window.start_loc.is_none() {
        window.start_loc = window.floating_size.clone();
    }

    //they must have a value, it is safe to unwrap
    let floating = &mut window.floating_size.unwrap();
    let starting = &window.start_loc.unwrap();

    floating.0 = starting.0 + offset_x;
    floating.1 = starting.1 + offset_y;

    //set min sizes for the floating window
    if floating.0 < 100 {
        floating.0 = 100
    }
    if floating.1 < 100 {
        floating.1 = 100
    }

    window.floating_size = Some(floating.clone());
}
