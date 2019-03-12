use super::DisplayEvent;
use super::WindowHandle;
use super::XWrap;
use crate::models::WindowChange;
use x11_dl::xlib;
use std::os::raw::{ c_long};

pub fn from_event(xw: &XWrap, event: xlib::XPropertyEvent) -> Option<DisplayEvent> {
    if event.window == xw.get_default_root() || event.state == xlib::PropertyDelete {
        return None;
    }
    match event.atom {
        xlib::XA_WM_TRANSIENT_FOR => {
            let handle = WindowHandle::XlibHandle(event.window);
            let mut change = WindowChange::new(handle);
            let trans = xw.get_transient_for(event.window);
            if let Some(trans) = trans {
                change.transient = Some(Some(WindowHandle::XlibHandle(trans)));
            } else {
                change.transient = Some(None);
            }
            Some( DisplayEvent::WindowChange(change) )
        },
        xlib::XA_WM_NORMAL_HINTS => {
            if let Some(change) = build_change_for_size_hints(xw, event.window){
                return Some( DisplayEvent::WindowChange(change) );
            }
            None
        },
        xlib::XA_WM_HINTS => {
            // TODO: update wm hints 
            // never focus, is urgent
            None
        },
        _ => None,
    }
}


fn build_change_for_size_hints(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    if let Some(sizing) = get_hint_sizing(xw, window) {
        let handle = WindowHandle::XlibHandle(window);
        let mut change = WindowChange::new(handle);
        if sizing.flags & xlib::PBaseSize > 0 {
            change.floating_size = Some( (sizing.base_width, sizing.base_height) );
        }
        if sizing.flags & xlib::PMinSize > 0 {
            change.floating_size = Some( (sizing.min_width, sizing.min_height) );
        }
    }
    None
}

fn get_hint_sizing(xw: &XWrap, window: xlib::Window) -> Option<xlib::XSizeHints> {
    let mut xsize: xlib::XSizeHints = unsafe { std::mem::uninitialized() };
    let mut msize: c_long = xlib::PSize;
    let status = unsafe{ (xw.xlib.XGetWMNormalHints)(xw.display, window, &mut xsize, &mut msize) };
    match status {
        0 => None,
        _ => {
            Some(xsize)
        }
    }
}
