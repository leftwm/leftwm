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

            match trans {
                Some(trans) => change.transient = Some(Some(WindowHandle::XlibHandle(trans))),
                None => change.transient = Some(None),
            }

            Some(DisplayEvent::WindowChange(change))
        }
        xlib::XA_WM_NORMAL_HINTS => {
            //let atom_name = xw.atoms.get_name(event.atom);
            //crate::logging::log_info("XPropertyEvent", &format!("{:?} {:?}", atom_name, event));
            match build_change_for_size_hints(xw, event.window) {
                Some(change) => Some(DisplayEvent::WindowChange(change)),
                None => None,
            }
        }
        xlib::XA_WM_HINTS => match xw.get_wmhints(event.window) {
            Some(hints) if hints.flags & xlib::InputHint != 0 => {
                let handle = WindowHandle::XlibHandle(event.window);
                let mut change = WindowChange::new(handle);
                change.never_focus = Some(hints.input == 0);
                Some(DisplayEvent::WindowChange(change))
            }
            Some(_hints) => None,
            None => None,
        },
        xlib::XA_WM_NAME => update_title(xw, event.window),
        _ => {
            if event.atom == xw.atoms.NetWMName {
                return update_title(xw, event.window);
            }

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

fn update_title(xw: &XWrap, window: xlib::Window) -> Option<DisplayEvent> {
    let title = xw.get_window_name(window);
    let handle = WindowHandle::XlibHandle(window);
    let mut change = WindowChange::new(handle);
    change.name = Some(title);
    Some(DisplayEvent::WindowChange(change))
}
