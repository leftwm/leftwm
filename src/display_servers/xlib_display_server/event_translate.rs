use super::event_translate_client_message;
use super::event_translate_property_notify;
use super::DisplayEvent;
use super::XWrap;
use crate::models::Mode;
use crate::models::Window;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::models::WindowType;
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

            // window is deleted
            xlib::UnmapNotify => from_unmap_event(raw_event),

            // window is deleted
            xlib::DestroyNotify => {
                let event = xlib::XDestroyWindowEvent::from(raw_event);
                let h = WindowHandle::XlibHandle(event.window);
                Some(DisplayEvent::WindowDestroy(h))
            }

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
                Some(DisplayEvent::MouseCombo(event.state, event.button, h))
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
    //check that the window isn't requesting to be unmanaged
    let attr = xw.get_window_attrs(event.window).ok()?;
    if attr.override_redirect > 0 {
        return None;
    }
    //build the new window, and fill in info about it from xlib
    let name = xw.get_window_name(event.window);
    let pid = xw.get_window_pid(event.window);
    log::info!("PID: {:?} {:?}", pid, name);
    let mut w = Window::new(handle, name, pid);
    let trans = xw.get_transient_for(event.window);
    if let Some(hint) = xw.get_hint_sizing_as_xyhw(event.window) {
        hint.update_window_floating(&mut w);
        w.set_requested(hint)
    }
    w.set_states(xw.get_window_states(event.window));
    if w.floating() {
        if let Ok(geo) = xw.get_window_geometry(event.window) {
            log::debug!("geo: {geo:?}", geo = geo);
            geo.update_window_floating(&mut w);
        }
    }
    if let Some(trans) = trans {
        w.transient = Some(WindowHandle::XlibHandle(trans));
    }
    w.type_ = xw.get_window_type(event.window);
    let cursor = xw.get_cursor_point().unwrap_or_default();
    Some(DisplayEvent::WindowCreate(w, cursor.0, cursor.1))
}

fn from_unmap_event(raw_event: xlib::XEvent) -> Option<DisplayEvent> {
    let event = xlib::XUnmapEvent::from(raw_event);
    if event.send_event == xlib::False {
        None
    } else {
        let h = WindowHandle::XlibHandle(event.window);
        Some(DisplayEvent::WindowDestroy(h))
    }
}

fn from_enter_notify(xw: &XWrap, raw_event: xlib::XEvent) -> Option<DisplayEvent> {
    match &xw.mode {
        Mode::MovingWindow(_) | Mode::ResizingWindow(_) => return None,
        Mode::Normal => {}
    };
    let event = xlib::XEnterWindowEvent::from(raw_event);
    let crossing = xlib::XCrossingEvent::from(raw_event);
    if (crossing.mode != xlib::NotifyNormal || crossing.detail == xlib::NotifyInferior)
        && crossing.window != xw.get_default_root()
    {
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
    if window_type == WindowType::Normal {
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
    if window_type == WindowType::Dock {
        if let Some(dock_area) = xw.get_window_strut_array(event.window) {
            let dems = xw.screens_area_dimensions();
            if let Some(strut_xywh) = dock_area.as_xyhw(dems.0, dems.1) {
                change.strut = Some(strut_xywh.into())
            }
        }
    }
    Some(DisplayEvent::WindowChange(change))
}
