use super::DisplayEvent;
use super::XWrap;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::Command;
use std::convert::TryFrom;
use std::os::raw::c_long;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XClientMessageEvent) -> Option<DisplayEvent> {
    let atom_name = xw.atoms.get_name(event.message_type);
    log::trace!("ClientMessage: {} : {:?}", event.window, atom_name);

    if event.message_type == xw.atoms.NetCurrentDesktop {
        let value = event.data.get_long(0);
        match usize::try_from(value) {
            Ok(index) => {
                return Some(DisplayEvent::SendCommand(Command::GoToTag {
                    tag: index + 1,
                    swap: false,
                }));
            }
            Err(err) => {
                log::debug!(
                    "Received invalid value for current desktop new index ({}): {}",
                    value,
                    err,
                );
                return None;
            }
        }
    }

    //if the client is trying to toggle fullscreen without changing the window state, change it too
    if event.message_type == xw.atoms.NetWMState
        && (event.data.get_long(1) == xw.atoms.NetWMStateFullscreen as c_long
            || event.data.get_long(2) == xw.atoms.NetWMStateFullscreen as c_long)
    {
        let set_fullscreen = event.data.get_long(0) == 1;
        let toggle_fullscreen = event.data.get_long(0) == 2;
        let mut states = xw.get_window_states_atoms(event.window);
        //determine what to change the state to
        let fullscreen = if toggle_fullscreen {
            !states.contains(&xw.atoms.NetWMStateFullscreen)
        } else {
            set_fullscreen
        };
        //update the list of states
        if fullscreen {
            states.push(xw.atoms.NetWMStateFullscreen);
        } else {
            states.retain(|x| x != &xw.atoms.NetWMStateFullscreen);
        }
        states.sort_unstable();
        states.dedup();
        //set the windows state
        xw.set_window_states_atoms(event.window, &states);
    }

    //update the window states
    if event.message_type == xw.atoms.NetWMState {
        let handle = WindowHandle::XlibHandle(event.window);
        let mut change = WindowChange::new(handle);
        let states = xw.get_window_states(event.window);
        change.states = Some(states);
        return Some(DisplayEvent::WindowChange(change));
    }

    None
}
