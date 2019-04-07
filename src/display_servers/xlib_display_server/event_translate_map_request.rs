use super::DisplayEvent;
use super::XWrap;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::XYHW;
use crate::utils::logging::*;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XMapRequestEvent) -> Option<DisplayEvent> {
    log_xevent(&format!("MapRequest {:?}", event));
    let handle = WindowHandle::XlibHandle(event.window);
    //first subscribe to window events so we don't miss any
    xw.subscribe_to_window_events(&handle);

    match xw.get_window_attrs(event.window) {
        Ok(attr) if attr.override_redirect > 0 => None,
        Ok(attr) => {
            let name = xw.get_window_name(event.window);
            let mut w = Window::new(handle, name);
            w.floating = xw.get_hint_sizing_as_xyhw(event.window);
            let trans = xw.get_transient_for(event.window);
            if let Some(trans) = trans {
                w.transient = Some(WindowHandle::XlibHandle(trans));
                let _ = update_if_transient(xw, &mut w, trans);
            }
            w.type_ = xw.get_window_type(event.window);
            Some(DisplayEvent::WindowCreate(w))
        }
        Err(_) => None,
    }
}

fn update_if_transient(xw: &XWrap, window: &mut Window, trans: xlib::Window) -> Result<(), ()> {
    log_info("TRANS", "in update_if_transient");

    //get a loc/sizing for the new transient window. if we can't default to something so we can fill in the reset
    let mut xyhw = window.floating.unwrap_or_default();

    let parent_geo = xw.get_window_geometry(trans)?;
    let parent_attrs = xw.get_window_attrs(trans)?;

    log_info("TRANS GEO", &format!("{:?}", parent_geo));
    log_info("TRANS ATTRS", &format!("{:?}", parent_attrs));

    //first we need to make sure the trans has a height/width
    if xyhw.h == 0 || xyhw.w == 0 {
        xyhw.h = parent_geo.h / 2;
        xyhw.w = parent_geo.w / 2;
    }

    log_info("TRANS XYHW", &format!("{:?}", xyhw));

    //center the trans loc in the middle on the parent
    xyhw.x = parent_attrs.x + (parent_geo.w / 2) - (xyhw.w / 2);
    xyhw.y = parent_attrs.y + (parent_geo.h / 2) - (xyhw.h / 2);
    log_info("TRANS XYHW", &format!("{:?}", xyhw));

    window.floating = Some(xyhw);
    log_info("TRANS WINDOW", &format!("{:?}", window));

    Ok(())
}
