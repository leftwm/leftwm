use leftwm_core::{
    models::{WindowChange, WindowHandle},
    Command, DisplayEvent,
};
use x11rb::protocol::xproto;

use crate::{xwrap::XWrap, X11rbWindowHandle};

use crate::error::Result;

pub(crate) fn from_event(
    event: xproto::ClientMessageEvent,
    xw: &XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if !xw.managed_windows.contains(&event.window) && event.window != xw.get_default_root() {
        return Ok(None);
    }
    let atom_name = xw.atoms.get_name(event.type_);
    tracing::trace!("ClientMessage: {} : {:?}", event.window, atom_name);

    if event.type_ == xw.atoms.NetCurrentDesktop {
        let value = event.data.as_data32();
        match usize::try_from(value[0]) {
            Ok(index) => {
                let event = DisplayEvent::SendCommand(Command::GoToTag {
                    tag: index + 1,
                    swap: false,
                });
                return Ok(Some(event));
            }
            Err(err) => {
                tracing::debug!(
                    "Received invalid value for current desktop new index ({}): {}",
                    value[0],
                    err,
                );
                return Ok(None);
            }
        }
    }

    if event.type_ == xw.atoms.NetWMDesktop {
        let value = event.data.as_data32();
        match usize::try_from(value[0]) {
            Ok(index) => {
                let event = DisplayEvent::SendCommand(Command::SendWindowToTag {
                    tag: index + 1,
                    window: Some(WindowHandle(X11rbWindowHandle(event.window))),
                });
                return Ok(Some(event));
            }
            Err(err) => {
                tracing::debug!(
                    "Received invalid value for current desktop new index ({}): {}",
                    value[0],
                    err,
                );
                return Ok(None);
            }
        }
    }

    if event.type_ == xw.atoms.NetActiveWindow {
        xw.set_window_urgency(event.window, true)?;
        return Ok(None);
    }

    if event.type_ == xw.atoms.NetWMState {
        let data = event.data.as_data32();

        if data[1] == xw.atoms.NetWMStateFullscreen || data[2] == xw.atoms.NetWMStateFullscreen {
            let set_fullscreen = data[0] == 1;
            let toggle_fullscreen = data[0] == 2;
            let mut states = xw.get_window_states_atoms(event.window)?;

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
            xw.set_window_states_atoms(event.window, &states)?;
        }

        // update the window states
        let mut change = WindowChange::new(WindowHandle(X11rbWindowHandle(event.window)));
        let states = xw.get_window_states(event.window)?;
        change.states = Some(states);
        return Ok(Some(DisplayEvent::WindowChange(change)));
    }

    Ok(None)
}
