//! Xlib calls related to a window.

use leftwm_core::{
    models::{WindowChange, WindowHandle, WindowType, Xyhw},
    DisplayEvent, Window,
};
use x11rb::{protocol::xproto, x11_utils::Serialize};

use crate::xatom::WMStateWindowState;
use crate::{error::Result, X11rbWindowHandle};

use super::{root_event_mask, XWrap};

impl XWrap {
    /// Sets up a window before we manage it.
    pub fn setup_window(
        &self,
        window: xproto::Window,
    ) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
        // Check that the window isn't requesting to be unmanaged
        let attrs = self.get_window_attrs(window)?;
        if attrs.override_redirect || self.managed_windows.contains(&window) {
            return Ok(None);
        }
        let handle = WindowHandle(X11rbWindowHandle(window));
        // Gather info about the window from xlib.
        let name = self.get_window_name(window)?;
        let legacy_name = self.get_window_legacy_name(window)?;
        let class = self.get_window_class(window)?;
        let pid = self.get_window_pid(window)?;
        let r#type = self.get_window_type(window)?;
        let states = self.get_window_states(window)?;
        let actions = self.get_window_actions_atoms(window)?;
        let mut can_resize = actions.contains(&self.atoms.NetWMActionResize);
        let trans = self.get_transient_for(window)?;
        let sizing_hint = self.get_hint_sizing_as_xyhw(window)?;
        let wm_hint = self.get_wmhints(window)?;

        // Build the new window, and fill in info about it.
        let mut w = Window::new(handle, Some(name), Some(pid));
        w.res_name = class
            .as_ref()
            .map(|c| String::from_utf8(c.instance().to_vec()).ok())
            .flatten();
        w.res_class = class
            .map(|c| String::from_utf8(c.class().to_vec()).ok())
            .flatten();
        w.legacy_name = Some(legacy_name);
        w.r#type = r#type.clone();
        w.states = states;
        w.transient = trans.map(|h| WindowHandle(X11rbWindowHandle(window)));
        // // Initialise the windows floating with the pre-mapped settings.
        // let xyhw = XyhwChange {
        //     x: Some(attrs.x),
        //     y: Some(attrs.y),
        //     w: Some(attrs.width),
        //     h: Some(attrs.height),
        //     ..XyhwChange::default()
        // };
        // xyhw.update_window_floating(&mut w);
        sizing_hint
            .unwrap_or_default()
            .update_window_floating(&mut w);
        let mut requested = Xyhw::default();
        // xyhw.update(&mut requested);
        sizing_hint.unwrap_or_default().update(&mut requested);

        if let Some(hint) = sizing_hint {
            // Ignore this for now for non-splashes as it causes issues, e.g. mintstick is non-resizable but is too
            // small, issue #614: https://github.com/leftwm/leftwm/issues/614.
            can_resize = match (r#type, hint.minw, hint.minh, hint.maxw, hint.maxh) {
                (
                    WindowType::Splash,
                    Some(min_width),
                    Some(min_height),
                    Some(max_width),
                    Some(max_height),
                ) => can_resize || min_width != max_width || min_height != max_height,
                _ => true,
            };
            // Use the pre-mapped sizes if they are bigger.
            // hint.w = std::cmp::max(xyhw.w, hint.w);
            // hint.h = std::cmp::max(xyhw.h, hint.h);
            hint.update_window_floating(&mut w);
            hint.update(&mut requested);
        }

        w.requested = Some(requested);
        w.can_resize = can_resize;
        if let Some(hint) = wm_hint {
            w.never_focus = hint.input.unwrap_or(false);
            w.urgent = hint.urgent;
        }
        // Is this needed? Made it so it doens't overwrite prior sizing.
        if w.floating() && sizing_hint.is_none() {
            let geo = self.get_window_geometry(window)?;
            geo.update_window_floating(&mut w);
        }

