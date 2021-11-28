//! Xlib calls related to a window.
use super::{Window, WindowHandle, NORMAL_STATE};
use crate::models::{FocusBehaviour, WindowChange, WindowType, Xyhw};
use crate::{DisplayEvent, XWrap};
use std::os::raw::{c_long, c_ulong};
use x11_dl::xlib;

impl XWrap {
    /// Sets up a window that we want to manage.
    // `XMapWindow`: https://tronche.com/gui/x/xlib/window/XMapWindow.html
    // `XSync`: https://tronche.com/gui/x/xlib/event-handling/XSync.html
    pub fn setup_managed_window(
        &mut self,
        h: WindowHandle,
        follow_mouse: bool,
    ) -> Option<DisplayEvent> {
        self.subscribe_to_window_events(&h);
        if let WindowHandle::XlibHandle(handle) = h {
            self.managed_windows.push(handle);
            unsafe {
                // Make sure the window is mapped.
                (self.xlib.XMapWindow)(self.display, handle);

                // Let Xlib know we are managing this window.
                let list = vec![handle as c_long];
                self.append_property_long(
                    self.root,
                    self.atoms.NetClientList,
                    xlib::XA_WINDOW,
                    &list,
                );
                std::mem::forget(list);

                (self.xlib.XSync)(self.display, 0);
            }

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
                if follow_mouse {
                    let _ = self.move_cursor_to_window(handle);
                }
                if self.focus_behaviour == FocusBehaviour::ClickTo {
                    self.ungrab_buttons(handle);
                    self.grab_buttons(handle, xlib::Button1, xlib::AnyModifier);
                }
            }
            // Make sure there is at least an empty list of _NET_WM_STATE.
            let states = self.get_window_states_atoms(handle);
            self.set_window_states_atoms(handle, &states);
            // Set WM_STATE to normal state to allow window sharing.
            self.set_wm_states(handle, &[NORMAL_STATE]);
        }
        None
    }

    /// Teardown a managed window when it is destroyed.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XSync`: https://tronche.com/gui/x/xlib/event-handling/XSync.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn teardown_managed_window(&mut self, h: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            unsafe {
                (self.xlib.XGrabServer)(self.display);
                self.managed_windows.retain(|x| *x != *handle);
                self.set_client_list();
                self.ungrab_buttons(*handle);
                (self.xlib.XSync)(self.display, 0);
                (self.xlib.XUngrabServer)(self.display);
            }
        }
    }

    /// Updates a window.
    // `XMoveWindow`: https://tronche.com/gui/x/xlib/window/XMoveWindow.html
    // `XConfigureWindow`: https://tronche.com/gui/x/xlib/window/XConfigureWindow.html
    // `XSync`: https://tronche.com/gui/x/xlib/event-handling/XSync.html
    // `XMoveResizeWindow`: https://tronche.com/gui/x/xlib/window/XMoveResizeWindow.html
    // `XSetWindowBorder`: https://tronche.com/gui/x/xlib/window/XSetWindowBorder.html
    pub fn update_window(&self, window: &Window, is_focused: bool, hide_offset: i32) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            if window.visible() {
                // If type dock we only need to move it.
                // Also fixes issues with eww.
                if window.is_unmanaged() {
                    unsafe {
                        (self.xlib.XMoveWindow)(self.display, handle, window.x(), window.y());
                    }
                    return;
                }
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
                let w: u32 = window.width() as u32;
                let h: u32 = window.height() as u32;
                self.move_resize_window(handle, window.x(), window.y(), w, h);
                unsafe {
                    let mut color: c_ulong = if is_focused {
                        self.colors.active
                    } else if window.floating() {
                        self.colors.floating
                    } else {
                        self.colors.normal
                    };
                    // Force border opacity to 0xff.
                    let mut bytes = color.to_le_bytes();
                    bytes[3] = 0xff;
                    color = c_ulong::from_le_bytes(bytes);

                    (self.xlib.XSetWindowBorder)(self.display, handle, color);
                }
                if !is_focused && self.focus_behaviour == FocusBehaviour::ClickTo {
                    self.ungrab_buttons(handle);
                    self.grab_buttons(handle, xlib::Button1, xlib::AnyModifier);
                }
                self.send_config(window);
            } else {
                unsafe {
                    // If not visible window is placed of screen.
                    (self.xlib.XMoveWindow)(self.display, handle, hide_offset, window.y());
                }
            }
        }
    }

    /// Makes a window take focus.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn window_take_focus(&mut self, window: &Window) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            // Play a click when in ClickToFocus.
            if self.focus_behaviour == FocusBehaviour::ClickTo {
                self.play_click(handle);
                self.click_event = None;
            }
            self.grab_mouse_clicks(handle);

            if !window.never_focus {
                // Mark this window as the `_NET_ACTIVE_WINDOW`
                unsafe {
                    (self.xlib.XSetInputFocus)(
                        self.display,
                        handle,
                        xlib::RevertToPointerRoot,
                        xlib::CurrentTime,
                    );
                    let list = vec![handle as c_long];
                    self.replace_property_long(
                        self.root,
                        self.atoms.NetActiveWindow,
                        xlib::XA_WINDOW,
                        &list,
                    );
                    std::mem::forget(list);
                }
            }
            // This fixes windows that process the `WMTakeFocus` event too slow.
            // See: https://github.com/leftwm/leftwm/pull/563
            if self.focus_behaviour != FocusBehaviour::Sloppy {
                // Tell the window to take focus
                self.send_xevent_atom(handle, self.atoms.WMTakeFocus);
            }
        }
    }

    /// Unfocuses all windows.
    // `XSetInputFocus`: https://tronche.com/gui/x/xlib/input/XSetInputFocus.html
    pub fn unfocus(&self) {
        let handle = self.root;
        unsafe {
            (self.xlib.XSetInputFocus)(self.display, handle, xlib::RevertToNone, xlib::CurrentTime);
            self.replace_property_long(
                self.root,
                self.atoms.NetActiveWindow,
                xlib::XA_WINDOW,
                &[c_long::MAX],
            );
        }
    }

    /// Restacks the windows to the order of the vec.
    // `XRestackWindows`: https://tronche.com/gui/x/xlib/window/XRestackWindows.html
    pub fn restack(&self, handles: Vec<WindowHandle>) {
        let mut windows = vec![];
        for handle in handles {
            if let WindowHandle::XlibHandle(window) = handle {
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
    pub fn move_to_top(&self, handle: &WindowHandle) {
        if let WindowHandle::XlibHandle(window) = handle {
            unsafe {
                (self.xlib.XRaiseWindow)(self.display, *window);
            }
        }
    }

    /// Kills a window.
    // `XGrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XGrabServer.html
    // `XSetCloseDownMode`: https://tronche.com/gui/x/xlib/display/XSetCloseDownMode.html
    // `XKillClient`: https://tronche.com/gui/x/xlib/window-and-session-manager/XKillClient.html
    // `XSync`: https://tronche.com/gui/x/xlib/event-handling/XSync.html
    // `XUngrabServer`: https://tronche.com/gui/x/xlib/window-and-session-manager/XUngrabServer.html
    pub fn kill_window(&self, h: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            //nicely ask the window to close
            if !self.send_xevent_atom(*handle, self.atoms.WMDelete) {
                //force kill the app
                unsafe {
                    (self.xlib.XGrabServer)(self.display);
                    (self.xlib.XSetCloseDownMode)(self.display, xlib::DestroyAll);
                    (self.xlib.XKillClient)(self.display, *handle);
                    (self.xlib.XSync)(self.display, xlib::False);
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
        unsafe {
            (self.xlib.XSelectInput)(self.display, window, mask);
        }
    }

    /// Subscribe to the wanted events of a window.
    pub fn subscribe_to_window_events(&self, handle: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = handle {
            let mask = xlib::EnterWindowMask
                | xlib::FocusChangeMask
                | xlib::PropertyChangeMask
                | xlib::StructureNotifyMask;
            self.subscribe_to_event(*handle, mask);
        }
    }
}
