//! Xlib calls related to a window.
use super::{
    on_error_from_xlib, on_error_from_xlib_dummy, Window, WindowHandle, ICONIC_STATE, NORMAL_STATE,
    ROOT_EVENT_MASK, WITHDRAWN_STATE,
};
use crate::{XWrap, XlibWindowHandle};
use leftwm_core::models::{WindowChange, WindowType, Xyhw, XyhwChange};
use leftwm_core::DisplayEvent;
use std::os::raw::{c_long, c_ulong};
use x11_dl::xlib;

impl XWrap {
    /// Sets up a window before we manage it.
    #[must_use]
    pub fn setup_window(&self, window: xlib::Window) -> Option<DisplayEvent<XlibWindowHandle>> {
        // Check that the window isn't requesting to be unmanaged
        let attrs = match self.get_window_attrs(window) {
            Ok(attr) if attr.override_redirect == 0 && !self.managed_windows.contains(&window) => {
                attr
            }
            _ => return None,
        };
        let handle = WindowHandle(XlibWindowHandle(window));
        // Gather info about the window from xlib.
        let name = self.get_window_name(window);
        let legacy_name = self.get_window_legacy_name(window);
        let class = self.get_window_class(window);
        let pid = self.get_window_pid(window);
        let r#type = self.get_window_type(window);
        let states = self.get_window_states(window);
        let actions = self.get_window_actions_atoms(window);
        let mut can_resize = actions.contains(&self.atoms.NetWMActionResize);
        let trans = self.get_transient_for(window);
        let sizing_hint = self.get_hint_sizing_as_xyhw(window);
        let wm_hint = self.get_wmhints(window);

        // Build the new window, and fill in info about it.
        let mut w = Window::new(handle, name, pid);
        if let Some((res_name, res_class)) = class {
            w.res_name = Some(res_name);
            w.res_class = Some(res_class);
        }
        w.legacy_name = legacy_name;
        w.r#type = r#type.clone();
        w.states = states;
        if let Some(trans) = trans {
            w.transient = Some(WindowHandle(XlibWindowHandle(trans)));
        }
        // Initialise the windows floating with the pre-mapped settings.
        let xyhw = XyhwChange {
            x: Some(attrs.x),
            y: Some(attrs.y),
            w: Some(attrs.width),
            h: Some(attrs.height),
            ..XyhwChange::default()
        };
        xyhw.update_window_floating(&mut w);
        let mut requested = Xyhw::default();
        xyhw.update(&mut requested);
        if let Some(mut hint) = sizing_hint {
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
            hint.w = std::cmp::max(xyhw.w, hint.w);
            hint.h = std::cmp::max(xyhw.h, hint.h);
            hint.update_window_floating(&mut w);
            hint.update(&mut requested);
        }
        w.requested = Some(requested);
        w.can_resize = can_resize;
        if let Some(hint) = wm_hint {
            w.never_focus = hint.flags & xlib::InputHint != 0 && hint.input == 0;
        }
        if let Some(hint) = wm_hint {
            w.urgent = hint.flags & xlib::XUrgencyHint != 0;
        }
        // Is this needed? Made it so it doens't overwrite prior sizing.
        if w.floating() && sizing_hint.is_none() {
            if let Ok(geo) = self.get_window_geometry(window) {
                geo.update_window_floating(&mut w);
            }
        }

        let cursor = self.get_cursor_point().unwrap_or_default();
        Some(DisplayEvent::WindowCreate(w, cursor.0, cursor.1))
    }

