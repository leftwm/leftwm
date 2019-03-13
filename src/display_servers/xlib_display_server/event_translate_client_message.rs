use super::DisplayEvent;
use super::XWrap;
use crate::Command;
use x11_dl::xlib;

pub fn from_event(xw: &XWrap, event: xlib::XClientMessageEvent) -> Option<DisplayEvent> {
    //let atom_name = xw.atoms.get_name(event.message_type);

    if event.message_type == xw.atoms.NetCurrentDesktop {
        return goto_tag_by_index(xw, event.data.get_long(0));
    }

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
        let tag_name = xw.tags[index as usize].clone();
        Some(DisplayEvent::SendCommand(Command::GotoTag, Some(tag_name)))
    } else {
        None
    }
}
