use super::event_translate_client_message;
use super::event_translate_property_notify;
use super::DisplayEvent;
use super::XWrap;
use crate::models;
use crate::models::Mode;
use crate::models::Window;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::models::WindowType;
use crate::models::Xyhw;
use crate::models::XyhwChange;
use models::FocusBehaviour;
use std::os::raw::c_ulong;
use x11_dl::xlib;

pub struct XEvent<'a>(pub &'a mut XWrap, pub xlib::XEvent);

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
                Some(DisplayEvent::MouseCombo(mod_mask, event.button, h))
            }
            xlib::ButtonRelease => match xw.mode {
                models::Mode::Normal => None,
                _ => Some(DisplayEvent::ChangeToNormalMode),
            },

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

            xlib::MotionNotify => from_motion_notify(raw_event, xw),

            xlib::ConfigureRequest => from_configure_request(xw, raw_event),

            _other => None,
        }
    }
}

fn from_map_request(raw_event: xlib::XEvent, xw: &mut XWrap) -> Option<DisplayEvent> {
    let event = xlib::XMapRequestEvent::from(raw_event);
    let handle = WindowHandle::XlibHandle(event.window);
    xw.subscribe_to_window_events(&handle);
    // Check that the window isn't requesting to be unmanaged
    let attrs = match xw.get_window_attrs(event.window) {
        Ok(attr) if attr.override_redirect == 0 => attr,
        _ => return None,
    };
    // Gather info about the window from xlib.
    let name = xw.get_window_name(event.window);
    let legacy_name = xw.get_window_legacy_name(event.window);
    let class = xw.get_window_class(event.window);
    log::info!("WM class: {:?}", class);
    let pid = xw.get_window_pid(event.window);
    let r#type = xw.get_window_type(event.window);
    let states = xw.get_window_states(event.window);
    let actions = xw.get_window_actions_atoms(event.window);
    let mut can_resize = actions.contains(&xw.atoms.NetWMActionResize);
    let trans = xw.get_transient_for(event.window);
    let sizing_hint = xw.get_hint_sizing_as_xyhw(event.window);
    let wm_hint = xw.get_wmhints(event.window);

    // Build the new window, and fill in info about it.
    let mut w = Window::new(handle, name, pid);
    w.wm_class = class;
    w.legacy_name = legacy_name;
    w.r#type = r#type.clone();
    w.set_states(states);
    if let Some(trans) = trans {
        w.transient = Some(WindowHandle::XlibHandle(trans));
    }
    // Initialise the windows floating with the pre-mapped settings.
    let xyhw = XyhwChange {
        x: Some(attrs.x),
        y: Some(attrs.y),
        w: Some(attrs.width),
        h: Some(attrs.height),
        ..XyhwChange::default()
    };
    xyhw.update_window_floating(&mut w);
    let mut requested = Xyhw::default();
    xyhw.update(&mut requested);
    if let Some(mut hint) = sizing_hint {
        // Ignore this for now for non-splashes as it causes issues, e.g. mintstick is non-resizable but is too
        // small, issue #614: https://github.com/leftwm/leftwm/issues/614.
        can_resize = match (r#type, hint.minw, hint.minh, hint.maxw, hint.maxh) {
            (
                WindowType::Splash,
                Some(min_width),
                Some(min_height),
                Some(max_width),
                Some(max_height),
            ) => can_resize || min_width != max_width || min_height != max_height,
            _ => true,
        };
        // Use the pre-mapped sizes if they are bigger.
        hint.w = std::cmp::max(xyhw.w, hint.w);
        hint.h = std::cmp::max(xyhw.h, hint.h);
        hint.update_window_floating(&mut w);
        hint.update(&mut requested);
    }
    w.requested = Some(requested);
    w.can_resize = can_resize;
    if let Some(hint) = wm_hint {
        w.never_focus = hint.flags & xlib::InputHint != 0 && hint.input == 0;
    }
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
        log::debug!("Updating keyboard");
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
        Mode::Normal if xw.focus_behaviour != FocusBehaviour::Sloppy => return None,
        Mode::Normal => {}
    };
    let event = xlib::XEnterWindowEvent::from(raw_event);
    let crossing = xlib::XCrossingEvent::from(raw_event);
    if crossing.detail == xlib::NotifyInferior && crossing.window != xw.get_default_root() {
        return None;
    }
    let h = WindowHandle::XlibHandle(event.window);
    Some(DisplayEvent::WindowTakeFocus(h))
}

fn from_motion_notify(raw_event: xlib::XEvent, xw: &mut XWrap) -> Option<DisplayEvent> {
    let event = xlib::XMotionEvent::from(raw_event);
    // Limit motion events to current refresh rate.

    if xw.refresh_rate as c_ulong > 0
        && event.time - xw.motion_event_limiter > (1000 / xw.refresh_rate as c_ulong)
    {
        xw.motion_event_limiter = event.time;
        let event_h = WindowHandle::XlibHandle(event.window);
        let offset_x = event.x_root - xw.mode_origin.0;
        let offset_y = event.y_root - xw.mode_origin.1;
        let display_event = match &xw.mode {
            Mode::MovingWindow(h) => DisplayEvent::MoveWindow(*h, offset_x, offset_y),
            Mode::ResizingWindow(h) => DisplayEvent::ResizeWindow(*h, offset_x, offset_y),
            Mode::Normal if xw.focus_behaviour == FocusBehaviour::Sloppy => {
                DisplayEvent::Movement(event_h, event.x_root, event.y_root)
            }
            Mode::Normal => return None,
        };
        return Some(display_event);
    }

    None
}

fn from_configure_request(xw: &XWrap, raw_event: xlib::XEvent) -> Option<DisplayEvent> {
    match &xw.mode {
        Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
        Mode::Normal => {}
    };
    let event = xlib::XConfigureRequestEvent::from(raw_event);
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
    let handle = WindowHandle::XlibHandle(event.window);
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
