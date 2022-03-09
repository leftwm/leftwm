use super::{
    event_translate_client_message, event_translate_property_notify, xwrap::WITHDRAWN_STATE,
    DisplayEvent, XWrap,
};
use crate::models::{Mode, WindowChange, WindowType, XyhwChange};
use std::os::raw::c_ulong;
use x11_dl::xlib;

pub struct XEvent<'a>(pub &'a mut XWrap, pub xlib::XEvent);

impl<'a> From<XEvent<'a>> for Option<DisplayEvent> {
    fn from(x_event: XEvent) -> Self {
        let raw_event = x_event.1;
        let normal_mode = x_event.0.mode == Mode::Normal;
        let sloppy_behaviour = x_event.0.focus_behaviour.is_sloppy();

        match raw_event.get_type() {
            // New window is mapped.
            xlib::MapRequest => from_map_request(x_event),
            // Window is unmapped.
            xlib::UnmapNotify => from_unmap_event(x_event),
            // Window is destroyed.
            xlib::DestroyNotify => from_destroy_notify(x_event),
            // Window client message.
            xlib::ClientMessage if normal_mode => from_client_message(&x_event),
            // Window property notify.
            xlib::PropertyNotify if normal_mode => from_property_notify(&x_event),
            // Window configure request.
            xlib::ConfigureRequest if normal_mode => from_configure_request(x_event),
            // Mouse entered notify.
            xlib::EnterNotify if normal_mode && sloppy_behaviour => from_enter_notify(&x_event),
            // Mouse motion notify.
            xlib::MotionNotify => from_motion_notify(x_event),
            // Mouse button pressed.
            xlib::ButtonPress => Some(from_button_press(raw_event)),
            // Mouse button released.
            xlib::ButtonRelease if !normal_mode => Some(from_button_release(x_event)),
            // Keyboard key pressed.
            xlib::KeyPress => Some(from_key_press(x_event)),
            // Listen for keyboard changes.
            xlib::MappingNotify => from_mapping_notify(x_event),
            _other => None,
        }
    }
}

fn from_map_request(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let event = xlib::XMapRequestEvent::from(x_event.1);
    xw.setup_window(event.window)
}

fn from_unmap_event(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let event = xlib::XUnmapEvent::from(x_event.1);
    if xw.managed_windows.contains(&event.window) {
        // Set WM_STATE to withdrawn state.
        xw.set_wm_states(event.window, &[WITHDRAWN_STATE]);
        if event.send_event == xlib::False {
            let h = event.window.into();
            xw.teardown_managed_window(&h);
            return Some(DisplayEvent::WindowDestroy(h));
        }
    }
    None
}

fn from_destroy_notify(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let event = xlib::XDestroyWindowEvent::from(x_event.1);
    if xw.managed_windows.contains(&event.window) {
        let h = event.window.into();
        xw.teardown_managed_window(&h);
        return Some(DisplayEvent::WindowDestroy(h));
    }
    None
}

fn from_client_message(x_event: &XEvent) -> Option<DisplayEvent> {
    let event = xlib::XClientMessageEvent::from(x_event.1);
    event_translate_client_message::from_event(x_event.0, event)
}

fn from_property_notify(x_event: &XEvent) -> Option<DisplayEvent> {
    let event = xlib::XPropertyEvent::from(x_event.1);
    event_translate_property_notify::from_event(x_event.0, event)
}

