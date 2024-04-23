//! Xlib calls related to a mouse.
use x11rb::{protocol::xproto, x11_utils::Serialize};

use super::{button_event_mask, mouse_event_mask, XWrap};

use crate::error::Result;

impl XWrap {
    /// Grabs the mouse clicks of a window.
    pub fn grab_mouse_clicks(&self, handle: xproto::Window, is_focused: bool) -> Result<()> {
        self.ungrab_buttons(handle)?;
        if !is_focused {
            self.grab_buttons(handle, xproto::ButtonIndex::M1, xproto::ModMask::ANY)?;
            self.grab_buttons(handle, xproto::ButtonIndex::M3, xproto::ModMask::ANY)?;
        }
        let mouse_key_mask = xproto::ModMask::from(self.mouse_key_mask.bits());
        self.grab_buttons(handle, xproto::ButtonIndex::M1, mouse_key_mask)?;
        self.grab_buttons(
            handle,
            xproto::ButtonIndex::M1,
            mouse_key_mask | xproto::ModMask::SHIFT,
        )?;
        self.grab_buttons(handle, xproto::ButtonIndex::M3, mouse_key_mask)?;
        self.grab_buttons(
            handle,
            xproto::ButtonIndex::M3,
            mouse_key_mask | xproto::ModMask::SHIFT,
        )?;
        Ok(())
    }

    /// Grabs the button with the modifier for a window.
    pub fn grab_buttons(
        &self,
        window: xproto::Window,
        button: xproto::ButtonIndex,
        modifiers: xproto::ModMask,
    ) -> Result<()> {
        let mods: Vec<xproto::ModMask> = vec![
            modifiers,
            modifiers | xproto::ModMask::M2,
            modifiers | xproto::ModMask::LOCK,
        ];
        for m in mods {
            xproto::grab_button(
                &self.conn,
                false,
                window,
                button_event_mask(),
                xproto::GrabMode::ASYNC,
                xproto::GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                button,
                m,
            )?;
        }
        Ok(())
    }

    /// Cleans all currently grabbed buttons of a window.
    pub fn ungrab_buttons(&self, handle: xproto::Window) -> Result<()> {
        xproto::ungrab_button(
            &self.conn,
            xproto::ButtonIndex::ANY,
            handle,
            xproto::ModMask::ANY,
        )?;
        Ok(())
    }

    /// Grabs the cursor and sets its visual.
    pub fn grab_pointer(&self, cursor: xproto::Cursor) -> Result<()> {
        xproto::grab_pointer(
            &self.conn,
            false,
            self.root,
            mouse_event_mask(),
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
            x11rb::NONE,
            cursor,
            x11rb::CURRENT_TIME,
        )?;
        Ok(())
    }

    /// Ungrab the cursor.
    pub fn ungrab_pointer(&self) -> Result<()> {
        xproto::ungrab_pointer(&self.conn, x11rb::CURRENT_TIME)?;
        Ok(())
    }

    /// Move the cursor to a window.
    pub fn move_cursor_to_window(&self, window: xproto::Window) -> Result<()> {
        let geo = xproto::get_geometry(&self.conn, window)?.reply()?;
        let point = (
            i32::from(geo.x) + (i32::from(geo.width) / 2),
            i32::from(geo.y) + (i32::from(geo.height) / 2),
        );
        self.move_cursor_to_point(point)
    }

    /// Move the cursor to a point.
    pub fn move_cursor_to_point(&self, point: (i32, i32)) -> Result<()> {
        if point.0 >= 0 && point.1 >= 0 {
            xproto::warp_pointer(
                &self.conn,
                x11rb::NONE,
                self.root,
                0,
                0,
                0,
                0,
                i16::try_from(point.0)?,
                i16::try_from(point.1)?,
            )?;
        }
        Ok(())
    }

    /// Replay a click on a window.
    pub fn replay_click(
        &self,
        focused_window: xproto::Window,
        button: xproto::Button,
    ) -> Result<()> {
        let mut event = xproto::ButtonPressEvent {
            child: self.get_default_root(),
            detail: button,
            same_screen: true,
            ..Default::default()
        };
        event.child = self.get_default_root();
        loop {
            let reply = xproto::query_pointer(&self.conn, event.child)?.reply()?;
            if reply.child != x11rb::NONE {
                break;
            }
            event.child = reply.child;
            event.root = reply.root;
            event.root_x = reply.root_x;
            event.root_y = reply.root_y;
            event.event_x = reply.win_x;
            event.event_y = reply.win_y;
        }

        if event.child == focused_window {
            event.state = xproto::KeyButMask::BUTTON1;
            self.send_xevent(
                event.child,
                false,
                xproto::EventMask::BUTTON_PRESS,
                &event.serialize(),
            )?;

            event.state.remove(xproto::KeyButMask::BUTTON1);
            self.send_xevent(
                event.child,
                false,
                xproto::EventMask::BUTTON_RELEASE,
                &event.serialize(),
            )?;
        }
        Ok(())
    }
}
