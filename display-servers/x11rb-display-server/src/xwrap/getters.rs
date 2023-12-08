use std::ffi::{c_char, CStr, CString};

use leftwm_core::models::{
    BBox, DockArea, Screen, WindowHandle, WindowState, WindowType, XyhwChange,
};
use x11rb::{
    connection::Connection,
    properties::{WmClass, WmHints, WmSizeHints},
    protocol::{
        randr, xinerama,
        xproto::{self, ConnectionExt},
    },
};

use crate::xatom::WMStateWindowState;

use super::{XWrap, MAX_PROPERTY_VALUE_LEN};

impl XWrap {
    // Public functions.

    /// Returns the child windows of all roots.
    /// # Errors
    ///
    /// Will error if root has no windows or there is an error
    /// obtaining the root windows. See `get_windows_for_root`.
    pub fn get_all_windows(&self) -> Result<Vec<xproto::Window>, Box<dyn std::error::Error>> {
        let mut all = Vec::new();
        for root in self.get_roots() {
            let some_windows = self.get_windows_for_root(root)?;
            for w in some_windows {
                all.push(w);
            }
        }
        Ok(all)
    }

    /// Returns a `XColor` for a color.
    pub fn get_color(&self, color: String) -> u32 {
        let screen = &self.conn.setup().roots[self.display];
        let (red, green, blue) = parse_color_string(color);
        let rep = self
            .conn
            .alloc_color(screen.default_colormap, red, green, blue)
            .unwrap()
            .reply()
            .unwrap();
        rep.pixel
    }

