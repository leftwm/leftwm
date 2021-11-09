use super::event_translate_client_message;
use super::event_translate_property_notify;
use super::DisplayEvent;
use super::XWrap;
use crate::models::Mode;
use crate::models::Window;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::models::WindowType;
use crate::models::Xyhw;
use crate::models::XyhwChange;
use x11_dl::xlib;

pub struct XEvent<'a>(pub &'a XWrap, pub xlib::XEvent);

impl<'a> From<XEvent<'a>> for Option<DisplayEvent> {
    fn from(x_event: XEvent) -> Self {
        let xw = x_event.0;
        let raw_event = x_event.1;

        match raw_event.get_type() {
            // new window is created
            xlib::MapRequest => from_map_request(raw_event, xw),

            // listen for keyboard changes
            xlib::MappingNotify => from_mapping_notify(raw_event, xw),

            // window is deleted
            xlib::UnmapNotify | xlib::DestroyNotify => Some(from_unmap_event(raw_event)),

            xlib::ClientMessage => {
                match &xw.mode {
                    Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
                    Mode::Normal => {}
                };
                let event = xlib::XClientMessageEvent::from(raw_event);
                event_translate_client_message::from_event(xw, event)
            }

            xlib::ButtonPress => {
                let event = xlib::XButtonPressedEvent::from(raw_event);
                let h = WindowHandle::XlibHandle(event.window);
                let mut mod_mask = event.state;
                mod_mask &= !(xlib::Mod2Mask | xlib::LockMask);
                xw.replay_click(mod_mask);
                Some(DisplayEvent::MouseCombo(mod_mask, event.button, h))
            }
            xlib::ButtonRelease => Some(DisplayEvent::ChangeToNormalMode),

            xlib::EnterNotify => from_enter_notify(xw, raw_event),

            xlib::PropertyNotify => {
                match &xw.mode {
                    Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
                    Mode::Normal => {}
                };
                let event = xlib::XPropertyEvent::from(raw_event);
                event_translate_property_notify::from_event(xw, event)
            }

            xlib::KeyPress => {
                let event = xlib::XKeyEvent::from(raw_event);
                let sym = xw.keycode_to_keysym(event.keycode);
                Some(DisplayEvent::KeyCombo(event.state, sym))
            }

            xlib::MotionNotify => Some(from_motion_notify(raw_event, xw)),

            xlib::ConfigureRequest => from_configure_request(xw, raw_event),

            _other => None,
        }
    }
}

fn from_map_request(raw_event: xlib::XEvent, xw: &XWrap) -> Option<DisplayEvent> {
    let event = xlib::XMapRequestEvent::from(raw_event);
    let handle = WindowHandle::XlibHandle(event.window);
    xw.subscribe_to_window_events(&handle);
    // Check that the window isn't requesting to be unmanaged
    let attr = xw.get_window_attrs(event.window).ok()?;
    if attr.override_redirect > 0 {
        return None;
    }
    // Gather info about the window from xlib.
    let name = xw.get_window_name(event.window);
    let pid = xw.get_window_pid(event.window);
    let type_ = xw.get_window_type(event.window);
    let states = xw.get_window_states(event.window);
    let actions = xw.get_window_actions_atoms(event.window);
    let mut can_resize = actions.contains(&xw.atoms.NetWMActionResize);
    let trans = xw.get_transient_for(event.window);
    let sizing_hint = xw.get_hint_sizing_as_xyhw(event.window);

    // Build the new window, and fill in info about it.
    let mut w = Window::new(handle, name, pid);
    w.type_ = type_;
    w.set_states(states);
    if let Some(trans) = trans {
        w.transient = Some(WindowHandle::XlibHandle(trans));
    }
    if let Some(hint) = sizing_hint {
        if hint.minw.is_none() || hint.minh.is_none() || hint.maxw.is_none() || hint.maxh.is_none()
        {
            can_resize = true;
        } else {
            can_resize = can_resize || hint.minw != hint.maxw || hint.minh != hint.maxh;
        }
        hint.update_window_floating(&mut w);
        w.set_requested(hint);
    }
    w.can_resize = can_resize;
    // Is this needed? Made it so it doens't overwrite prior sizing.
    if w.floating() && sizing_hint.is_none() {
        if let Ok(geo) = xw.get_window_geometry(event.window) {
            geo.update_window_floating(&mut w);
        }
    }

    let cursor = xw.get_cursor_point().unwrap_or_default();
    Some(DisplayEvent::WindowCreate(w, cursor.0, cursor.1))
}

