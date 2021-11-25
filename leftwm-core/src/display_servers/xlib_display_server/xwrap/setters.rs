//! `XWrap` setters.
use super::WindowHandle;
use crate::models::TagId;
use crate::XWrap;
use std::ffi::CString;
use std::os::raw::{c_long, c_ulong};
use x11_dl::xlib;

impl XWrap {
    // Public functions.

    /// Appends a window property.
    // `XChangeProperty`: https://tronche.com/gui/x/xlib/window-information/XChangeProperty.html
    pub fn append_property_long(
        &self,
        window: xlib::Window,
        property: xlib::Atom,
        r#type: xlib::Atom,
        data: &[c_long],
    ) {
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                property,
                r#type,
                32,
                xlib::PropModeAppend,
                data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
        }
    }

    /// Replaces a window property.
    // `XChangeProperty`: https://tronche.com/gui/x/xlib/window-information/XChangeProperty.html
    pub fn replace_property_long(
        &self,
        window: xlib::Window,
        property: xlib::Atom,
        r#type: xlib::Atom,
        data: &[c_long],
    ) {
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                property,
                r#type,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
        }
    }

    /// Sets the client list to the currently managed windows.
    // `XDeleteProperty`: https://tronche.com/gui/x/xlib/window-information/XDeleteProperty.html
    pub fn set_client_list(&self) {
        unsafe {
            (self.xlib.XDeleteProperty)(self.display, self.root, self.atoms.NetClientList);
        }
        for w in &self.managed_windows {
            let list = vec![*w as c_long];
            self.append_property_long(self.root, self.atoms.NetClientList, xlib::XA_WINDOW, &list);
        }
    }

    /// Sets the current desktop.
    pub fn set_current_desktop(&self, current_tags: &[TagId]) {
        let mut indexes: Vec<u32> = current_tags
            .iter()
            .map(|tag| tag.to_owned() as u32 - 1)
            .collect();
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    // /// Sets the current viewport.
    // fn set_current_viewport(&self, tags: Vec<&String>) {
    //     let mut indexes: Vec<u32> = vec![];
    //     for tag in tags {
    //         for (i, mytag) in self.tags.iter().enumerate() {
    //             if tag.contains(mytag) {
    //                 indexes.push(i as u32);
    //             }
    //         }
    //     }
    //     if indexes.is_empty() {
    //         indexes.push(0);
    //     }
    //     self.set_desktop_prop(&indexes, self.atoms.NetDesktopViewport);
    // }

    /// Sets a desktop property.
    pub fn set_desktop_prop(&self, data: &[u32], atom: c_ulong) {
        let x_data: Vec<c_long> = data.iter().map(|x| *x as c_long).collect();
        self.replace_property_long(self.root, atom, xlib::XA_CARDINAL, &x_data);
    }

    /// Sets a desktop property with type `c_ulong`.
    pub fn set_desktop_prop_c_ulong(&self, value: c_ulong, atom: c_ulong, r#type: c_ulong) {
        let data = vec![value as c_long];
        self.replace_property_long(self.root, atom, r#type, &data);
    }

    /// Sets a desktop property with type string.
    // `XChangeProperty`: https://tronche.com/gui/x/xlib/window-information/XChangeProperty.html
    pub fn set_desktop_prop_string(&self, value: &str, atom: c_ulong) {
        if let Ok(cstring) = CString::new(value) {
            unsafe {
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.root,
                    atom,
                    xlib::XA_CARDINAL,
                    8,
                    xlib::PropModeReplace,
                    cstring.as_ptr().cast::<u8>(),
                    value.len() as i32,
                );
                std::mem::forget(cstring);
            }
        }
    }

    /// Sets a windows state.
    pub fn set_state(&self, handle: WindowHandle, toggle_to: bool, atom: xlib::Atom) {
        if let WindowHandle::XlibHandle(h) = handle {
            let mut states = self.get_window_states_atoms(h);
            if toggle_to {
                if states.contains(&atom) {
                    return;
                }
                states.push(atom);
            } else {
                let index = match states.iter().position(|s| s == &atom) {
                    Some(i) => i,
                    None => return,
                };
                states.remove(index);
            }
            self.set_window_states_atoms(h, &states);
        }
    }

    pub fn set_window_config(
        &self,
        window: xlib::Window,
        mut window_changes: xlib::XWindowChanges,
        unlock: u32,
    ) {
        unsafe {
            (self.xlib.XConfigureWindow)(self.display, window, unlock, &mut window_changes);
            (self.xlib.XSync)(self.display, 0);
        }
    }

    /// Sets what desktop a window is on.
    pub fn set_window_desktop(&self, window: xlib::Window, current_tags: &[TagId]) {
        let mut indexes: Vec<c_long> = current_tags.iter().map(|tag| (tag - 1) as c_long).collect();
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.replace_property_long(window, self.atoms.NetWMDesktop, xlib::XA_CARDINAL, &indexes);
    }

    /// Sets the atom states of a window.
    pub fn set_window_states_atoms(&self, window: xlib::Window, states: &[xlib::Atom]) {
        let data: Vec<c_long> = states.iter().map(|x| *x as c_long).collect();
        self.replace_property_long(window, self.atoms.NetWMState, xlib::XA_ATOM, &data);
    }

    /// Sets the `WM_STATE` of a window.
    pub fn set_wm_states(&self, window: xlib::Window, states: &[c_long]) {
        self.replace_property_long(window, self.atoms.WMState, self.atoms.WMState, states);
    }
}
