use std::{backtrace::Backtrace, ffi::CStr};

use leftwm_core::models::{
    BBox, DockArea, Screen, WindowHandle, WindowState, WindowType, XyhwChange,
};
use x11rb::{
    connection::Connection,
    properties::{WmClass, WmHints, WmSizeHints},
    protocol::{randr, xinerama, xproto},
};

use crate::{
    error::{BackendError, ErrorKind, Result},
    xatom::WMStateWindowState,
    X11rbWindowHandle,
};

use super::{XWrap, MAX_PROPERTY_VALUE_LEN};

impl XWrap {
    // Public functions.

    /// Returns the child windows of all roots.
    /// # Errors
    ///
    /// Will error if root has no windows or there is an error
    /// obtaining the root windows. See `get_windows_for_root`.
    pub fn get_all_windows(&self) -> Result<Vec<xproto::Window>> {
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
    pub fn get_color(&self, color: &str) -> Result<u32> {
        let screen = &self.conn.setup().roots[self.display];
        let (red, green, blue) = parse_color_string(color)?;

        let reply =
            xproto::alloc_color(&self.conn, screen.default_colormap, red, green, blue)?.reply()?;
        Ok(reply.pixel)
    }

    /// Returns the current position of the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    pub fn get_cursor_point(&self) -> Result<(i32, i32)> {
        let roots = self.get_roots();
        for w in roots {
            let reply = xproto::query_pointer(&self.conn, w)?.reply();
            if let Ok(reply) = reply {
                return Ok((reply.win_x.into(), reply.win_y.into()));
            }
        }
        Err(BackendError {
            src: None,
            msg: "No root window",
            backtrace: Backtrace::capture(),
            kind: ErrorKind::RootWindowNotFound,
        })
    }

