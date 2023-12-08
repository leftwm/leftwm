use std::ffi::CString;

use leftwm_core::models::{TagId, WindowHandle};
use x11rb::{protocol::xproto::{self, ChangeWindowAttributesAux, PropMode}, properties::WmHints};

use crate::xatom;

use super::XWrap;

impl XWrap {
    // Public functions.

    /// Appends a window property.
    pub fn append_property_u32(
        &self,
        window: xproto::Window,
        property: xproto::Atom,
        r#type: xproto::Atom,
        data: &[u32],
    ) {
        let modified_data: Vec<u8> = data
            .into_iter()
            .map(|data| data.to_ne_bytes())
            .flatten()
            .collect();
        xproto::change_property(
            &self.conn,
            xproto::PropMode::APPEND,
            window,
            property,
            r#type,
            32,
            data.len() as u32,
            modified_data.as_slice(),
        )
        .unwrap();
    }

    /// Replaces a window property.
    pub fn replace_property_u32(
        &self,
        window: xproto::Window,
        property: xproto::Atom,
        r#type: xproto::Atom,
        data: &[u32],
    ) {
        let modified_data: Vec<u8> = data
            .into_iter()
            .map(|data| data.to_ne_bytes())
            .flatten()
            .collect();
        xproto::change_property(
            &self.conn,
            PropMode::REPLACE,
            window,
            property,
            r#type,
            32,
            data.len() as u32,
            modified_data.as_slice(),
        )
        .unwrap();
    }

    /// Sets the client list to the currently managed windows.
    pub fn set_client_list(&self) {
        xproto::delete_property(&self.conn, self.root, self.atoms.NetClientList).unwrap();
        for w in &self.managed_windows {
            self.append_property_u32(
                self.root,
                self.atoms.NetClientList,
                xproto::AtomEnum::ATOM.into(),
                &[*w],
            );
        }
    }

    /// Sets the current desktop.
    pub fn set_current_desktop(&self, current_tag: Option<TagId>) {
        let indexes: Vec<u32> = match current_tag {
            Some(tag) => vec![tag as u32 - 1],
            None => vec![0],
        };
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    /// Sets a desktop property.
    pub fn set_desktop_prop(&self, data: &[u32], atom: xproto::Atom) {
        self.replace_property_u32(self.root, atom, xproto::AtomEnum::CARDINAL.into(), data);
    }

    /// Sets a desktop property with type `u32`.
    pub fn set_desktop_prop_u32(&self, value: u32, atom: xproto::Atom, r#type: xproto::Atom) {
        self.replace_property_u32(self.root, atom, r#type, &[value]);
    }

    /// Sets a desktop property with type string.
    pub fn set_desktop_prop_string(&self, value: &str, atom: xproto::Atom, encoding: xproto::Atom) {
        if let Ok(cstring) = CString::new(value) {
            let data = cstring.as_bytes();
            xproto::change_property(
                &self.conn,
                xproto::PropMode::REPLACE,
                self.root,
                atom,
                encoding,
                8,
                data.len() as u32,
                data,
            ).unwrap();
        }
    }

    /// Sets a windows state.
    pub fn set_state(&self, handle: WindowHandle, toggle_to: bool, atom: xproto::Atom) {
        if let WindowHandle::X11rbHandle(h) = handle {
            let mut states = self.get_window_states_atoms(h);
            if toggle_to {
                if states.contains(&atom) {
                    return;
                }
                states.push(atom);
            } else {
                let Some(index) = states.iter().position(|s| s == &atom) else { return };
                states.remove(index);
            }
            self.set_window_states_atoms(h, &states);
        }
    }

    /// Sets a windows border color.
    pub fn set_window_border_color(&self, window: xproto::Window, color: u32) {
        xproto::change_window_attributes(
            &self.conn,
            window,
            &ChangeWindowAttributesAux::new().border_pixel(color),
        )
        .unwrap();
    }

    pub fn set_background_color(&self, color: u32) {
        xproto::change_window_attributes(
            &self.conn,
            self.root,
            &ChangeWindowAttributesAux::new().backing_pixel(color),
        )
        .unwrap();
    }

    /// Sets a windows configuration.
    pub fn set_window_config(
        &self,
        window: xproto::Window,
        window_changes: &xproto::ConfigureWindowAux,
    ) {
        xproto::configure_window(&self.conn, window, window_changes).unwrap();
    }

    /// Sets what desktop a window is on.
    pub fn set_window_desktop(&self, window: xproto::Window, current_tag: &TagId) {
        let mut indexes: Vec<u32> = vec![*current_tag as u32 - 1];
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.replace_property_u32(
            window,
            self.atoms.NetWMDesktop,
            xproto::AtomEnum::CARDINAL.into(),
            &indexes,
        );
    }

    /// Sets the atom states of a window.
    pub fn set_window_states_atoms(&self, window: xproto::Window, states: &[xproto::Atom]) {
        self.replace_property_u32(window, self.atoms.NetWMState, xproto::AtomEnum::ATOM.into(), states);
    }

    pub fn set_window_urgency(&self, window: xproto::Window, is_urgent: bool) {
        if let Some(mut wmh) = self.get_wmhints(window) {
            if wmh.urgent == is_urgent {
                return;
            }
            wmh.urgent = is_urgent;
            wmh.set(&self.conn, window).unwrap();
        }
    }

    /// Sets the `XWMHints` of a window.
    pub fn set_wmhints(&self, window: xproto::Window, wmh: &WmHints) {
        wmh.set(&self.conn, window).unwrap();
    }

    /// Sets the `WM_STATE` of a window.
    pub fn set_wm_state(&self, window: xproto::Window, states: xatom::WMStateWindowState) {
        self.replace_property_u32(window, self.atoms.WMState, self.atoms.WMState, &[states.into()]);
    }
}
