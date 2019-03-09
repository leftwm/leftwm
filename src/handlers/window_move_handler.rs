use super::*;

pub fn process(manager: &mut Manager, handle: &WindowHandle, offset_x: i32, offset_y: i32) -> bool {
    for w in &mut manager.windows {
        if &w.handle == handle {
            process_window(w, offset_x, offset_y);
            return true;
        }
    }
    false
}

fn process_window(window: &mut Window, offset_x: i32, offset_y: i32) {
    //println!("MOVING_WINDOW: {:?}", &window.handle);
    window.floating = true;
    if window.floating_loc.is_none() {
        window.floating_loc = Some((window.x(), window.y()));
    }
    if window.start_loc.is_none() {
        window.start_loc = window.floating_loc.clone();
    }

    //they must have a value, it is safe to unwrap
    let floating = &mut window.floating_loc.unwrap();
    let starting = &window.start_loc.unwrap();

    floating.0 = starting.0 + offset_x;
    floating.1 = starting.1 + offset_y;
    window.floating_loc = Some(floating.clone());
}
