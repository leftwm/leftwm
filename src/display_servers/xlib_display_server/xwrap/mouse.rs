//! Xlib calls related to a mouse.
use super::{XlibError, MOUSEMASK};
use crate::display_servers::xlib_display_server::xwrap::BUTTONMASK;
use crate::models::FocusBehaviour;
use crate::utils::xkeysym_lookup::ModMask;
use crate::XWrap;
use std::os::raw::{c_int, c_ulong};
use x11_dl::xlib;

impl XWrap {
    /// Grabs the mouse clicks of a window.
    pub fn grab_mouse_clicks(&self, handle: xlib::Window) {
        self.ungrab_buttons(handle);
        self.grab_buttons(handle, xlib::Button1, self.mouse_key_mask);
        self.grab_buttons(handle, xlib::Button1, self.mouse_key_mask | xlib::ShiftMask);
        self.grab_buttons(handle, xlib::Button3, self.mouse_key_mask);
        self.grab_buttons(handle, xlib::Button3, self.mouse_key_mask | xlib::ShiftMask);
    }

    /// Grabs the button with the modifier for a window.
    pub fn grab_buttons(&self, window: xlib::Window, button: u32, modifiers: u32) {
        // Grab the buttons with and without numlock (Mod2).
        let mods: Vec<u32> = vec![
            modifiers,
            modifiers | xlib::Mod2Mask,
            modifiers | xlib::LockMask,
        ];
        for m in mods {
            unsafe {
                (self.xlib.XGrabButton)(
                    self.display,
                    button,
                    m,
                    window,
                    0,
                    BUTTONMASK as u32,
                    xlib::GrabModeSync,
                    xlib::GrabModeAsync,
                    0,
                    0,
                );
            }
        }
    }

    /// Cleans all currently grabbed buttons of a window.
    pub fn ungrab_buttons(&self, handle: xlib::Window) {
        unsafe {
            (self.xlib.XUngrabButton)(
                self.display,
                xlib::AnyButton as u32,
                xlib::AnyModifier,
                handle,
            );
        }
    }

    /// Grabs the cursor and sets its visual.
    pub fn grab_pointer(&self, cursor: c_ulong) {
        unsafe {
            //grab the mouse
            (self.xlib.XGrabPointer)(
                self.display,
                self.root,
                0,
                MOUSEMASK as u32,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
                0,
                cursor,
                xlib::CurrentTime,
            );
        }
    }

    /// Ungrab the cursor.
    pub fn ungrab_pointer(&self) {
        unsafe {
            //release the mouse grab
            (self.xlib.XUngrabPointer)(self.display, xlib::CurrentTime);
        }
    }

    /// Move the cursor to a window.
    /// # Errors
    ///
    /// Will error if unable to obtain window attributes. See `get_window_attrs`.
    pub fn move_cursor_to_window(&self, window: xlib::Window) -> Result<(), XlibError> {
        let attrs = self.get_window_attrs(window)?;
        let point = (attrs.x + (attrs.width / 2), attrs.y + (attrs.height / 2));
        self.move_cursor_to_point(point)
    }

    /// Move the cursor to a point.
    /// # Errors
    ///
    /// Error indicates `XlibError`.
    // TODO: Verify that Error is unreachable or specify conditions that may result
    // in an error.
    pub fn move_cursor_to_point(&self, point: (i32, i32)) -> Result<(), XlibError> {
        if point.0 >= 0 && point.1 >= 0 {
            let none: c_int = 0;
            unsafe {
                (self.xlib.XWarpPointer)(
                    self.display,
                    none as c_ulong,
                    self.root,
                    none,
                    none,
                    none as u32,
                    none as u32,
                    point.0,
                    point.1,
                );
            }
        }
        Ok(())
    }

    /// Replay a click on a window.
    pub fn replay_click(&self, mod_mask: ModMask) {
        // Only replay the click when in ClickToFocus and we are not trying to move/resize the
        // window.
        if self.focus_behaviour == FocusBehaviour::ClickTo
            && !(mod_mask == self.mouse_key_mask
                || mod_mask == (self.mouse_key_mask | xlib::ShiftMask))
        {
            unsafe {
                (self.xlib.XAllowEvents)(self.display, xlib::ReplayPointer, xlib::CurrentTime);
                (self.xlib.XSync)(self.display, 0);
            }
        }
    }
}
