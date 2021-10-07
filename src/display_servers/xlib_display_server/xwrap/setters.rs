//! `XWrap` setters.
use super::{Window, WindowHandle};
use crate::XWrap;
use std::ffi::CString;
use std::os::raw::c_ulong;
use x11_dl::xlib;

impl XWrap {
    /// Sets the client list to the currently managed windows.
    pub fn set_client_list(&self) {
        unsafe {
            (self.xlib.XDeleteProperty)(self.display, self.root, self.atoms.NetClientList);
            for w in &self.managed_windows {
                let list = vec![*w];
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.root,
                    self.atoms.NetClientList,
                    xlib::XA_WINDOW,
                    32,
                    xlib::PropModeAppend,
                    list.as_ptr().cast::<u8>(),
                    1,
                );
                std::mem::forget(list);
            }
        }
    }

    /// Sets the current desktop.
    pub fn set_current_desktop(&self, current_tags: &str) {
        let mut indexes: Vec<u32> = vec![];
        for (i, tag) in self.tags.iter().enumerate() {
            if current_tags.contains(tag) {
                indexes.push(i as u32);
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    /// Sets the current viewport.
    pub fn set_current_viewport(&self, tags: Vec<&String>) {
        let mut indexes: Vec<u32> = vec![];
        for tag in tags {
            for (i, mytag) in self.tags.iter().enumerate() {
                if tag.contains(mytag) {
                    indexes.push(i as u32);
                }
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.set_desktop_prop(&indexes, self.atoms.NetDesktopViewport);
    }

    /// Sets a desktop property.
    pub fn set_desktop_prop(&self, data: &[u32], atom: c_ulong) {
        let x_data = data.to_owned();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                self.root,
                atom,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                x_data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
            std::mem::forget(x_data);
        }
    }

    /// Sets a desktop property with type `c_ulong`.
    pub fn set_desktop_prop_c_ulong(&self, value: c_ulong, atom: c_ulong, type_: c_ulong) {
        let data = vec![value as u32];
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                self.root,
                atom,
                type_,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                1_i32,
            );
            std::mem::forget(data);
        }
    }

    /// Sets a desktop property with type string.
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

    /// Sets a windows fullscreen state.
    pub fn set_fullscreen(&self, window: &Window, fullscreen: bool) {
        if let WindowHandle::XlibHandle(h) = window.handle {
            let atom = self.atoms.NetWMStateFullscreen;
            let mut states = self.get_window_states_atoms(h);
            if fullscreen {
                if states.contains(&atom) {
                    return;
                }
                states.push(atom);
            } else if !fullscreen {
                let index = match states.iter().position(|s| s == &atom) {
                    Some(i) => i,
                    None => return,
                };
                states.remove(index);
            }
            self.set_window_states_atoms(h, &states);
        }
    }

    /// Sets what desktop a window is on.
    pub fn set_window_desktop(&self, window: xlib::Window, current_tags: &str) {
        let mut indexes: Vec<u32> = vec![];
        for (i, tag) in self.tags.iter().enumerate() {
            if current_tags.contains(tag) {
                let tag = i as u32;
                indexes.push(tag);
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.NetWMDesktop,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                indexes.as_ptr().cast::<u8>(),
                indexes.len() as i32,
            );
            std::mem::forget(indexes);
        }
    }

    /// Sets the atom states of a window.
    pub fn set_window_states_atoms(&self, window: xlib::Window, states: &[xlib::Atom]) {
        let data: Vec<u32> = states.iter().map(|x| *x as u32).collect();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.NetWMState,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
            std::mem::forget(data);
        }
    }

    /// Sets the `WM_STATE` of a window.
    pub fn set_wm_states(&self, window: xlib::Window, states: &[u8]) {
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.WMState,
                self.atoms.WMState,
                32,
                xlib::PropModeReplace,
                states.as_ptr().cast::<u8>(),
                states.len() as i32,
            );
        }
    }
}
