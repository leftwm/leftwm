use super::event_translate_client_message;
use super::event_translate_property_notify;
use super::DisplayEvent;
use super::XWrap;
use crate::models::Mode;
use crate::models::Window;
use crate::models::WindowHandle;
use x11_dl::xlib;

pub struct XEvent<'a>(pub &'a XWrap, pub xlib::XEvent);

impl<'a> From<XEvent<'a>> for Option<DisplayEvent> {
    fn from(xevent: XEvent) -> Self {
        let xw = xevent.0;
        let raw_event = xevent.1;

        match raw_event.get_type() {
            // new window is created
            xlib::MapRequest => {
                let event = xlib::XMapRequestEvent::from(raw_event);
                //println!("MapRequest {:?}", event);
                let handle = WindowHandle::XlibHandle(event.window);
                //first subscribe to window events so we don't miss any
                xw.subscribe_to_window_events(&handle);

                match xw.get_window_attrs(event.window) {
                    Ok(attr) if attr.override_redirect > 0 => None,
                    Ok(_attr) => {
                        let name = xw.get_window_name(event.window);
                        let mut w = Window::new(handle, name);
                        let trans = xw.get_transient_for(event.window);

                        if let Some(hint) = xw.get_hint_sizing_as_xyhw(event.window) {
                            hint.update_window(&mut w);
                        }

                        if w.floating.is_none() {
                            if let Ok(geo) = xw.get_window_geometry(event.window) {
                                geo.update_window(&mut w);
                            }
                        }

                        if let Some(trans) = trans {
                            w.transient = Some(WindowHandle::XlibHandle(trans));
                        }
                        w.type_ = xw.get_window_type(event.window);

                        Some(DisplayEvent::WindowCreate(w))
                    }
                    Err(_) => None,
                }
            }

            // window is deleted
            xlib::UnmapNotify => {
                let event = xlib::XUnmapEvent::from(raw_event);
                //println!("UnmapNotify {:?}", event);
                if event.send_event != xlib::False {
                    let h = WindowHandle::XlibHandle(event.window);
                    Some(DisplayEvent::WindowDestroy(h))
                } else {
                    None
                }
            }

            // window is deleted
            xlib::DestroyNotify => {
                let event = xlib::XDestroyWindowEvent::from(raw_event);
                let h = WindowHandle::XlibHandle(event.window);
                Some(DisplayEvent::WindowDestroy(h))
            }

            xlib::ClientMessage => {
                let event = xlib::XClientMessageEvent::from(raw_event);
                //println!("ClientMessage {:?}", event);
                event_translate_client_message::from_event(xw, event)
            }

            xlib::ButtonPress => {
                let event = xlib::XButtonPressedEvent::from(raw_event);
                let h = WindowHandle::XlibHandle(event.window);
                Some(DisplayEvent::MouseCombo(event.state, event.button, h))
            }
            xlib::ButtonRelease => Some(DisplayEvent::ChangeToNormalMode),

            xlib::EnterNotify => {
                let event = xlib::XEnterWindowEvent::from(raw_event);
                //println!("EnterNotify {:?}", event);
                let crossing = xlib::XCrossingEvent::from(raw_event);
                if (crossing.mode != xlib::NotifyNormal || crossing.detail == xlib::NotifyInferior)
                    && crossing.window != xw.get_default_root()
                {
                    return None;
                }
                let h = WindowHandle::XlibHandle(event.window);
                let mouse_loc = xw.get_pointer_location();

                mouse_loc.and_then(|loc| Some(DisplayEvent::FocusedWindow(h, loc.0, loc.1)))
            }

            xlib::PropertyNotify => {
                let event = xlib::XPropertyEvent::from(raw_event);
                //println!("PropertyNotify {:?}", event);
                event_translate_property_notify::from_event(xw, event)
            }

            xlib::KeyPress => {
                let event = xlib::XKeyEvent::from(raw_event);
                //println!("KeyPress {:?}", event);
                let sym = xw.keycode_to_keysym(event.keycode);
                Some(DisplayEvent::KeyCombo(event.state, sym))
            }

            xlib::MotionNotify => {
                let event = xlib::XMotionEvent::from(raw_event);
                //println!("MotionNotify {:?}", event);
                let event_h = WindowHandle::XlibHandle(event.window);
                let offset_x = event.x_root - xw.mode_origin.0;
                let offset_y = event.y_root - xw.mode_origin.1;
                match &xw.mode {
                    Mode::NormalMode => {
                        Some(DisplayEvent::Movement(event_h, event.x_root, event.y_root))
                    }
                    Mode::MovingWindow(h) => {
                        Some(DisplayEvent::MoveWindow(h.clone(), offset_x, offset_y))
                    }
                    Mode::ResizingWindow(h) => {
                        Some(DisplayEvent::ResizeWindow(h.clone(), offset_x, offset_y))
                    }
                }
            }
            xlib::FocusIn => {
                //let event = xlib::XFocusChangeEvent::from(raw_event);
                //println!("FocusIn {:?}", event);
                None
            }

            _other => {
                //println!("Other {:?}", other);
                None
            }
        }
    }
}
