//! Xlib calls related to a mouse.
use super::{XlibError, MOUSEMASK};
use crate::xwrap::BUTTONMASK;
use crate::XWrap;
use std::os::raw::{c_int, c_uint, c_ulong};
use x11_dl::xlib;

impl XWrap {
    /// Grabs the mouse clicks of a window.
    pub fn grab_mouse_clicks(&self, handle: xlib::Window, is_focused: bool) {
        self.ungrab_buttons(handle);
        if !is_focused {
            self.grab_buttons(handle, xlib::Button1, xlib::AnyModifier);
            self.grab_buttons(handle, xlib::Button3, xlib::AnyModifier);
        }
        self.grab_buttons(handle, xlib::Button1, self.mouse_key_mask.bits() as u32);
        self.grab_buttons(
            handle,
            xlib::Button1,
            self.mouse_key_mask.bits() as u32 | xlib::ShiftMask,
        );
        self.grab_buttons(handle, xlib::Button3, self.mouse_key_mask.bits() as u32);
        self.grab_buttons(
            handle,
            xlib::Button3,
            self.mouse_key_mask.bits() as u32 | xlib::ShiftMask,
        );
    }

    /// Grabs the button with the modifier for a window.
    // `XGrabButton`: https://tronche.com/gui/x/xlib/input/XGrabButton.html
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
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                    0,
                    0,
                );
            }
        }
    }

    /// Cleans all currently grabbed buttons of a window.
    // `XUngrabButton`: https://tronche.com/gui/x/xlib/input/XUngrabButton.html
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
    // `XGrabPointer`: https://tronche.com/gui/x/xlib/input/XGrabPointer.html
    pub fn grab_pointer(&self, cursor: c_ulong) {
        unsafe {
            // grab the mouse
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
    // `XUngrabPointer`: https://tronche.com/gui/x/xlib/input/XUngrabPointer.html
    pub fn ungrab_pointer(&self) {
        unsafe {
            // release the mouse grab
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
    // `XWarpPointer`: https://tronche.com/gui/x/xlib/input/XWarpPointer.html
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
    // `XQueryPointer`: https://tronche.com/gui/x/xlib/window-information/XQueryPointer.html
    pub fn replay_click(&self, focused_window: xlib::Window, button: c_uint) {
        unsafe {
            let mut event: xlib::XButtonEvent = std::mem::zeroed();
            event.button = button;
            event.same_screen = xlib::True;
            event.subwindow = self.get_default_root();

            while event.subwindow != 0 {
                event.window = event.subwindow;
                (self.xlib.XQueryPointer)(
                    self.display,
                    event.window,
                    &mut event.root,
                    &mut event.subwindow,
                    &mut event.x_root,
                    &mut event.y_root,
                    &mut event.x,
                    &mut event.y,
                    &mut event.state,
                );
            }

            // Make sure we are clicking on the focused window. This also prevents clicks when
            // focus is changed by a keybind.
            if event.window == focused_window {
                event.type_ = xlib::ButtonPress;
                self.send_xevent(event.window, 0, xlib::ButtonPressMask, &mut event.into());

                event.type_ = xlib::ButtonRelease;
                self.send_xevent(event.window, 0, xlib::ButtonReleaseMask, &mut event.into());
            }
        }
    }

    /// Release the pointer if it is frozen.
    // `XAllowEvents`: https://linux.die.net/man/3/xallowevents
    pub fn allow_pointer_events(&self) {
        unsafe { (self.xlib.XAllowEvents)(self.display, xlib::SyncPointer, xlib::CurrentTime) };
    }
}
