use super::DisplayEvent;
use super::XWrap;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::models::WindowType;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XPropertyEvent) -> Option<DisplayEvent> {
    if event.window == xw.get_default_root() || event.state == xlib::PropertyDelete {
        return None;
    }

    let event_name = xw.get_xatom_name(event.atom).unwrap();
    log::trace!("PropertyNotify: {} : {:?}", event_name, &event);

    match event.atom {
        xlib::XA_WM_TRANSIENT_FOR => {
            let handle = WindowHandle::XlibHandle(event.window);
            let mut change = WindowChange::new(handle);
            let trans = xw.get_transient_for(event.window);

            match trans {
                Some(trans) => change.transient = Some(Some(WindowHandle::XlibHandle(trans))),
                None => change.transient = Some(None),
            }

            Some(DisplayEvent::WindowChange(change))
        }
        xlib::XA_WM_NORMAL_HINTS => match build_change_for_size_hints(xw, event.window) {
            Some(change) => Some(DisplayEvent::WindowChange(change)),
            None => None,
        },
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

            if event.atom == xw.atoms.NetWMStrut
                || event.atom == xw.atoms.NetWMStrutPartial
                    && xw.get_window_type(event.window) == WindowType::Dock
            {
                if let Some(change) = build_change_for_size_strut_partial(xw, event.window) {
                    return Some(DisplayEvent::WindowChange(change));
                }
            }

            None
        }
    }
}

fn build_change_for_size_strut_partial(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    let handle = WindowHandle::XlibHandle(window);
    let mut change = WindowChange::new(handle);
    let dock_area = xw.get_window_strut_array(window)?;
    let dems = xw.screens_area_dimensions();
    let xywh = dock_area.as_xyhw(dems.0, dems.1)?;
    change.floating = Some(xywh.into());
    change.type_ = Some(WindowType::Dock);
    Some(change)
}

fn build_change_for_size_hints(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    let handle = WindowHandle::XlibHandle(window);
    let mut change = WindowChange::new(handle);
    let hint = xw.get_hint_sizing_as_xyhw(window)?;
    if hint.x.is_none() && hint.y.is_none() && hint.w.is_none() && hint.h.is_none() {
        //junk hint; change change anything
        return None;
    }
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