    /// Returns the current window under the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    pub fn get_cursor_window(&self) -> Result<WindowHandle<X11rbWindowHandle>> {
        let roots = self.get_roots();
        for w in roots {
            let reply = xproto::query_pointer(&self.conn, w)?.reply();
            if let Ok(reply) = reply {
                return Ok(WindowHandle(X11rbWindowHandle(reply.child)));
            }
        }
        Err(BackendError {
            src: None,
            msg: "No root window",
            backtrace: Backtrace::capture(),
            kind: ErrorKind::RootWindowNotFound,
        })
    }

    /// Returns the handle of the default root.
    #[must_use]
    pub const fn get_default_root_handle(&self) -> WindowHandle<X11rbWindowHandle> {
        WindowHandle(X11rbWindowHandle(self.root))
    }

    /// Returns the default root.
    #[must_use]
    pub const fn get_default_root(&self) -> xproto::Window {
        self.root
    }

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window as a `XyhwChange`.
    pub fn get_hint_sizing_as_xyhw(&self, window: xproto::Window) -> Result<Option<XyhwChange>> {
        let hints = self.get_hint_sizing(window)?;
        if let Some(size) = hints {
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

            return Ok(Some(xyhw));
        }
        Ok(None)
    }

    /// Returns the next `Xevent` of the xserver.
    pub fn poll_next_event(&self) -> Result<Option<x11rb::protocol::Event>> {
        Ok(self.conn.poll_for_event()?)
    }

    /// Returns all the screens of the display.
    // TODO: Check if this is working, because it's most likely not
    pub fn get_screens(&self) -> Result<Vec<Screen<X11rbWindowHandle>>> {
        if let Ok(screen_resources) = randr::get_screen_resources(&self.conn, self.root)?.reply() {
            return Ok(screen_resources
                .outputs
                .iter()
                .filter_map(|&output| {
                    randr::get_output_info(&self.conn, output, screen_resources.config_timestamp)
                        .ok()
                })
                .filter_map(|res| res.reply().ok())
                .filter_map(|output_info| {
                    //FIX: This always fails
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
                    let mut s = Screen {
                        bbox: BBox {
                            x: i32::from(crtc_info.x),
                            y: i32::from(crtc_info.y),
                            width: i32::from(crtc_info.width),
                            height: i32::from(crtc_info.height),
                        },
                        ..Default::default()
                    };
                    s.root = self.get_default_root_handle();
                    s.output = name.to_string();
                    s
                })
                .collect());
        }

        let is_active = xinerama::is_active(&self.conn)?.reply()?;

        if is_active.state == 0 {
            // NON-XINERAMA
            return Ok(self
                .get_roots()
                .map(|w| self.get_hint_sizing_as_xyhw(w))
                .collect::<Result<Vec<Option<XyhwChange>>>>()?
                .into_iter()
                .flatten()
                .map(|xyhw| Screen {
                    bbox: BBox {
                        x: xyhw.x.unwrap_or_default(),
                        y: xyhw.y.unwrap_or_default(),
                        width: xyhw.w.unwrap_or_default(),
                        height: xyhw.h.unwrap_or_default(),
                    },
                    ..Default::default()
                })
                .collect());
        }

        let root = self.get_default_root_handle();
        let screens = xinerama::query_screens(&self.conn)?.reply()?;
        Ok(screens
            .screen_info
            .iter()
            .map(|screen_info| {
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
            .collect())
    }

    /// Returns the dimensions of the screens.
    pub fn get_screens_area_dimensions(&self) -> Result<(i32, i32)> {
        let mut height = 0;
        let mut width = 0;
        for s in self.get_screens()? {
            height = std::cmp::max(height, s.bbox.height + s.bbox.y);
            width = std::cmp::max(width, s.bbox.width + s.bbox.x);
        }
        Ok((height, width))
    }

    /// Returns the transient parent of a window.
    pub fn get_transient_for(&self, window: xproto::Window) -> Result<Option<xproto::Window>> {
        match xproto::get_property(
            &self.conn,
            false,
            window,
            xproto::AtomEnum::WM_TRANSIENT_FOR,
            xproto::AtomEnum::WINDOW,
            0,
            1,
        )?
        .reply()?
        .value32()
        {
            Some(mut i) => Ok(i.next()),
            None => Ok(None),
        }
    }

    /// Returns the atom actions of a window.
    pub fn get_window_actions_atoms(&self, window: xproto::Window) -> Result<Vec<xproto::Atom>> {
        let reply = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMAction,
            xproto::AtomEnum::ATOM,
            0,
            0,
        )?
        .reply()?;

        Ok(reply.value32().map_or(Vec::new(), Iterator::collect))
    }

    /// Returns the attributes of a window.
    pub fn get_window_attrs(
        &self,
        window: xproto::Window,
    ) -> Result<xproto::GetWindowAttributesReply> {
        Ok(xproto::get_window_attributes(&self.conn, window)?.reply()?)
    }

    /// Returns a windows class `WM_CLASS`
    pub fn get_window_class(&self, window: xproto::Window) -> Result<Option<WmClass>> {
        Ok(WmClass::get(&self.conn, window)?.reply()?)
    }

    /// Returns the geometry of a window as a `XyhwChange` struct.
    pub fn get_window_geometry(&self, window: xproto::Window) -> Result<XyhwChange> {
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
    pub fn get_window_name(&self, window: xproto::Window) -> Result<String> {
        if let Ok(text) = self.get_text_prop(window, self.atoms.NetWMName) {
            return Ok(text);
        }
        // fallback to legacy name
        self.get_window_legacy_name(window)
    }

    /// Returns a `WM_NAME` (not `_NET`windows name).
    pub fn get_window_legacy_name(&self, window: xproto::Window) -> Result<String> {
        self.get_text_prop(window, xproto::AtomEnum::WM_NAME.into())
    }

    /// Returns a windows `_NET_WM_PID`.
    pub fn get_window_pid(&self, window: xproto::Window) -> Result<u32> {
        let prop = self.get_property(
            window,
            self.atoms.NetWMPid,
            xproto::AtomEnum::CARDINAL.into(),
        )?;
        if prop.is_empty() {
            return Ok(x11rb::NONE);
        }
        Ok(prop[0])
    }

    /// Returns the states of a window.
    pub fn get_window_states(&self, window: xproto::Window) -> Result<Vec<WindowState>> {
        let window_states_atoms = self.get_window_states_atoms(window)?;

        // if window is maximized both horizontally and vertically
        // `WindowState::Maximized` is used
        // instead of `WindowState::MaximizedVert` and `WindowState::MaximizedHorz`
        let maximized = window_states_atoms.contains(&self.atoms.NetWMStateMaximizedVert)
            && window_states_atoms.contains(&self.atoms.NetWMStateMaximizedHorz);

        let mut window_states: Vec<WindowState> = window_states_atoms
            .iter()
            .map(|a| match a {
                x if x == &self.atoms.NetWMStateModal => WindowState::Modal,
                x if x == &self.atoms.NetWMStateSticky => WindowState::Sticky,
                x if x == &self.atoms.NetWMStateMaximizedVert && !maximized => {
                    WindowState::MaximizedVert
                }
                x if x == &self.atoms.NetWMStateMaximizedHorz && !maximized => {
                    WindowState::MaximizedHorz
                }
                x if x == &self.atoms.NetWMStateShaded => WindowState::Shaded,
                x if x == &self.atoms.NetWMStateSkipTaskbar => WindowState::SkipTaskbar,
                x if x == &self.atoms.NetWMStateSkipPager => WindowState::SkipPager,
                x if x == &self.atoms.NetWMStateHidden => WindowState::Hidden,
                x if x == &self.atoms.NetWMStateFullscreen => WindowState::Fullscreen,
                x if x == &self.atoms.NetWMStateAbove => WindowState::Above,
                x if x == &self.atoms.NetWMStateBelow => WindowState::Below,
                _ => WindowState::Modal,
            })
            .collect();

        if maximized {
            window_states.push(WindowState::Maximized);
        }

        Ok(window_states)
    }

    /// Returns the atom states of a window.
    pub fn get_window_states_atoms(&self, window: xproto::Window) -> Result<Vec<xproto::Atom>> {
        let reply = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMState,
            xproto::AtomEnum::ATOM,
            0,
            MAX_PROPERTY_VALUE_LEN / 4,
        )?
        .reply()?;

        Ok(reply.value32().map_or(Vec::new(), Iterator::collect))
    }

    /// Returns structure of a window as a `DockArea`.
    pub fn get_window_strut_array(&self, window: xproto::Window) -> Result<Option<DockArea>> {
        // More modern structure.
        if let Some(d) = self.get_window_strut_array_strut_partial(window)? {
            tracing::debug!("STRUT:[{:?}] {:?}", window, d);
            return Ok(Some(d));
        }
        // Older structure.
        if let Some(d) = self.get_window_strut_array_strut(window)? {
            tracing::debug!("STRUT:[{:?}] {:?}", window, d);
            return Ok(Some(d));
        }
        Ok(None)
    }

    /// Returns the type of a window.
    pub fn get_window_type(&self, window: xproto::Window) -> Result<WindowType> {
        let reply = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMWindowType,
            xproto::AtomEnum::ATOM,
            0,
            1,
        )?
        .reply()?;

        let Some(mut val) = reply.value32() else {
            return Ok(WindowType::Normal);
        };

        Ok(match val.next() {
            x if x == Some(self.atoms.NetWMWindowTypeDesktop) => WindowType::Desktop,
            x if x == Some(self.atoms.NetWMWindowTypeDock) => WindowType::Dock,
            x if x == Some(self.atoms.NetWMWindowTypeToolbar) => WindowType::Toolbar,
            x if x == Some(self.atoms.NetWMWindowTypeMenu) => WindowType::Menu,
            x if x == Some(self.atoms.NetWMWindowTypeUtility) => WindowType::Utility,
            x if x == Some(self.atoms.NetWMWindowTypeSplash) => WindowType::Splash,
            x if x == Some(self.atoms.NetWMWindowTypeDialog) => WindowType::Dialog,
            _ => WindowType::Normal,
        })
    }

    /// Returns the `WM_HINTS` of a window.
    pub fn get_wmhints(&self, window: xproto::Window) -> Result<Option<WmHints>> {
        Ok(WmHints::get(&self.conn, window)?.reply()?)
    }

    /// Returns the `WM_STATE` of a window.
    pub fn get_wm_state(
        &self,
        window: xproto::Window,
    ) -> Result<(WMStateWindowState, Option<xproto::Window>)> {
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
        )?
        .reply()?;

        let Some(values) = rep.value32().map(Iterator::collect::<Vec<u32>>) else {
            return Ok((WMStateWindowState::Normal, None));
        };
        Ok((
            values
                .first()
                .and_then(|v| v.try_into().ok())
                .unwrap_or(WMStateWindowState::Normal),
            values.get(1).copied(),
        ))
    }

    /// Returns the name of a `XAtom`.
    pub fn get_xatom_name(&self, atom: xproto::Atom) -> Result<String> {
        let name = xproto::get_atom_name(&self.conn, atom)?.reply()?.name;
        Ok(String::from_utf8(name)?)
    }

    // Internal functions.

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window.
    pub fn get_hint_sizing(&self, window: xproto::Window) -> Result<Option<WmSizeHints>> {
        Ok(WmSizeHints::get(&self.conn, window, self.atoms.WMNormalHints)?.reply()?)
    }

    /// Returns a cardinal property of a window.
    fn get_property(
        &self,
        window: xproto::Window,
        property: xproto::Atom,
        r#type: xproto::Atom,
    ) -> Result<Vec<xproto::Atom>> {
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
    fn get_text_prop(&self, window: xproto::Window, atom: xproto::Atom) -> Result<String> {
        let prop = xproto::get_property(
            &self.conn,
            false,
            window,
            atom,
            // Not sure for the type atom here...
            xproto::AtomEnum::ANY,
            0,
            MAX_PROPERTY_VALUE_LEN,
        )?
        .reply()?;
        let s = String::from_utf8(prop.value)?;
        Ok(s)
    }

    /// Returns the child windows of a root.
    fn get_windows_for_root(&self, root: xproto::Window) -> Result<Vec<xproto::Window>> {
        let oui = xproto::query_tree(&self.conn, root)?.reply()?;
        Ok(oui.children)
    }

    /// Returns the `_NET_WM_STRUT` as a `DockArea`.
    fn get_window_strut_array_strut(&self, window: xproto::Window) -> Result<Option<DockArea>> {
        let res = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMStrut,
            xproto::AtomEnum::CARDINAL,
            0,
            12,
        )?
        .reply()?;

        let values = res.value32();
        if let Some(v) = values {
            let values = v
                .map(i32::try_from)
                .map(|e| e.map_err(BackendError::from))
                .collect::<Result<Vec<i32>>>()?;
            Ok(Some(IntoDockArea(&values).into()))
        } else {
            Ok(None)
        }
    }

    /// Returns the `_NET_WM_STRUT_PARTIAL` as a `DockArea`.
    fn get_window_strut_array_strut_partial(
        &self,
        window: xproto::Window,
    ) -> Result<Option<DockArea>> {
        let res = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.NetWMStrutPartial,
            xproto::AtomEnum::CARDINAL,
            0,
            12,
        )?
        .reply()?;

        let values = res.value32();
        if let Some(v) = values {
            let values = v
                .map(i32::try_from)
                .map(|e| e.map_err(BackendError::from))
                .collect::<Result<Vec<i32>>>()?;
            Ok(Some(IntoDockArea(&values).into()))
        } else {
            Ok(None)
        }
    }
}

/// Parses a color string written in the hex format #RRGGBB to a tuple of u16.
/// Since colors in hex format are represented using 8 bits, we need to adjust them to represent
/// the right proportion of color on a 16 bits value by multiplying by 256
fn parse_color_string(color: &str) -> Result<(u16, u16, u16)> {
    Ok((
        u16::from_str_radix(&color[1..3], 16)? * 256,
        u16::from_str_radix(&color[3..5], 16)? * 256,
        u16::from_str_radix(&color[5..7], 16)? * 256,
    ))
}

struct IntoDockArea<'a>(&'a [i32]);

impl From<IntoDockArea<'_>> for DockArea {
    fn from(val: IntoDockArea<'_>) -> Self {
        DockArea {
            left: val.0[0],
            right: val.0[1],
            top: val.0[2],
            bottom: val.0[3],
            left_start_y: val.0[4],
            left_end_y: val.0[5],
            right_start_y: val.0[6],
            right_end_y: val.0[7],
            top_start_x: val.0[8],
            top_end_x: val.0[9],
            bottom_start_x: val.0[10],
            bottom_end_x: val.0[11],
        }
    }
}
