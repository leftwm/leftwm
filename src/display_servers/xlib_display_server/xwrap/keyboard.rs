use super::utils;
use super::XlibError;
use crate::config::Keybind;
use crate::XWrap;
use std::os::raw::c_ulong;
use x11_dl::xlib;

impl XWrap {
    /// Converts a keycode to a keysym.
    #[must_use]
    pub fn keycode_to_keysym(&self, keycode: u32) -> utils::xkeysym_lookup::XKeysym {
        // Not using XKeysymToKeycode because deprecated.
        let sym = unsafe { (self.xlib.XkbKeycodeToKeysym)(self.display, keycode as u8, 0, 0) };
        sym as u32
    }

    /// Converts a keysym to a keycode.
    pub fn keysym_to_keycode(&self, keysym: utils::xkeysym_lookup::XKeysym) -> u32 {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, keysym.into()) };
        u32::from(code)
    }

    /// Updates the keyboard mapping.
    /// # Errors
    ///
    /// Will error if updating the keyboard failed.
    pub fn refresh_keyboard(&self, evt: &mut xlib::XMappingEvent) -> Result<(), XlibError> {
        let status = unsafe { (self.xlib.XRefreshKeyboardMapping)(evt) };
        if status == 0 {
            Err(XlibError::FailedStatus)
        } else {
            Ok(())
        }
    }

    /// Grabs the keysym with the modifier for a window.
    pub fn grab_keys(&self, root: xlib::Window, keysym: u32, modifiers: u32) {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, c_ulong::from(keysym)) };
        // Grab the keys with and without numlock (Mod2).
        let mods: Vec<u32> = vec![
            modifiers,
            modifiers | xlib::Mod2Mask,
            modifiers | xlib::LockMask,
        ];
        for m in mods {
            unsafe {
                (self.xlib.XGrabKey)(
                    self.display,
                    i32::from(code),
                    m,
                    root,
                    1,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
            }
        }
    }

    /// Resets the keybindings to a list of keybindings.
    pub fn reset_grabs(&self, keybinds: &[Keybind]) {
        // Cleanup key grabs.
        unsafe {
            (self.xlib.XUngrabKey)(self.display, xlib::AnyKey, xlib::AnyModifier, self.root);
        }

        // Grab all the key combos from the config file.
        for kb in keybinds {
            if let Some(keysym) = utils::xkeysym_lookup::into_keysym(&kb.key) {
                let modmask = utils::xkeysym_lookup::into_modmask(&kb.modifier);
                self.grab_keys(self.root, keysym, modmask);
            }
        }
    }
}
