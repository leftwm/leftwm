use super::{DisplayEvent, XWrap};
use crate::models::{WindowChange, WindowType, Xyhw};
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XPropertyEvent) -> Option<DisplayEvent> {
    if event.window == xw.get_default_root()
        || event.state == xlib::PropertyDelete
        || !xw.managed_windows.contains(&event.window)
    {
        return None;
    }

    let event_name = xw.get_xatom_name(event.atom).ok()?;
    log::trace!("PropertyNotify: {} : {:?}", event_name, &event);

    match event.atom {
        xlib::XA_WM_TRANSIENT_FOR => {
            let window_type = xw.get_window_type(event.window);
            let handle = event.window.into();
            let mut change = WindowChange::new(handle);
            if window_type != WindowType::Normal {
                let trans = xw.get_transient_for(event.window);
                match trans {
                    Some(trans) => change.transient = Some(Some(trans.into())),
                    None => change.transient = Some(None),
                }
            }
            Some(DisplayEvent::WindowChange(change))
        }
        xlib::XA_WM_NORMAL_HINTS => {
            build_change_for_size_hints(xw, event.window).map(DisplayEvent::WindowChange)
        }
        xlib::XA_WM_HINTS => match xw.get_wmhints(event.window) {
            Some(hints) if hints.flags & xlib::InputHint != 0 => {
                let handle = event.window.into();
                let mut change = WindowChange::new(handle);
                change.never_focus = Some(hints.input == 0);
                Some(DisplayEvent::WindowChange(change))
            }
            Some(_hints) => None,
            None => None,
        },
        xlib::XA_WM_NAME => Some(update_title(xw, event.window)),
        _ => {
            if event.atom == xw.atoms.NetWMName {
                return Some(update_title(xw, event.window));
            }

            if event.atom == xw.atoms.NetWMStrut
                || event.atom == xw.atoms.NetWMStrutPartial
                    && xw.get_window_type(event.window) == WindowType::Dock
            {
                if let Some(change) = build_change_for_size_strut_partial(xw, event.window) {
                    return Some(DisplayEvent::WindowChange(change));
                }
            }

            if event.atom == xw.atoms.NetWMState {
                let handle = event.window.into();
                let mut change = WindowChange::new(handle);
                let states = xw.get_window_states(event.window);
                change.states = Some(states);
                return Some(DisplayEvent::WindowChange(change));
            }

            None
        }
    }
}

fn build_change_for_size_strut_partial(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    let handle = window.into();
    let mut change = WindowChange::new(handle);
    let r#type = xw.get_window_type(window);

    if let Some(dock_area) = xw.get_window_strut_array(window) {
        let dems = xw.get_screens_area_dimensions();
        let screen = xw
            .get_screens()
            .iter()
            .find(|s| s.contains_dock_area(dock_area, dems))?
            .clone();

        if let Some(xyhw) = dock_area.as_xyhw(dems.0, dems.1, &screen) {
            change.floating = Some(xyhw.into());
            change.r#type = Some(r#type);
            return Some(change);
        }
    } else if let Ok(geo) = xw.get_window_geometry(window) {
        let mut xyhw = Xyhw::default();
        geo.update(&mut xyhw);
        change.floating = Some(xyhw.into());
        change.r#type = Some(r#type);
        return Some(change);
    }
    None
}

fn build_change_for_size_hints(xw: &XWrap, window: xlib::Window) -> Option<WindowChange> {
    let handle = window.into();
    let mut change = WindowChange::new(handle);
    let hint = xw.get_hint_sizing_as_xyhw(window)?;
    if hint.x.is_none() && hint.y.is_none() && hint.w.is_none() && hint.h.is_none() {
        //junk hint; change change anything
        return None;
    }
    let mut xyhw = Xyhw::default();
    hint.update(&mut xyhw);
    change.requested = Some(xyhw);
    Some(change)
}

fn update_title(xw: &XWrap, window: xlib::Window) -> DisplayEvent {
    let title = xw.get_window_name(window);
    let handle = window.into();
    let mut change = WindowChange::new(handle);
    change.name = Some(title);
    DisplayEvent::WindowChange(change)
}
