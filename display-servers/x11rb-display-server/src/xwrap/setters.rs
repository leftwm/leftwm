use std::ffi::CString;

use leftwm_core::models::{TagId, WindowHandle};
use x11rb::{
    properties::WmHints,
    protocol::xproto::{self, ChangeWindowAttributesAux, PropMode},
};

use crate::{error::Result, xatom, X11rbWindowHandle};

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
    ) -> Result<()> {
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
        )?;
        Ok(())
    }

    /// Replaces a window property.
    pub fn replace_property_u32(
        &self,
        window: xproto::Window,
        property: xproto::Atom,
        r#type: xproto::Atom,
        data: &[u32],
    ) -> Result<()> {
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
        )?;
        Ok(())
    }

    /// Sets the client list to the currently managed windows.
    pub fn set_client_list(&self) -> Result<()> {
        xproto::delete_property(&self.conn, self.root, self.atoms.NetClientList)?;
        for w in &self.managed_windows {
            self.append_property_u32(
                self.root,
                self.atoms.NetClientList,
                xproto::AtomEnum::ATOM.into(),
                &[*w],
            )?;
        }
        Ok(())
    }

    /// Sets the current desktop.
    pub fn set_current_desktop(&self, current_tag: Option<TagId>) -> Result<()> {
        let indexes: Vec<u32> = match current_tag {
            Some(tag) => vec![tag as u32 - 1],
            None => vec![0],
        };
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop)?;
        Ok(())
    }

    /// Sets a desktop property.
    pub fn set_desktop_prop(&self, data: &[u32], atom: xproto::Atom) -> Result<()> {
        self.replace_property_u32(self.root, atom, xproto::AtomEnum::CARDINAL.into(), data)
    }

    /// Sets a desktop property with type `u32`.
    pub fn set_desktop_prop_u32(
        &self,
        value: u32,
        atom: xproto::Atom,
        r#type: xproto::Atom,
    ) -> Result<()> {
        self.replace_property_u32(self.root, atom, r#type, &[value])
    }

    /// Sets a desktop property with type string.
    pub fn set_desktop_prop_string(
        &self,
        value: &str,
        atom: xproto::Atom,
        encoding: xproto::Atom,
    ) -> Result<()> {
        let cstring = CString::new(value)?;
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
        )?;
        Ok(())
    }

    /// Sets a windows state.
    pub fn set_state(
        &self,
        handle: WindowHandle<X11rbWindowHandle>,
        toggle_to: bool,
        atom: xproto::Atom,
    ) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(h)) = handle {
            let mut states = self.get_window_states_atoms(h)?;
            if toggle_to {
                if states.contains(&atom) {
                    return Ok(());
                }
                states.push(atom);
            } else {
                let Some(index) = states.iter().position(|s| s == &atom) else {
                    return Ok(());
                };
                states.remove(index);
            }
            self.set_window_states_atoms(h, &states)?;
        }
        Ok(())
    }

    /// Sets a windows border color.
    pub fn set_window_border_color(&self, window: xproto::Window, color: u32) -> Result<()> {
        xproto::change_window_attributes(
            &self.conn,
            window,
            &ChangeWindowAttributesAux::new().border_pixel(color),
        )?;
        Ok(())
    }

    pub fn set_background_color(&self, color: u32) -> Result<()> {
        xproto::change_window_attributes(
            &self.conn,
            self.root,
            &ChangeWindowAttributesAux::new().background_pixel(color),
        )?;
        xproto::clear_area(&self.conn, false, self.root, 0, 0, 0, 0)?;
        self.sync()?;
        Ok(())
    }

    /// Sets a windows configuration.
    pub fn set_window_config(
        &self,
        window: xproto::Window,
        window_changes: &xproto::ConfigureWindowAux,
    ) -> Result<()> {
        xproto::configure_window(&self.conn, window, window_changes)?;
        Ok(())
    }

    /// Sets what desktop a window is on.
    pub fn set_window_desktop(&self, window: xproto::Window, current_tag: &TagId) -> Result<()> {
        let mut indexes: Vec<u32> = vec![*current_tag as u32 - 1];
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.replace_property_u32(
            window,
            self.atoms.NetWMDesktop,
            xproto::AtomEnum::CARDINAL.into(),
            &indexes,
        )
    }

    /// Sets the atom states of a window.
    pub fn set_window_states_atoms(
        &self,
        window: xproto::Window,
        states: &[xproto::Atom],
    ) -> Result<()> {
        self.replace_property_u32(
            window,
            self.atoms.NetWMState,
            xproto::AtomEnum::ATOM.into(),
            states,
        )
    }

    pub fn set_window_urgency(&self, window: xproto::Window, is_urgent: bool) -> Result<()> {
        if let Some(mut wmh) = self.get_wmhints(window)? {
            if wmh.urgent == is_urgent {
                return Ok(());
            }
            wmh.urgent = is_urgent;
            wmh.set(&self.conn, window)?;
        }
        Ok(())
    }

    /// Sets the `XWMHints` of a window.
    pub fn set_wmhints(&self, window: xproto::Window, wmh: &WmHints) -> Result<()> {
        wmh.set(&self.conn, window)?;
        Ok(())
    }

    /// Sets the `WM_STATE` of a window.
    pub fn set_wm_state(
        &self,
        window: xproto::Window,
        states: xatom::WMStateWindowState,
    ) -> Result<()> {
        self.replace_property_u32(
            window,
            self.atoms.WMState,
            self.atoms.WMState,
            &[states.into()],
        )
    }
}