    /// Returns the current position of the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    pub fn get_cursor_point(&self) -> Result<(i32, i32), Box<dyn std::error::Error>> {
        let roots = self.get_roots();
        for w in roots {
            let reply = xproto::query_pointer(&self.conn, w)?.reply();
            if let Ok(reply) = reply {
                return Ok((reply.win_x.into(), reply.win_y.into()));
            }
        }
        Err(Box::from("Root window not found"))
    }

    /// Returns the current window under the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    pub fn get_cursor_window(&self) -> Result<WindowHandle, Box<dyn std::error::Error>> {
        let roots = self.get_roots();
        for w in roots {
            let reply = xproto::query_pointer(&self.conn, w)?.reply();
            if let Ok(reply) = reply {
                return Ok(WindowHandle::X11rbHandle(reply.child));
            }
        }
        Err(Box::from("Root window not found"))
    }

    /// Returns the handle of the default root.
    #[must_use]
    pub const fn get_default_root_handle(&self) -> WindowHandle {
        WindowHandle::X11rbHandle(self.root)
    }

    /// Returns the default root.
    #[must_use]
    pub const fn get_default_root(&self) -> xproto::Window {
        self.root
    }

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window as a `XyhwChange`.
    #[must_use]
    pub fn get_hint_sizing_as_xyhw(&self, window: xproto::Window) -> Option<XyhwChange> {
        let hint = self.get_hint_sizing(window);
        if let Some(size) = hint {
            let mut xyhw = XyhwChange::default();

            if let Some((_specification, width, height)) = size.size {
                xyhw.w = Some(width);
                xyhw.h = Some(height);
            } else if let Some((width, height)) = size.base_size {
                xyhw.w = Some(width);
                xyhw.h = Some(height);
            }

            if let Some((width, height)) = size.size_increment {
                xyhw.w = Some(width);
                xyhw.h = Some(height);
            }

            if let Some((max_width, max_height)) = size.max_size {
                xyhw.maxw = Some(max_width);
                xyhw.maxh = Some(max_height);
            }

            if let Some((min_width, min_height)) = size.min_size {
                xyhw.minw = Some(min_width);
                xyhw.minh = Some(min_height);
            }
            // Make sure that width and height are not smaller than the min values.
            xyhw.w = std::cmp::max(xyhw.w, xyhw.minw);
            xyhw.h = std::cmp::max(xyhw.h, xyhw.minh);
            // Ignore the sizing if the sizing is set to 0.
            xyhw.w = xyhw.w.filter(|&w| w != 0);
            xyhw.h = xyhw.h.filter(|&h| h != 0);

            if let Some((_specification, x, y)) = size.position {
                xyhw.x = Some(x);
                xyhw.y = Some(y);
            }
            // TODO: support min/max aspect
            // if size.flags & xlib::PAspect != 0 {
            //     //c->mina = (float)size.min_aspect.y / size.min_aspect.x;
            //     //c->maxa = (float)size.max_aspect.x / size.max_aspect.y;
            // }

            return Some(xyhw);
        }
        None
    }

    /// Returns the next `Xevent` that matches the mask of the xserver.
    // pub fn get_mask_event(&self) -> xlib::XEvent {
    //     unsafe {
    //         let mut event: xlib::XEvent = std::mem::zeroed();
    //         (self.xlib.XMaskEvent)(
    //             self.display,
    //             MOUSEMASK | xlib::SubstructureRedirectMask | xlib::ExposureMask,
    //             &mut event,
    //         );
    //         event
    //     }
    // }

    /// Returns the next `Xevent` of the xserver.
    #[must_use]
    pub fn poll_next_event(&self) -> Option<x11rb::protocol::Event> {
        self.conn.poll_for_event().unwrap()
    }

    /// Returns all the screens of the display.
    /// # Panics
    ///
    /// Panics if xorg cannot be contacted (xlib missing, not started, etc.)
    /// Also panics if window attrs cannot be obtained.
    #[must_use]
    pub fn get_screens(&self) -> Vec<Screen> {
        if let Ok(screen_resources) = randr::get_screen_resources(&self.conn, self.root)
            .unwrap()
            .reply()
        {
            return screen_resources
                .outputs
                .iter()
                .filter_map(|&output| {
                    randr::get_output_info(&self.conn, output, screen_resources.config_timestamp)
                        .ok()
                })
                .filter_map(|res| res.reply().ok())
                .filter_map(|output_info| {
                    let name = match CStr::from_bytes_with_nul(&output_info.name) {
                        Ok(name) => name.to_str().unwrap(),
                        Err(_) => "output_name",
                    };
                    Some((
                        randr::get_crtc_info(
                            &self.conn,
                            output_info.crtc,
                            screen_resources.config_timestamp,
                        )
                        .ok()?,
                        name.to_string(),
                    ))
                })
                .filter_map(|(res, name)| Some((res.reply().ok()?, name)))
                .map(|(crtc_info, name)| {
                    // This do not work apparently...
                    // let mut s = Screen::from(crtc_info);
                    let mut s = Screen {
                        bbox: BBox {
                            x: crtc_info.x as i32,
                            y: crtc_info.y as i32,
                            width: crtc_info.width as i32,
                            height: crtc_info.height as i32,
                        },
                        ..Default::default()
                    };
                    s.root = self.get_default_root_handle();
                    s.output = name.to_string();
                    s
                })
                .collect();
        }

        let is_active = xinerama::is_active(&self.conn).unwrap().reply().unwrap();

        if is_active.state == 0 {
            // NON-XINERAMA
            // Idk if this works
            return self
                .get_roots()
                .filter_map(|w| self.get_hint_sizing_as_xyhw(w))
                .map(|xyhw| Screen {
                    bbox: BBox {
                        x: xyhw.x.unwrap_or_default(),
                        y: xyhw.y.unwrap_or_default(),
                        width: xyhw.w.unwrap_or_default(),
                        height: xyhw.h.unwrap_or_default(),
                    },
                    ..Default::default()
                })
                .collect();
        }

        let root = self.get_default_root_handle();
        let screens = xinerama::query_screens(&self.conn)
            .unwrap()
            .reply()
            .unwrap();
        screens
            .screen_info
            .iter()
            .map(|screen_info| {
                // This do not work apparently...
                // let mut s = Screen::from(screen_info);
                let mut s = Screen {
                    bbox: BBox {
                        height: screen_info.height.into(),
                        width: screen_info.width.into(),
                        x: screen_info.x_org.into(),
                        y: screen_info.y_org.into(),
                    },
                    ..Default::default()
                };
                s.root = root;
                s
            })
            .collect()
    }

    /// Returns the dimensions of the screens.
    #[must_use]
    pub fn get_screens_area_dimensions(&self) -> (i32, i32) {
        let mut height = 0;
        let mut width = 0;
        for s in self.get_screens() {
            height = std::cmp::max(height, s.bbox.height + s.bbox.y);
            width = std::cmp::max(width, s.bbox.width + s.bbox.x);
        }
        (height, width)
    }

    /// Returns the transient parent of a window.
    #[must_use]
    pub fn get_transient_for(&self, window: xproto::Window) -> Option<xproto::Window> {
        xproto::get_property(
            &self.conn,
            false,
            window,
            xproto::AtomEnum::WM_TRANSIENT_FOR,
            xproto::AtomEnum::WINDOW,
            0,
            1,
        )
        .ok()?
        .reply()
        .ok()?
        .value32()?
        .next()
    }

    /// Returns the atom actions of a window.
    #[must_use]
    pub fn get_window_actions_atoms(&self, window: xproto::Window) -> Vec<xproto::Atom> {
        let Ok(cookie) = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMAction,
            xproto::AtomEnum::ATOM,
            0,
            0,
        ) else {
            return vec![];
        };

        let Ok(reply) = cookie.reply() else {
            return vec![];
        };

        let rt = match reply.value32() {
            Some(values) => values.collect(),
            None => vec![],
        };
        rt
    }

    /// Returns the attributes of a window.
    /// # Errors
    ///
    /// Will error if window status is 0 (no attributes).
    // `XGetWindowAttributes`: https://tronche.com/gui/x/xlib/window-information/XGetWindowAttributes.html
    pub fn get_window_attrs(
        &self,
        window: xproto::Window,
    ) -> Result<xproto::GetWindowAttributesReply, Box<dyn std::error::Error>> {
        Ok(xproto::get_window_attributes(&self.conn, window)?.reply()?)
    }

    /// Returns a windows class `WM_CLASS`
    // `XGetClassHint`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetClassHint.html
    #[must_use]
    pub fn get_window_class(&self, window: xproto::Window) -> Option<WmClass> {
        WmClass::get(&self.conn, window)
            .unwrap()
            .reply()
            .ok()
            .flatten()
    }

    /// Returns the geometry of a window as a `XyhwChange` struct.
    /// # Errors
    ///
    /// Errors if Xlib returns a status of 0.
    // `XGetGeometry`: https://tronche.com/gui/x/xlib/window-information/XGetGeometry.html
    pub fn get_window_geometry(
        &self,
        window: xproto::Window,
    ) -> Result<XyhwChange, Box<dyn std::error::Error>> {
        let geo = xproto::get_geometry(&self.conn, window)?.reply()?;
        Ok(XyhwChange {
            x: Some(geo.x.into()),
            y: Some(geo.y.into()),
            h: Some(geo.height.into()),
            w: Some(geo.width.into()),
            ..Default::default()
        })
    }

    /// Returns a windows name.
    #[must_use]
    pub fn get_window_name(&self, window: xproto::Window) -> Option<String> {
        if let Ok(text) = self.get_text_prop(window, self.atoms.NetWMName) {
            return Some(text);
        }
        // fallback to legacy name
        self.get_window_legacy_name(window)
    }

    /// Returns a `WM_NAME` (not `_NET`windows name).
    #[must_use]
    pub fn get_window_legacy_name(&self, window: xproto::Window) -> Option<String> {
        if let Ok(text) = self.get_text_prop(window, xproto::AtomEnum::WM_NAME.into()) {
            return Some(text);
        }
        None
    }

    /// Returns a windows `_NET_WM_PID`.
    #[must_use]
    pub fn get_window_pid(&self, window: xproto::Window) -> Option<u32> {
        let prop = self
            .get_property(
                window,
                self.atoms.NetWMPid,
                xproto::AtomEnum::CARDINAL.into(),
            )
            .ok()?;
        Some(prop[0])
    }

    /// Returns the states of a window.
    #[must_use]
    pub fn get_window_states(&self, window: xproto::Window) -> Vec<WindowState> {
        self.get_window_states_atoms(window)
            .iter()
            .map(|a| match a {
                x if x == &self.atoms.NetWMStateModal => WindowState::Modal,
                x if x == &self.atoms.NetWMStateSticky => WindowState::Sticky,
                x if x == &self.atoms.NetWMStateMaximizedVert => WindowState::MaximizedVert,
                x if x == &self.atoms.NetWMStateMaximizedHorz => WindowState::MaximizedHorz,
                x if x == &self.atoms.NetWMStateShaded => WindowState::Shaded,
                x if x == &self.atoms.NetWMStateSkipTaskbar => WindowState::SkipTaskbar,
                x if x == &self.atoms.NetWMStateSkipPager => WindowState::SkipPager,
                x if x == &self.atoms.NetWMStateHidden => WindowState::Hidden,
                x if x == &self.atoms.NetWMStateFullscreen => WindowState::Fullscreen,
                x if x == &self.atoms.NetWMStateAbove => WindowState::Above,
                x if x == &self.atoms.NetWMStateBelow => WindowState::Below,
                _ => WindowState::Modal,
            })
            .collect()
    }

    /// Returns the atom states of a window.
    // `XGetWindowProperty`: https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    #[must_use]
    pub fn get_window_states_atoms(&self, window: xproto::Window) -> Vec<xproto::Atom> {
        let Ok(cookie) = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMState,
            xproto::AtomEnum::ATOM,
            0,
            MAX_PROPERTY_VALUE_LEN / 4,
        ) else {
            return vec![];
        };

        let Ok(rep) = cookie.reply() else {
            return vec![];
        };

        let rt = match rep.value32() {
            Some(atoms) => atoms.collect(),
            None => vec![],
        };
        rt
    }

    /// Returns structure of a window as a `DockArea`.
    #[must_use]
    pub fn get_window_strut_array(&self, window: xproto::Window) -> Option<DockArea> {
        // More modern structure.
        if let Some(d) = self.get_window_strut_array_strut_partial(window) {
            tracing::debug!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        // Older structure.
        if let Some(d) = self.get_window_strut_array_strut(window) {
            tracing::debug!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        None
    }

    /// Returns the type of a window.
    #[must_use]
    pub fn get_window_type(&self, window: xproto::Window) -> WindowType {
        let Ok(cookie) = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMWindowType,
            xproto::AtomEnum::ATOM,
            0,
            1,
        ) else {
            return WindowType::Normal;
        };

        let Ok(res) = cookie.reply() else {
            return WindowType::Normal;
        };

        let Some(mut val) = res.value32() else {
            return WindowType::Normal;
        };

        match val.next() {
            x if x == Some(self.atoms.NetWMWindowTypeDesktop) => WindowType::Desktop,
            x if x == Some(self.atoms.NetWMWindowTypeDock) => WindowType::Dock,
            x if x == Some(self.atoms.NetWMWindowTypeToolbar) => WindowType::Toolbar,
            x if x == Some(self.atoms.NetWMWindowTypeMenu) => WindowType::Menu,
            x if x == Some(self.atoms.NetWMWindowTypeUtility) => WindowType::Utility,
            x if x == Some(self.atoms.NetWMWindowTypeSplash) => WindowType::Splash,
            x if x == Some(self.atoms.NetWMWindowTypeDialog) => WindowType::Dialog,
            _ => WindowType::Normal,
        }
    }

    /// Returns the `WM_HINTS` of a window.
    // `XGetWMHints`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetWMHints.html
    #[must_use]
    pub fn get_wmhints(&self, window: xproto::Window) -> Option<WmHints> {
        WmHints::get(&self.conn, window)
            .unwrap()
            .reply()
            .ok()
            .flatten()
    }

    /// Returns the `WM_STATE` of a window.
    pub fn get_wm_state(
        &self,
        window: xproto::Window,
    ) -> Option<(WMStateWindowState, Option<xproto::Window>)> {
        // `WM_STATE` contains 2 properties:
        //   - state (CARD32)
        //   - icon (WINDOW)
        let rep = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.WMState,
            self.atoms.WMState,
            0,
            2,
        )
        .ok()?
        .reply()
        .ok()?;

        let values: Vec<u32> = rep.value32()?.collect();
        Some((values.get(0)?.try_into().ok()?, values.get(1).copied()))
    }

    /// Returns the name of a `XAtom`.
    /// # Errors
    ///
    /// Errors if `XAtom` is not valid.
    // `XGetAtomName`: https://tronche.com/gui/x/xlib/window-information/XGetAtomName.html
    pub fn get_xatom_name(&self, atom: xproto::Atom) -> Result<String, Box<dyn std::error::Error>> {
        let mut name = xproto::get_atom_name(&self.conn, atom)?.reply()?.name;
        Ok(unsafe { CString::from_raw(name.as_mut_ptr() as *mut c_char) }.into_string()?)
    }

    // Internal functions.

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window.
    #[must_use]
    pub fn get_hint_sizing(&self, window: xproto::Window) -> Option<WmSizeHints> {
        WmSizeHints::get(&self.conn, window, self.atoms.WMNormalHints)
            .unwrap()
            .reply()
            .ok()
            .flatten()
    }

    /// Returns a cardinal property of a window.
    /// # Errors
    ///
    /// Errors if window status = 0.
    // `XGetWindowProperty`: https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    fn get_property(
        &self,
        window: xproto::Window,
        property: xproto::Atom,
        r#type: xproto::Atom,
        // ) -> Result<(*const c_uchar, c_ulong), XlibError> {
    ) -> Result<Vec<xproto::Atom>, Box<dyn std::error::Error>> {
        let res =
            xproto::get_property(&self.conn, false, window, property, r#type, 0, 0)?.reply()?;

        let rt = match res.value32() {
            Some(props) => props.collect(),
            None => vec![],
        };
        Ok(rt)
    }

    /// Returns all the roots of the display.
    fn get_roots(&self) -> impl Iterator<Item = xproto::Window> + '_ {
        self.conn.setup().roots.iter().map(|screen| screen.root)
    }

    /// Returns a text property for a window.
    /// # Errors
    ///
    /// Errors if window status = 0.
    // `XGetTextProperty`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetTextProperty.html
    // `XTextPropertyToStringList`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XTextPropertyToStringList.html
    // `XmbTextPropertyToTextList`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XmbTextPropertyToTextList.html
    fn get_text_prop(
        &self,
        window: xproto::Window,
        atom: xproto::Atom,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Not sure for the type atom here...
        let prop = xproto::get_property(
            &self.conn,
            false,
            window,
            atom,
            xproto::AtomEnum::ANY,
            0,
            MAX_PROPERTY_VALUE_LEN,
        )?
        .reply()?;
        let s = CString::from_vec_with_nul(prop.value)?;
        Ok(s.into_string()?)
    }

    /// Returns the child windows of a root.
    /// # Errors
    ///
    /// Will error if unknown window status is returned.
    fn get_windows_for_root<'w>(
        &self,
        root: xproto::Window,
    ) -> Result<Vec<xproto::Window>, Box<dyn std::error::Error>> {
        let oui = xproto::query_tree(&self.conn, root)?.reply()?;
        Ok(oui.children)
    }

    /// Returns the `_NET_WM_STRUT` as a `DockArea`.
    fn get_window_strut_array_strut(&self, window: xproto::Window) -> Option<DockArea> {
        let res = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMStrut,
            xproto::AtomEnum::CARDINAL,
            0,
            12,
        )
        .ok()?
        .reply()
        .ok()?;

        let values: Vec<i32> = res.value32()?.map(|v| v as i32).collect();
        Some(DockArea::from(&values[..]))
    }

    /// Returns the `_NET_WM_STRUT_PARTIAL` as a `DockArea`.
    fn get_window_strut_array_strut_partial(&self, window: xproto::Window) -> Option<DockArea> {
        let res = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMStrutPartial,
            xproto::AtomEnum::CARDINAL,
            0,
            12,
        )
        .ok()?
        .reply()
        .ok()?;

        let values: Vec<i32> = res.value32()?.map(|v| v as i32).collect();
        Some(DockArea::from(&values[..]))
    }

    // /// Returns all the xscreens of the display.
    // // `XScreenCount`: https://tronche.com/gui/x/xlib/display/display-macros.html#ScreenCount
    // // `XScreenOfDisplay`: https://tronche.com/gui/x/xlib/display/display-macros.html#ScreensOfDisplay
    // fn get_xscreens(&self) -> impl Iterator<Item = xlib::Screen> + '_ {
    //     let screens = xinerama::query_screens(&self.conn).unwrap().reply().unwrap().screen_info;
    //     let screen_count = unsafe { (self.xlib.XScreenCount)(self.display) };
    //
    //     let screen_ids = 0..screen_count;
    //
    //     screen_ids
    //         .map(|screen_id| unsafe { *(self.xlib.XScreenOfDisplay)(self.display, screen_id) })
    // }
}

/// Parses a color string written in the hex format #RRGGBB
fn parse_color_string(color: String) -> (u16, u16, u16) {
    (
        u16::from_str_radix(&color[1..3], 16).unwrap(),
        u16::from_str_radix(&color[3..5], 16).unwrap(),
        u16::from_str_radix(&color[3..5], 16).unwrap(),
    )
}
