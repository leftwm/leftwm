use super::DisplayEvent;
use super::XWrap;
use crate::models::WindowChange;
use crate::models::WindowHandle;
use crate::Command;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XClientMessageEvent) -> Option<DisplayEvent> {
    //let atom_name = xw.atoms.get_name(event.message_type);
    //println!("ClientMessage: {} {:?}", event.window, atom_name);

    if event.message_type == xw.atoms.NetCurrentDesktop {
        return goto_tag_by_index(xw, event.data.get_long(0));
    }

    if event.message_type == xw.atoms.NetWMState && (
        event.data.get_long(1) == xw.atoms.NetWMStateFullscreen as i64
        || event.data.get_long(2) == xw.atoms.NetWMStateFullscreen as i64 )
    {
        let set_fullscreen = event.data.get_long(0) == 1;
        let toggle_fullscreen = event.data.get_long(0) == 2;
        let handle = WindowHandle::XlibHandle(event.window);
        let mut change = WindowChange::new(handle);
        if toggle_fullscreen {
            change.toggle_fullscreen = Some(true);
            return Some(DisplayEvent::WindowChange(change));
        } else {
            change.set_fullscreen = Some(set_fullscreen);
            return Some(DisplayEvent::WindowChange(change));
        }
    }

    //if event.message_type == xw.atoms.NetActiveWindow {
    //    let atom_name = xw.atoms.get_name(event.message_type);
    //}
    //if event.message_type == xw.atoms.NetWMState {
    //    println!("EVENT: {:?}", event);
    //    let data = event.data.get_long(1);
    //    let atom_name = xw.atoms.get_name(data as u64);
    //    println!("NetState: {}", atom_name);
    //}

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
