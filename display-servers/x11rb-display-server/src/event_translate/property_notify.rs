use leftwm_core::{
    models::{WindowChange, WindowHandle, WindowType, Xyhw},
    DisplayEvent,
};
use x11rb::{properties::WmHints, protocol::xproto};

use crate::xwrap::XWrap;

pub fn from_event(event: xproto::PropertyNotifyEvent, xw: &XWrap) -> Option<DisplayEvent> {
    if event.window == xw.get_default_root()
        || event.state == xproto::Property::DELETE
        || !xw.managed_windows.contains(&event.window)
    {
        return None;
    }

    let event_name = xw.get_xatom_name(event.atom).ok()?;
    tracing::trace!("PropertyNotify: {} : {:?}", event_name, &event);

    match xproto::AtomEnum::from(event.atom as u8) {
        xproto::AtomEnum::WM_TRANSIENT_FOR => {
            let handle = WindowHandle::X11rbHandle(event.window);
            let mut change = WindowChange::new(handle);

            let window_type = xw.get_window_type(event.window);
            if window_type != WindowType::Normal {
                let trans = xw.get_transient_for(event.window);
                change.transient = match trans {
                    Some(trans) => Some(Some(WindowHandle::X11rbHandle(trans))),
                    None => Some(None),
                }
            }

            Some(DisplayEvent::WindowChange(change))
        }

        xproto::AtomEnum::WM_NORMAL_HINTS => {
            let handle = WindowHandle::X11rbHandle(event.window);
            let mut change = WindowChange::new(handle);

            let hint = xw.get_hint_sizing_as_xyhw(event.window)?;
            if hint.x.is_none() && hint.y.is_none() && hint.w.is_none() && hint.h.is_none() {
                return None;
            }

            let mut xyhw = Xyhw::default();
            hint.update(&mut xyhw);
            change.requested = Some(xyhw);
            Some(DisplayEvent::WindowChange(change))
        }

        xproto::AtomEnum::WM_HINTS => xw
            .get_wmhints(event.window)
            .map(|hints| build_change_hints(event, hints))
            .map(DisplayEvent::WindowChange),

        xproto::AtomEnum::WM_NAME => update_title(xw, event.window),

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

            if event.atom == xw.atoms.NetWMState {
                let handle = WindowHandle::X11rbHandle(event.window);
                let mut change = WindowChange::new(handle);
                let states = xw.get_window_states(event.window);
                change.states = Some(states);
                return Some(DisplayEvent::WindowChange(change));
            }

            None
        }
    }
}

fn build_change_hints(event: xproto::PropertyNotifyEvent, hints: WmHints) -> WindowChange {
    let handle = WindowHandle::X11rbHandle(event.window);
    let mut change = WindowChange::new(handle);

    change.never_focus = hints.input.map(|i| !i);
    change.urgent = Some(hints.urgent);

    change
}

fn update_title(xw: &XWrap, window: xproto::Window) -> Option<DisplayEvent> {
    let title = xw.get_window_name(window);
    let handle = WindowHandle::X11rbHandle(window);
    let mut change = WindowChange::new(handle);
    change.name = Some(title);
    Some(DisplayEvent::WindowChange(change))
}

fn build_change_for_size_strut_partial(xw: &XWrap, window: xproto::Window) -> Option<WindowChange> {
    let handle = WindowHandle::X11rbHandle(window);
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