fn from_configure_request(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let event = xlib::XConfigureRequestEvent::from(x_event.1);
    // If the window is not mapped, configure it.
    if !xw.managed_windows.contains(&event.window) {
        let window_changes = xlib::XWindowChanges {
            x: event.x,
            y: event.y,
            width: event.width,
            height: event.height,
            border_width: event.border_width,
            sibling: event.above,
            stack_mode: event.detail,
        };
        let unlock = xlib::CWX
            | xlib::CWY
            | xlib::CWWidth
            | xlib::CWHeight
            | xlib::CWBorderWidth
            | xlib::CWSibling
            | xlib::CWStackMode;
        xw.set_window_config(event.window, window_changes, u32::from(unlock));
        xw.move_resize_window(
            event.window,
            event.x,
            event.y,
            event.width as u32,
            event.height as u32,
        );
        return None;
    }
    let window_type = xw.get_window_type(event.window);
    let trans = xw.get_transient_for(event.window);
    let handle = event.window.into();
    if window_type == WindowType::Normal && trans.is_none() {
        return Some(DisplayEvent::ConfigureXlibWindow(handle));
    }
    let mut change = WindowChange::new(handle);
    let xyhw = match window_type {
        // We want to handle the window positioning when it is a dialog or a normal window with a
        // parent.
        WindowType::Dialog | WindowType::Normal => XyhwChange {
            w: Some(event.width),
            h: Some(event.height),
            ..XyhwChange::default()
        },
        _ => XyhwChange {
            w: Some(event.width),
            h: Some(event.height),
            x: Some(event.x),
            y: Some(event.y),
            ..XyhwChange::default()
        },
    };
    change.floating = Some(xyhw);
    Some(DisplayEvent::WindowChange(change))
}

fn from_enter_notify(x_event: &XEvent) -> Option<DisplayEvent> {
    let event = xlib::XCrossingEvent::from(x_event.1);
    if (event.mode != xlib::NotifyNormal || event.detail == xlib::NotifyInferior)
        && event.window != x_event.0.get_default_root()
    {
        return None;
    }

    let h = event.window.into();
    Some(DisplayEvent::WindowTakeFocus(h))
}

fn from_motion_notify(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let event = xlib::XMotionEvent::from(x_event.1);

    // Limit motion events to current refresh rate.
    if xw.refresh_rate as c_ulong > 0
        && event.time - xw.motion_event_limiter > (1000 / xw.refresh_rate as c_ulong)
    {
        xw.motion_event_limiter = event.time;
        let event_h = event.window.into();
        let offset_x = event.x_root - xw.mode_origin.0;
        let offset_y = event.y_root - xw.mode_origin.1;
        let display_event = match xw.mode {
            Mode::ReadyToMove(h) => {
                xw.set_mode(Mode::MovingWindow(h));
                DisplayEvent::MoveWindow(h, offset_x, offset_y)
            }
            Mode::MovingWindow(h) => DisplayEvent::MoveWindow(h, offset_x, offset_y),
            Mode::ReadyToResize(h) => {
                xw.set_mode(Mode::ResizingWindow(h));
                DisplayEvent::ResizeWindow(h, offset_x, offset_y)
            }
            Mode::ResizingWindow(h) => DisplayEvent::ResizeWindow(h, offset_x, offset_y),
            Mode::Normal if xw.focus_behaviour.is_sloppy() => {
                DisplayEvent::Movement(event_h, event.x_root, event.y_root)
            }
            Mode::Normal => return None,
        };
        return Some(display_event);
    }

    None
}

fn from_button_press(raw_event: xlib::XEvent) -> DisplayEvent {
    let event = xlib::XButtonPressedEvent::from(raw_event);
    let h = event.window.into();
    let mut mod_mask = event.state;
    mod_mask &= !(xlib::Mod2Mask | xlib::LockMask);
    DisplayEvent::MouseCombo(mod_mask, event.button, h, event.x, event.y)
}

fn from_button_release(x_event: XEvent) -> DisplayEvent {
    let xw = x_event.0;
    xw.set_mode(Mode::Normal);
    DisplayEvent::ChangeToNormalMode
}

fn from_key_press(x_event: XEvent) -> DisplayEvent {
    let xw = x_event.0;
    let event = xlib::XKeyEvent::from(x_event.1);
    let sym = xw.keycode_to_keysym(event.keycode);
    DisplayEvent::KeyCombo(event.state, sym)
}

fn from_mapping_notify(x_event: XEvent) -> Option<DisplayEvent> {
    let xw = x_event.0;
    let mut event = xlib::XMappingEvent::from(x_event.1);
    if event.request == xlib::MappingModifier || event.request == xlib::MappingKeyboard {
        // Refresh keyboard.
        log::debug!("Updating keyboard");
        xw.refresh_keyboard(&mut event).ok()?;

        // SoftReload keybinds.
        Some(DisplayEvent::KeyGrabReload)
    } else {
        None
    }
}