        let cursor = self.get_cursor_point()?;
        Ok(Some(DisplayEvent::WindowCreate(w, cursor.0, cursor.1)))
    }

    /// Sets up a window that we want to manage.
    // `XMapWindow`: https://tronche.com/gui/x/xlib/window/XMapWindow.html
    pub fn setup_managed_window(
        &mut self,
        h: WindowHandle<X11rbWindowHandle>,
        floating: bool,
        follow_mouse: bool,
    ) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
        let WindowHandle(X11rbWindowHandle(handle)) = h else {
            return Ok(None);
        };
        self.subscribe_to_window_events(handle)?;
        self.managed_windows.push(handle);

        // Make sure the window is mapped.
        xproto::map_window(&self.conn, handle)?;

        // Let Xlib know we are managing this window.
        self.append_property_u32(
            self.root,
            self.atoms.NetClientList,
            xproto::AtomEnum::ATOM.into(),
            &[handle],
        )?;

        // Make sure there is at least an empty list of _NET_WM_STATE.
        let states = self.get_window_states_atoms(handle)?;
        self.set_window_states_atoms(handle, &states)?;

        // Set WM_STATE to normal state to allow window sharing.
        self.set_wm_state(handle, WMStateWindowState::Normal)?;

        let r#type = self.get_window_type(handle)?;
        if r#type == WindowType::Dock || r#type == WindowType::Desktop {
            if let Some(dock_area) = self.get_window_strut_array(handle)? {
                let dems = self.get_screens_area_dimensions()?;
                let Some(screen) = self
                    .get_screens()?
                    .iter()
                    .find(|s| s.contains_dock_area(dock_area, dems))
                    .cloned()
                else {
                    return Ok(None);
                };

                if let Some(xyhw) = dock_area.as_xyhw(dems.0, dems.1, &screen) {
                    let mut change = WindowChange::new(h);
                    change.strut = Some(xyhw.into());
                    change.r#type = Some(r#type);
                    return Ok(Some(DisplayEvent::WindowChange(change)));
                }
            } else {
                let geo = self.get_window_geometry(handle)?;
                let mut xyhw = Xyhw::default();
                geo.update(&mut xyhw);
                let mut change = WindowChange::new(h);
                change.strut = Some(xyhw.into());
                change.r#type = Some(r#type);
                return Ok(Some(DisplayEvent::WindowChange(change)));
            }
        } else {
            let color = if floating {
                self.colors.floating
            } else {
                self.colors.normal
            };
            self.set_window_border_color(handle, color)?;

            if follow_mouse {
                self.move_cursor_to_window(handle)?;
            }
            if self.focus_behaviour.is_clickto() {
                self.grab_mouse_clicks(handle, false)?;
            }
        }
        Ok(None)
    }

    /// Teardown a managed window when it is destroyed.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn teardown_managed_window(
        &mut self,
        h: &WindowHandle<X11rbWindowHandle>,
        destroyed: bool,
    ) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(handle)) = h {
            self.managed_windows.retain(|x| *x != *handle);
            if !destroyed {
                xproto::grab_server(&self.conn)?;
                self.ungrab_buttons(*handle)?;
                self.set_wm_state(*handle, WMStateWindowState::Withdrawn)?;
                self.sync()?;
                xproto::ungrab_server(&self.conn)?;
            }
            self.set_client_list()?;
        }
        Ok(())
    }

    /// Updates a window.
    pub fn update_window(&self, window: &Window<X11rbWindowHandle>) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(handle)) = window.handle {
            if window.visible() {
                let changes = xproto::ConfigureWindowAux {
                    x: Some(window.x()),
                    y: Some(window.y()),
                    width: Some(window.width() as u32),
                    height: Some(window.height() as u32),
                    border_width: Some(window.border() as u32),
                    ..Default::default()
                };
                self.set_window_config(handle, &changes)?;
                self.configure_window(window)?;
            }
            let (state, _) = self.get_wm_state(handle)?;
            // Only change when needed. This prevents task bar icons flashing (especially with steam).
            if window.visible() && state != WMStateWindowState::Normal {
                self.toggle_window_visibility(handle, true)?;
            } else if !window.visible() && state != WMStateWindowState::Iconic {
                self.toggle_window_visibility(handle, false)?;
            }
        }
        Ok(())
    }

    /// Maps and unmaps a window depending on it is visible.
    pub fn toggle_window_visibility(&self, window: xproto::Window, visible: bool) -> Result<()> {
        // We don't want to receive this map or unmap event.
        let mask_off = root_event_mask().remove(xproto::EventMask::SUBSTRUCTURE_NOTIFY);
        let mut attrs = xproto::ChangeWindowAttributesAux {
            event_mask: Some(mask_off),
            ..Default::default()
        };
        xproto::change_window_attributes(&self.conn, self.root, &attrs)?;
        if visible {
            // Set WM_STATE to normal state.
            self.set_wm_state(window, WMStateWindowState::Normal)?;
            // Make sure the window is mapped.
            xproto::map_window(&self.conn, window)?;
            // Regrab the mouse clicks but ignore `dock` windows as some don't handle click events put on them
            if self.focus_behaviour.is_clickto()
                && self.get_window_type(window)? != WindowType::Dock
            {
                self.grab_mouse_clicks(window, false)?;
            }
        } else {
            // Ungrab the mouse clicks.
            self.ungrab_buttons(window)?;
            // Make sure the window is unmapped.
            xproto::unmap_window(&self.conn, window)?;
            // Set WM_STATE to iconic state.
            self.set_wm_state(window, WMStateWindowState::Iconic)?;
        }
        attrs.event_mask = Some(root_event_mask());
        xproto::change_window_attributes(&self.conn, self.root, &attrs)?;
        Ok(())
    }

    /// Makes a window take focus.
    pub fn window_take_focus(
        &mut self,
        window: &Window<X11rbWindowHandle>,
        previous: Option<&Window<X11rbWindowHandle>>,
    ) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(handle)) = window.handle {
            // Update previous window.
            if let Some(previous) = previous {
                if let WindowHandle(X11rbWindowHandle(previous_handle)) = previous.handle {
                    let color = if previous.floating() {
                        self.colors.floating
                    } else {
                        self.colors.normal
                    };
                    self.set_window_border_color(previous_handle, color)?;
                    // Open up button1 clicking on the previously focused window.
                    if self.focus_behaviour.is_clickto() {
                        self.grab_mouse_clicks(previous_handle, false)?;
                    }
                }
            }
            self.focused_window = handle;
            self.grab_mouse_clicks(handle, true)?;
            self.set_window_urgency(handle, false)?;
            self.set_window_border_color(handle, self.colors.active)?;
            self.focus(handle, window.never_focus)?;
            self.sync()?;
        }
        Ok(())
    }

    /// Focuses a window.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn focus(&mut self, window: xproto::Window, never_focus: bool) -> Result<()> {
        if !never_focus {
            xproto::set_input_focus(
                &self.conn,
                xproto::InputFocus::POINTER_ROOT,
                window,
                x11rb::CURRENT_TIME,
            )?;
            self.replace_property_u32(
                window,
                self.atoms.NetActiveWindow,
                xproto::AtomEnum::ATOM.into(),
                &[window],
            )?;
        }
        // Tell the window to take focus
        self.send_xevent_atom(window, self.atoms.WMTakeFocus)?;
        Ok(())
    }

    /// Unfocuses all windows.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn unfocus(
        &self,
        handle: Option<WindowHandle<X11rbWindowHandle>>,
        floating: bool,
    ) -> Result<()> {
        if let Some(WindowHandle(X11rbWindowHandle(handle))) = handle {
            let color = if floating {
                self.colors.floating
            } else {
                self.colors.normal
            };
            self.set_window_border_color(handle, color)?;

            self.grab_mouse_clicks(handle, false)?;
        }
        xproto::set_input_focus(
            &self.conn,
            xproto::InputFocus::POINTER_ROOT,
            self.root,
            x11rb::CURRENT_TIME,
        )?;
        self.replace_property_u32(
            self.root,
            self.atoms.NetActiveWindow,
            xproto::AtomEnum::WINDOW.into(),
            &[x11rb::NONE],
        )
    }

    /// Send a `XConfigureEvent` for a window to X.
    pub fn configure_window(&self, window: &Window<X11rbWindowHandle>) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(handle)) = window.handle {
            let configure_event = xproto::ConfigureNotifyEvent {
                event: handle,
                window: handle,
                x: window.x() as i16,
                y: window.y() as i16,
                width: window.width() as u16,
                height: window.height() as u16,
                border_width: window.border() as u16,
                above_sibling: x11rb::NONE,
                override_redirect: false,
                ..Default::default()
            };
            self.send_xevent(
                handle,
                false,
                xproto::EventMask::STRUCTURE_NOTIFY,
                &configure_event.serialize(),
            )?;
        }
        Ok(())
    }

    /// Change a windows attributes.
    // `XChangeWindowAttributes`: https://tronche.com/gui/x/xlib/window/XChangeWindowAttributes.html
    // TODO: Is this method really useful ?
    pub fn change_window_attributes(
        &self,
        window: xproto::Window,
        attrs: &xproto::ChangeWindowAttributesAux,
    ) -> Result<()> {
        xproto::change_window_attributes(&self.conn, window, attrs)?;
        Ok(())
    }

    /// Restacks the windows to the order of the vec.
    // `XRestackWindows`: https://tronche.com/gui/x/xlib/window/XRestackWindows.html
    pub fn restack(&self, handles: Vec<WindowHandle<X11rbWindowHandle>>) -> Result<()> {
        let mut conf = xproto::ConfigureWindowAux::default();
        for i in 1..handles.len() {
            let Some(WindowHandle(X11rbWindowHandle(window))) = handles.get(i) else {
                continue;
            };

            conf.stack_mode = Some(xproto::StackMode::BELOW);
            conf.sibling = handles
                .get(i - 1)
                .copied()
                .map(|h| {
                    if let WindowHandle(X11rbWindowHandle(w)) = h {
                        Some(w)
                    } else {
                        None
                    }
                })
                .flatten();
            xproto::configure_window(&self.conn, *window, &conf)?;
        }
        Ok(())
    }

    pub fn move_resize_window(
        &self,
        window: xproto::Window,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) -> Result<()> {
        let attrs = xproto::ConfigureWindowAux {
            x: Some(x),
            y: Some(y),
            width: Some(w),
            height: Some(h),
            ..Default::default()
        };
        xproto::configure_window(&self.conn, window, &attrs)?;
        Ok(())
    }

    /// Raise a window.
    // `XRaiseWindow`: https://tronche.com/gui/x/xlib/window/XRaiseWindow.html
    pub fn move_to_top(&self, handle: &WindowHandle<X11rbWindowHandle>) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(window)) = handle {
            let attrs = xproto::ConfigureWindowAux {
                stack_mode: Some(xproto::StackMode::ABOVE),
                ..Default::default()
            };
            xproto::configure_window(&self.conn, *window, &attrs)?;
        }
        Ok(())
    }

    /// Kills a window.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XSetCloseDownMode`: https://tronche.com/gui/x/xlib/display/XSetCloseDownMode.html
    // `XKillClient`: https://tronche.com/gui/x/xlib/window-and-session-manager/XKillClient.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn kill_window(&self, h: &WindowHandle<X11rbWindowHandle>) -> Result<()> {
        if let WindowHandle(X11rbWindowHandle(handle)) = h {
            // Nicely ask the window to close.
            if !self.send_xevent_atom(*handle, self.atoms.WMDelete)? {
                // Force kill the window.
                xproto::grab_server(&self.conn)?;
                xproto::set_close_down_mode(&self.conn, xproto::CloseDown::DESTROY_ALL)?;
                xproto::kill_client(&self.conn, *handle)?;
                xproto::ungrab_server(&self.conn)?;
            }
        }
        Ok(())
    }

    /// Forcibly unmap a window.
    pub fn force_unmapped(&mut self, window: xproto::Window) -> Result<()> {
        let managed = self.managed_windows.contains(&window);
        if managed {
            self.managed_windows.retain(|x| *x != window);
            self.set_client_list()?;
        }
        Ok(())
    }

    /// Subscribe to an event of a window.
    // `XSelectInput`: https://tronche.com/gui/x/xlib/event-handling/XSelectInput.html
    pub fn subscribe_to_event(
        &self,
        window: xproto::Window,
        mask: xproto::EventMask,
    ) -> Result<()> {
        // In xlib `XSelectInput` "lock" the display with `XLockDisplay` when setting the event
        // mask, is this needed here ?
        let attrs = xproto::ChangeWindowAttributesAux {
            event_mask: Some(mask),
            ..Default::default()
        };
        xproto::change_window_attributes(&self.conn, window, &attrs)?;
        Ok(())
    }

    /// Subscribe to the wanted events of a window.
    pub fn subscribe_to_window_events(&self, window: xproto::Window) -> Result<()> {
        let mask = xproto::EventMask::ENTER_WINDOW
            | xproto::EventMask::FOCUS_CHANGE
            | xproto::EventMask::PROPERTY_CHANGE;
        self.subscribe_to_event(window, mask)
    }
}