fn from_mapping_notify(raw_event: xlib::XEvent, xw: &XWrap) -> Option<DisplayEvent> {
    let mut event = xlib::XMappingEvent::from(raw_event);
    if event.request == xlib::MappingModifier || event.request == xlib::MappingKeyboard {
        // refresh keyboard
        log::info!("Updating keyboard");
        xw.refresh_keyboard(&mut event).ok()?;

        // SoftReload keybinds
        Some(DisplayEvent::KeyGrabReload)
    } else {
        None
    }
}

fn from_unmap_event(raw_event: xlib::XEvent) -> DisplayEvent {
    let event = xlib::XUnmapEvent::from(raw_event);
    let h = WindowHandle::XlibHandle(event.window);
    DisplayEvent::WindowDestroy(h)
}

fn from_enter_notify(xw: &XWrap, raw_event: xlib::XEvent) -> Option<DisplayEvent> {
    match &xw.mode {
        Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
        Mode::Normal => {}
    };
    let event = xlib::XEnterWindowEvent::from(raw_event);
    let crossing = xlib::XCrossingEvent::from(raw_event);
    if crossing.detail == xlib::NotifyInferior && crossing.window != xw.get_default_root() {
        return None;
    }
    let h = WindowHandle::XlibHandle(event.window);
    Some(DisplayEvent::MouseEnteredWindow(h))
}

fn from_motion_notify(raw_event: xlib::XEvent, xw: &XWrap) -> DisplayEvent {
    let event = xlib::XMotionEvent::from(raw_event);
    let event_h = WindowHandle::XlibHandle(event.window);
    let offset_x = event.x_root - xw.mode_origin.0;
    let offset_y = event.y_root - xw.mode_origin.1;
    match &xw.mode {
        Mode::Normal => DisplayEvent::Movement(event_h, event.x_root, event.y_root),
        Mode::MovingWindow(h) => DisplayEvent::MoveWindow(*h, event.time, offset_x, offset_y),
        Mode::ResizingWindow(h) => DisplayEvent::ResizeWindow(*h, event.time, offset_x, offset_y),
    }
}

fn from_configure_request(xw: &XWrap, raw_event: xlib::XEvent) -> Option<DisplayEvent> {
    match &xw.mode {
        Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
        Mode::Normal => {}
    };
    let event = xlib::XConfigureRequestEvent::from(raw_event);
    let window_type = xw.get_window_type(event.window);
    if window_type == WindowType::Normal || window_type == WindowType::Dialog {
        return None;
    }
    let handle = WindowHandle::XlibHandle(event.window);
    let mut change = WindowChange::new(handle);
    let xyhw = XyhwChange {
        w: Some(event.width),
        h: Some(event.height),
        x: Some(event.x),
        y: Some(event.y),
        ..XyhwChange::default()
    };
    change.floating = Some(xyhw);
    if window_type == WindowType::Dock || window_type == WindowType::Desktop {
        if let Some(dock_area) = xw.get_window_strut_array(event.window) {
            let dems = xw.get_screens_area_dimensions();
            let screen = xw
                .get_screens()
                .iter()
                .find(|s| s.contains_dock_area(dock_area, dems))?
                .clone();

            if let Some(xyhw) = dock_area.as_xyhw(dems.0, dems.1, &screen) {
                change.strut = Some(xyhw.into());
            }
        } else if let Ok(geo) = xw.get_window_geometry(event.window) {
            let mut xyhw = Xyhw::default();
            geo.update(&mut xyhw);
            change.strut = Some(xyhw.into());
        }
    }
    Some(DisplayEvent::WindowChange(change))
}
