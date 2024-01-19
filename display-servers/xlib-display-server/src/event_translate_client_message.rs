use crate::XlibWindowHandle;

use super::{DisplayEvent, XWrap};
use leftwm_core::models::WindowHandle;
use leftwm_core::{models::WindowChange, Command};
use std::convert::TryFrom;
use std::os::raw::c_long;

use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XClientMessageEvent) -> Option<DisplayEvent<XlibWindowHandle>> {
    if !xw.managed_windows.contains(&event.window) && event.window != xw.get_default_root() {
        return None;
    }
    let atom_name = xw.atoms.get_name(event.message_type);
    tracing::trace!("ClientMessage: {} : {:?}", event.window, atom_name);

    if event.message_type == xw.atoms.NetCurrentDesktop {
        let value = event.data.get_long(0);
        match usize::try_from(value) {
            Ok(index) => {
                let event = DisplayEvent::SendCommand(Command::GoToTag {
                    tag: index + 1,
                    swap: false,
                });
                return Some(event);
            }
            Err(err) => {
                tracing::debug!(
                    "Received invalid value for current desktop new index ({}): {}",
                    value,
                    err,
                );
                return None;
            }
        }
    }
    if event.message_type == xw.atoms.NetWMDesktop {
        let value = event.data.get_long(0);
        match usize::try_from(value) {
            Ok(index) => {
                let event = DisplayEvent::SendCommand(Command::SendWindowToTag {
                    window: Some(WindowHandle(XlibWindowHandle(event.window))),
                    tag: index + 1,
                });
                return Some(event);
            }
            Err(err) => {
                tracing::debug!(
                    "Received invalid value for current desktop new index ({}): {}",
                    value,
                    err,
                );
                return None;
            }
        }
    }
    if event.message_type == xw.atoms.NetActiveWindow {
        xw.set_window_urgency(event.window, true);
        return None;
    }

    // if the client is trying to toggle fullscreen without changing the window state, change it too
    if event.message_type == xw.atoms.NetWMState
        && (event.data.get_long(1) == xw.atoms.NetWMStateFullscreen as c_long
            || event.data.get_long(2) == xw.atoms.NetWMStateFullscreen as c_long)
    {
        let set_fullscreen = event.data.get_long(0) == 1;
        let toggle_fullscreen = event.data.get_long(0) == 2;
        let mut states = xw.get_window_states_atoms(event.window);
        // determine what to change the state to
        let fullscreen = if toggle_fullscreen {
            !states.contains(&xw.atoms.NetWMStateFullscreen)
        } else {
            set_fullscreen
        };
        // update the list of states
        if fullscreen {
            states.push(xw.atoms.NetWMStateFullscreen);
        } else {
            states.retain(|x| x != &xw.atoms.NetWMStateFullscreen);
        }
        states.sort_unstable();
        states.dedup();
        // set the windows state
        xw.set_window_states_atoms(event.window, &states);
    }

    // update the window states
    if event.message_type == xw.atoms.NetWMState {
        let handle = WindowHandle(XlibWindowHandle(event.window));
        let mut change = WindowChange::new(handle);
        let states = xw.get_window_states(event.window);
        change.states = Some(states);
        return Some(DisplayEvent::WindowChange(change));
    }

    None
}
