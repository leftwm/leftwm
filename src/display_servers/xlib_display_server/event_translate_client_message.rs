use super::DisplayEvent;
use super::XWrap;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::Command;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XClientMessageEvent) -> Option<DisplayEvent> {
    let atom_name = xw.atoms.get_name(event.message_type);
    log::trace!("ClientMessage: {} : {:?}", event.window, atom_name);

    if event.message_type == xw.atoms.NetCurrentDesktop {
        return goto_tag_by_index(xw, event.data.get_long(0));
    }

    //if the client is trying to toggle fullscreen without changing the window state, change it too
    if event.message_type == xw.atoms.NetWMState
        && (event.data.get_long(1) == xw.atoms.NetWMStateFullscreen as i64
            || event.data.get_long(2) == xw.atoms.NetWMStateFullscreen as i64)
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
        states.sort();
        states.dedup();
        //set the windows state
        xw.set_window_states_atoms(event.window, states);
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

fn goto_tag_by_index(xw: &XWrap, index: i64) -> Option<DisplayEvent> {
    if index >= 0 && index < xw.tags.len() as i64 {
        let tag_num = index + 1;
        Some(DisplayEvent::SendCommand(
            Command::GotoTag,
            Some(tag_num.to_string()),
        ))
    } else {
        None
    }
}
