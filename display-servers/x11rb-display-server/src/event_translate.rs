use std::backtrace::BacktraceStatus;

use leftwm_core::{
    models::{WindowChange, WindowHandle, WindowType, XyhwChange},
    utils::modmask_lookup::{Button, ModMask},
    DisplayEvent, Mode,
};
use x11rb::protocol::{xproto, Event};

use crate::xwrap::XWrap;
use crate::{error::Result, X11rbWindowHandle};

mod client_message;
mod property_notify;

/// Translate events from x11rb to leftwm's DisplayEvent
pub(crate) fn translate(event: Event, xw: &mut XWrap) -> Option<DisplayEvent<X11rbWindowHandle>> {
    let is_normal = xw.mode == Mode::Normal;
    let is_sloppy = xw.focus_behaviour.is_sloppy();

    let res = match event {
        Event::MapRequest(e) => xw.setup_window(e.window),
        Event::UnmapNotify(e) => from_unmap_event(e, xw),
        Event::DestroyNotify(e) => from_destroy_notify(e, xw),
        Event::FocusIn(e) => from_focus_in(e, xw),
        Event::ClientMessage(e) if is_normal => client_message::from_event(e, xw),
        Event::PropertyNotify(e) if is_normal => property_notify::from_event(e, xw),
        Event::ConfigureRequest(e) if is_normal => from_configure_request(e, xw),
        Event::EnterNotify(e) if is_normal && is_sloppy => from_enter_notify(e, xw),
        Event::MotionNotify(e) => from_motion_notify(e, xw),
        Event::ButtonPress(e) => from_button_press(e, xw),
        Event::ButtonRelease(e) if !is_normal => from_button_release(e, xw),
        _ => return None,
    };
    match res {
        Ok(display_event) => display_event,
        Err(e) => {
            tracing::error!(
                "An error occured when processing the event {:?}: {}",
                event,
                e
            );
            #[cfg(debug_assertions)]
            {
                let trace = e.backtrace;
                if trace.status() == BacktraceStatus::Captured {
                    tracing::error!("{}", trace);
                }
            }
            None
        }
    }
}

fn from_unmap_event(
    event: xproto::UnmapNotifyEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if xw.managed_windows.contains(&event.window) {
        // can't check if this event originates from a SendEvent request
        // no idea how this is supposed to be handled
        let h = WindowHandle(X11rbWindowHandle(event.window));
        xw.teardown_managed_window(&h, false)?;
        return Ok(Some(DisplayEvent::WindowDestroy(h)));

        // Set WM_STATE to withdrawn state.
        // xw.set_wm_states(event.window, &[WITHDRAWN_STATE]);
    }
    Ok(None)
}

fn from_destroy_notify(
    event: xproto::DestroyNotifyEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if xw.managed_windows.contains(&event.window) {
        let h = WindowHandle(X11rbWindowHandle(event.window));
        xw.teardown_managed_window(&h, true)?;
        return Ok(Some(DisplayEvent::WindowDestroy(h)));
    }
    Ok(None)
}

fn from_focus_in(
    event: xproto::FocusInEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    // Check that if a window is taking focus, that it should be.
    if xw.focused_window != event.event {
        let never_focus = match xw.get_wmhints(xw.focused_window)? {
            Some(hint) => !hint.input.unwrap_or(true),
            None => false,
        };
        xw.focus(xw.focused_window, never_focus)?;
    }
    Ok(None)
}

