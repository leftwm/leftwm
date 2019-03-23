use super::DisplayEvent;
use super::WindowHandle;
use super::XWrap;
use crate::models::WindowChange;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XPropertyEvent) -> Option<DisplayEvent> {
    if event.window == xw.get_default_root() || event.state == xlib::PropertyDelete {
        return None;
    }
    match event.atom {
        xlib::XA_WM_TRANSIENT_FOR => {
            //let atom_name = xw.atoms.get_name(event.atom);
            //crate::logging::log_info("XPropertyEvent", &format!("{:?} {:?}", atom_name, event));
            let handle = WindowHandle::XlibHandle(event.window);
            let mut change = WindowChange::new(handle);
            let trans = xw.get_transient_for(event.window);
            if let Some(trans) = trans {
                change.transient = Some(Some(WindowHandle::XlibHandle(trans)));
            } else {
                change.transient = Some(None);
            }
            Some(DisplayEvent::WindowChange(change))
        }
        xlib::XA_WM_NORMAL_HINTS => {
            //let atom_name = xw.atoms.get_name(event.atom);
            //crate::logging::log_info("XPropertyEvent", &format!("{:?} {:?}", atom_name, event));
            if let Some(change) = build_change_for_size_hints(xw, event.window) {
                return Some(DisplayEvent::WindowChange(change));
            }
            None
        }
        xlib::XA_WM_HINTS => {
            //let atom_name = xw.atoms.get_name(event.atom);
            //crate::logging::log_info("XPropertyEvent", &format!("{:?} {:?}", atom_name, event));
            // TODO: update wm hints
            // never focus, is urgent
            //println!("XA_WM_HINTS");
            None
        }
        _ => {
            //let atom_name = xw.atoms.get_name(event.atom);
            //crate::logging::log_info("XPropertyEvent", &format!("{:?} {:?}", atom_name, event));
            None
        }
    }
}

fn build_change_for_size_hints(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    let handle = WindowHandle::XlibHandle(window);
    let mut change = WindowChange::new(handle);
    let hint = xw.get_hint_sizing_as_xyhw(window)?;
    change.floating = Some(hint);
    Some(change)
}