    /// Sets up a window that we want to manage.
    // `XMapWindow`: https://tronche.com/gui/x/xlib/window/XMapWindow.html
    pub fn setup_managed_window(
        &mut self,
        h: WindowHandle<XlibWindowHandle>,
        floating: bool,
        follow_mouse: bool,
    ) -> Option<DisplayEvent<XlibWindowHandle>> {
        let WindowHandle(XlibWindowHandle(handle)) = h;
        self.subscribe_to_window_events(handle);
        self.managed_windows.push(handle);
        // Make sure the window is mapped.
        unsafe { (self.xlib.XMapWindow)(self.display, handle) };
        // Let Xlib know we are managing this window.
        let list = vec![handle as c_long];
        self.append_property_long(self.root, self.atoms.NetClientList, xlib::XA_WINDOW, &list);

        // Make sure there is at least an empty list of _NET_WM_STATE.
        let states = self.get_window_states_atoms(handle);
        self.set_window_states_atoms(handle, &states);
        // Set WM_STATE to normal state to allow window sharing.
        self.set_wm_states(handle, &[NORMAL_STATE]);

        let r#type = self.get_window_type(handle);
        if r#type == WindowType::Dock || r#type == WindowType::Desktop {
            if let Some(dock_area) = self.get_window_strut_array(handle) {
                let dems = self.get_screens_area_dimensions();
                let screen = self
                    .get_screens()
                    .iter()
                    .find(|s| s.contains_dock_area(dock_area, dems))?
                    .clone();

                if let Some(xyhw) = dock_area.as_xyhw(dems.0, dems.1, &screen) {
                    let mut change = WindowChange::new(h);
                    change.strut = Some(xyhw.into());
                    change.r#type = Some(r#type);
                    return Some(DisplayEvent::WindowChange(change));
                }
            } else if let Ok(geo) = self.get_window_geometry(handle) {
                let mut xyhw = Xyhw::default();
                geo.update(&mut xyhw);
                let mut change = WindowChange::new(h);
                change.strut = Some(xyhw.into());
                change.r#type = Some(r#type);
                return Some(DisplayEvent::WindowChange(change));
            }
        } else {
            let color = if floating {
                self.colors.floating
            } else {
                self.colors.normal
            };
            self.set_window_border_color(handle, color);

            if follow_mouse {
                _ = self.move_cursor_to_window(handle);
            }
            if self.focus_behaviour.is_clickto() {
                self.grab_mouse_clicks(handle, false);
            }
        }
        None
    }

    /// Teardown a managed window when it is destroyed.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn teardown_managed_window(&mut self, h: &WindowHandle<XlibWindowHandle>, destroyed: bool) {
        if let WindowHandle(XlibWindowHandle(handle)) = h {
            self.managed_windows.retain(|x| *x != *handle);
            if !destroyed {
                unsafe {
                    (self.xlib.XGrabServer)(self.display);
                    (self.xlib.XSetErrorHandler)(Some(on_error_from_xlib_dummy));
                    self.ungrab_buttons(*handle);
                    self.set_wm_states(*handle, &[WITHDRAWN_STATE]);
                    self.sync();
                    (self.xlib.XSetErrorHandler)(Some(on_error_from_xlib));
                    (self.xlib.XUngrabServer)(self.display);
                }
            }
            self.set_client_list();
        }
    }

    /// Updates a window.
    pub fn update_window(&self, window: &Window<XlibWindowHandle>) {
        if let WindowHandle(XlibWindowHandle(handle)) = window.handle {
            if window.visible() {
                let changes = xlib::XWindowChanges {
                    x: window.x(),
                    y: window.y(),
                    width: window.width(),
                    height: window.height(),
                    border_width: window.border(),
                    sibling: 0,    // Not unlocked.
                    stack_mode: 0, // Not unlocked.
                };
                let unlock =
                    xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth;
                self.set_window_config(handle, changes, u32::from(unlock));
                self.configure_window(window);
            }
            let Some(state) = self.get_wm_state(handle) else {
                return;
            };
            // Only change when needed. This prevents task bar icons flashing (especially with steam).
            if window.visible() && state != NORMAL_STATE {
                self.toggle_window_visibility(handle, true);
            } else if !window.visible() && state != ICONIC_STATE {
                self.toggle_window_visibility(handle, false);
            }
        }
    }

    /// Maps and unmaps a window depending on it is visible.
    pub fn toggle_window_visibility(&self, window: xlib::Window, visible: bool) {
        // We don't want to receive this map or unmap event.
        let mask_off = ROOT_EVENT_MASK & !(xlib::SubstructureNotifyMask);
        let mut attrs: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attrs.event_mask = mask_off;
        self.change_window_attributes(self.root, xlib::CWEventMask, attrs);
        if visible {
            // Set WM_STATE to normal state.
            self.set_wm_states(window, &[NORMAL_STATE]);
            // Make sure the window is mapped.
            unsafe { (self.xlib.XMapWindow)(self.display, window) };
            // Regrab the mouse clicks but ignore `dock` windows as some don't handle click events put on them
            if self.focus_behaviour.is_clickto() && self.get_window_type(window) != WindowType::Dock
            {
                self.grab_mouse_clicks(window, false);
            }
        } else {
            // Ungrab the mouse clicks.
            self.ungrab_buttons(window);
            // Make sure the window is unmapped.
            unsafe { (self.xlib.XUnmapWindow)(self.display, window) };
            // Set WM_STATE to iconic state.
            self.set_wm_states(window, &[ICONIC_STATE]);
        }
        attrs.event_mask = ROOT_EVENT_MASK;
        self.change_window_attributes(self.root, xlib::CWEventMask, attrs);
    }

    /// Makes a window take focus.
    pub fn window_take_focus(
        &mut self,
        window: &Window<XlibWindowHandle>,
        previous: Option<&Window<XlibWindowHandle>>,
    ) {
        if let WindowHandle(XlibWindowHandle(handle)) = window.handle {
            // Update previous window.
            if let Some(previous) = previous {
                if let WindowHandle(XlibWindowHandle(previous_handle)) = previous.handle {
                    let color = if previous.floating() {
                        self.colors.floating
                    } else {
                        self.colors.normal
                    };
                    self.set_window_border_color(previous_handle, color);
                    // Open up button1 clicking on the previously focused window.
                    if self.focus_behaviour.is_clickto() {
                        self.grab_mouse_clicks(previous_handle, false);
                    }
                }
            }
            self.focused_window = handle;
            self.grab_mouse_clicks(handle, true);
            self.set_window_urgency(handle, false);
            self.set_window_border_color(handle, self.colors.active);
            self.focus(handle, window.never_focus);
            self.sync();
        }
    }

    /// Focuses a window.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn focus(&mut self, window: xlib::Window, never_focus: bool) {
        if !never_focus {
            unsafe {
                (self.xlib.XSetInputFocus)(
                    self.display,
                    window,
                    xlib::RevertToPointerRoot,
                    xlib::CurrentTime,
                );
                let list = vec![window as c_long];
                // Mark this window as the `_NET_ACTIVE_WINDOW`
                self.replace_property_long(
                    self.root,
                    self.atoms.NetActiveWindow,
                    xlib::XA_WINDOW,
                    &list,
                );
                std::mem::forget(list);
            }
        }
        // Tell the window to take focus
        self.send_xevent_atom(window, self.atoms.WMTakeFocus);
    }

    /// Unfocuses all windows.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn unfocus(&self, handle: Option<WindowHandle<XlibWindowHandle>>, floating: bool) {
        if let Some(WindowHandle(XlibWindowHandle(handle))) = handle {
            let color = if floating {
                self.colors.floating
            } else {
                self.colors.normal
            };
            self.set_window_border_color(handle, color);

            self.grab_mouse_clicks(handle, false);
        }
        unsafe {
            (self.xlib.XSetInputFocus)(
                self.display,
                self.root,
                xlib::RevertToPointerRoot,
                xlib::CurrentTime,
            );
            self.replace_property_long(
                self.root,
                self.atoms.NetActiveWindow,
                xlib::XA_WINDOW,
                &[c_long::MAX],
            );
        }
    }

    /// Send a `XConfigureEvent` for a window to X.
    pub fn configure_window(&self, window: &Window<XlibWindowHandle>) {
        if let WindowHandle(XlibWindowHandle(handle)) = window.handle {
            let mut configure_event: xlib::XConfigureEvent = unsafe { std::mem::zeroed() };
            configure_event.type_ = xlib::ConfigureNotify;
            configure_event.display = self.display;
            configure_event.event = handle;
            configure_event.window = handle;
            configure_event.x = window.x();
            configure_event.y = window.y();
            configure_event.width = window.width();
            configure_event.height = window.height();
            configure_event.border_width = window.border;
            configure_event.above = 0;
            configure_event.override_redirect = 0;
            self.send_xevent(
                handle,
                0,
                xlib::StructureNotifyMask,
                &mut configure_event.into(),
            );
        }
    }

    /// Change a windows attributes.
    // `XChangeWindowAttributes`: https://tronche.com/gui/x/xlib/window/XChangeWindowAttributes.html
    pub fn change_window_attributes(
        &self,
        window: xlib::Window,
        mask: c_ulong,
        mut attrs: xlib::XSetWindowAttributes,
    ) {
        unsafe {
            (self.xlib.XChangeWindowAttributes)(self.display, window, mask, &mut attrs);
        }
    }

    /// Restacks the windows to the order of the vec.
    // `XRestackWindows`: https://tronche.com/gui/x/xlib/window/XRestackWindows.html
    pub fn restack(&self, handles: Vec<WindowHandle<XlibWindowHandle>>) {
        let mut windows = vec![];
        for handle in handles {
            if let WindowHandle(XlibWindowHandle(window)) = handle {
                windows.push(window);
            }
        }
        let size = windows.len();
        let ptr = windows.as_mut_ptr();
        unsafe {
            (self.xlib.XRestackWindows)(self.display, ptr, size as i32);
        }
    }

    pub fn move_resize_window(&self, window: xlib::Window, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            (self.xlib.XMoveResizeWindow)(self.display, window, x, y, w, h);
        }
    }

    /// Raise a window.
    // `XRaiseWindow`: https://tronche.com/gui/x/xlib/window/XRaiseWindow.html
    pub fn move_to_top(&self, handle: &WindowHandle<XlibWindowHandle>) {
        if let WindowHandle(XlibWindowHandle(window)) = handle {
            unsafe {
                (self.xlib.XRaiseWindow)(self.display, *window);
            }
        }
    }

    /// Kills a window.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XSetCloseDownMode`: https://tronche.com/gui/x/xlib/display/XSetCloseDownMode.html
    // `XKillClient`: https://tronche.com/gui/x/xlib/window-and-session-manager/XKillClient.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn kill_window(&self, h: &WindowHandle<XlibWindowHandle>) {
        if let WindowHandle(XlibWindowHandle(handle)) = h {
            // Nicely ask the window to close.
            if !self.send_xevent_atom(*handle, self.atoms.WMDelete) {
                // Force kill the window.
                unsafe {
                    (self.xlib.XGrabServer)(self.display);
                    (self.xlib.XSetErrorHandler)(Some(on_error_from_xlib_dummy));
                    (self.xlib.XSetCloseDownMode)(self.display, xlib::DestroyAll);
                    (self.xlib.XKillClient)(self.display, *handle);
                    self.sync();
                    (self.xlib.XSetErrorHandler)(Some(on_error_from_xlib));
                    (self.xlib.XUngrabServer)(self.display);
                }
            }
        }
    }

    /// Forcibly unmap a window.
    pub fn force_unmapped(&mut self, window: xlib::Window) {
        let managed = self.managed_windows.contains(&window);
        if managed {
            self.managed_windows.retain(|x| *x != window);
            self.set_client_list();
        }
    }

    /// Subscribe to an event of a window.
    // `XSelectInput`: https://tronche.com/gui/x/xlib/event-handling/XSelectInput.html
    pub fn subscribe_to_event(&self, window: xlib::Window, mask: c_long) {
        unsafe { (self.xlib.XSelectInput)(self.display, window, mask) };
    }

    /// Subscribe to the wanted events of a window.
    pub fn subscribe_to_window_events(&self, window: xlib::Window) {
        let mask = xlib::EnterWindowMask | xlib::FocusChangeMask | xlib::PropertyChangeMask;
        self.subscribe_to_event(window, mask);
    }
}