fn from_configure_request(
    event: xproto::ConfigureRequestEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    // If the window is not mapped, configure it.
    if !xw.managed_windows.contains(&event.window) {
        let window_changes = xproto::ConfigureWindowAux {
            x: Some(event.x.into()),
            y: Some(event.y.into()),
            width: Some(event.width.into()),
            height: Some(event.height.into()),
            border_width: Some(event.border_width.into()),
            sibling: Some(event.sibling),
            stack_mode: Some(event.stack_mode),
            ..Default::default()
        };
        xw.set_window_config(event.window, &window_changes)?;
        xw.move_resize_window(
            event.window,
            event.x.into(),
            event.y.into(),
            event.width as u32,
            event.height as u32,
        )?;
        return Ok(None);
    }
    let window_type = xw.get_window_type(event.window)?;
    let trans = xw.get_transient_for(event.window)?;
    let handle = WindowHandle(X11rbWindowHandle(event.window));
    if window_type == WindowType::Normal && trans.is_none() {
        return Ok(Some(DisplayEvent::ConfigureXlibWindow(handle)));
    }
    let mut change = WindowChange::new(handle);
    let xyhw = match window_type {
        // We want to handle the window positioning when it is a dialog or a normal window with a
        // parent.
        WindowType::Dialog | WindowType::Normal => XyhwChange {
            w: Some(event.width.into()),
            h: Some(event.height.into()),
            ..XyhwChange::default()
        },
        _ => XyhwChange {
            w: Some(event.width.into()),
            h: Some(event.height.into()),
            x: Some(event.x.into()),
            y: Some(event.y.into()),
            ..XyhwChange::default()
        },
    };
    change.floating = Some(xyhw);
    Ok(Some(DisplayEvent::WindowChange(change)))
}

fn from_enter_notify(
    event: xproto::EnterNotifyEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if event.mode != xproto::NotifyMode::NORMAL
        || event.detail == xproto::NotifyDetail::INFERIOR
        || event.event == xw.get_default_root()
    {
        return Ok(None);
    }

    let h = WindowHandle(X11rbWindowHandle(event.event));
    Ok(Some(DisplayEvent::WindowTakeFocus(h)))
}

fn from_motion_notify(
    event: xproto::MotionNotifyEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    // Limit motion events to current refresh rate.
    if xw.refresh_rate > 0 && event.time - xw.motion_event_limiter > (1000 / xw.refresh_rate) {
        xw.motion_event_limiter = event.time;
        let event_h = WindowHandle(X11rbWindowHandle(event.event));
        let offset_x = event.root_x as i32 - xw.mode_origin.0;
        let offset_y = event.root_y as i32 - xw.mode_origin.1;
        let display_event = match xw.mode {
            Mode::ReadyToMove(h) => {
                xw.set_mode(Mode::MovingWindow(h))?;
                DisplayEvent::MoveWindow(h, offset_x, offset_y)
            }
            Mode::MovingWindow(h) => DisplayEvent::MoveWindow(h, offset_x, offset_y),
            Mode::ReadyToResize(h) => {
                xw.set_mode(Mode::ResizingWindow(h))?;
                DisplayEvent::ResizeWindow(h, offset_x, offset_y)
            }
            Mode::ResizingWindow(h) => DisplayEvent::ResizeWindow(h, offset_x, offset_y),
            Mode::Normal if xw.focus_behaviour.is_sloppy() => {
                DisplayEvent::Movement(event_h, event.root_x as i32, event.root_y as i32)
            }
            Mode::Normal => return Ok(None),
        };
        return Ok(Some(display_event));
    }

    Ok(None)
}

fn from_button_press(
    event: xproto::ButtonPressEvent,
    _xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    let h = WindowHandle(X11rbWindowHandle(event.event));
    let mod_mask = event.state;
    mod_mask.remove(xproto::KeyButMask::MOD2 | xproto::KeyButMask::LOCK);
    Ok(Some(DisplayEvent::MouseCombo(
        ModMask::from_bits_retain(mod_mask.bits()),
        Button::from_bits_retain(event.detail),
        h,
        event.root_x as i32,
        event.root_y as i32,
    )))
}

fn from_button_release(
    _event: xproto::ButtonReleaseEvent,
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.set_mode(Mode::Normal)?;
    Ok(Some(DisplayEvent::ChangeToNormalMode))
}
