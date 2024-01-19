//! `XWrap` getters.
use super::{Screen, WindowHandle, XlibError, MAX_PROPERTY_VALUE_LEN, MOUSEMASK};
use crate::{XWrap, XlibWindowHandle};
use leftwm_core::models::{BBox, DockArea, WindowState, WindowType, XyhwChange};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};
use std::slice;
use x11_dl::xinerama::XineramaScreenInfo;
use x11_dl::xlib::{self, XWindowAttributes};
use x11_dl::xrandr::XRRCrtcInfo;

impl XWrap {
    // Public functions.

    /// Returns the child windows of all roots.
    /// # Errors
    ///
    /// Will error if root has no windows or there is an error
    /// obtaining the root windows. See `get_windows_for_root`.
    pub fn get_all_windows(&self) -> Result<Vec<xlib::Window>, String> {
        let mut all = Vec::new();
        for root in self.get_roots() {
            match self.get_windows_for_root(root) {
                Ok(some_windows) => {
                    for w in some_windows {
                        all.push(*w);
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Ok(all)
    }

    /// Returns a `XColor` for a color.
    // `XDefaultScreen`: https://tronche.com/gui/x/xlib/display/display-macros.html#DefaultScreen
    // `XDefaultColormap`: https://tronche.com/gui/x/xlib/display/display-macros.html#DefaultColormap
    // `XAllocNamedColor`: https://tronche.com/gui/x/xlib/color/XAllocNamedColor.html
    #[must_use]
    pub fn get_color(&self, color: String) -> c_ulong {
        unsafe {
            let screen = (self.xlib.XDefaultScreen)(self.display);
            let cmap: xlib::Colormap = (self.xlib.XDefaultColormap)(self.display, screen);
            let color_cstr = CString::new(color).unwrap_or_default().into_raw();
            let mut color: xlib::XColor = std::mem::zeroed();
            (self.xlib.XAllocNamedColor)(self.display, cmap, color_cstr, &mut color, &mut color);
            color.pixel
        }
    }

    /// Returns the current position of the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    // `XQueryPointer`: https://tronche.com/gui/x/xlib/window-information/XQueryPointer.html
    pub fn get_cursor_point(&self) -> Result<(i32, i32), XlibError> {
        let roots = self.get_roots();
        for w in roots {
            let mut root_return: xlib::Window = 0;
            let mut child_return: xlib::Window = 0;
            let mut root_x_return: c_int = 0;
            let mut root_y_return: c_int = 0;
            let mut win_x_return: c_int = 0;
            let mut win_y_return: c_int = 0;
            let mut mask_return: c_uint = 0;
            let success = unsafe {
                (self.xlib.XQueryPointer)(
                    self.display,
                    w,
                    &mut root_return,
                    &mut child_return,
                    &mut root_x_return,
                    &mut root_y_return,
                    &mut win_x_return,
                    &mut win_y_return,
                    &mut mask_return,
                )
            };
            if success > 0 {
                return Ok((win_x_return, win_y_return));
            }
        }
        Err(XlibError::RootWindowNotFound)
    }

    /// Returns the current window under the cursor.
    /// # Errors
    ///
    /// Will error if root window cannot be found.
    // `XQueryPointer`: https://tronche.com/gui/x/xlib/window-information/XQueryPointer.html
    pub fn get_cursor_window(&self) -> Result<WindowHandle<XlibWindowHandle>, XlibError> {
        let roots = self.get_roots();
        for w in roots {
            let mut root_return: xlib::Window = 0;
            let mut child_return: xlib::Window = 0;
            let mut root_x_return: c_int = 0;
            let mut root_y_return: c_int = 0;
            let mut win_x_return: c_int = 0;
            let mut win_y_return: c_int = 0;
            let mut mask_return: c_uint = 0;
            let success = unsafe {
                (self.xlib.XQueryPointer)(
                    self.display,
                    w,
                    &mut root_return,
                    &mut child_return,
                    &mut root_x_return,
                    &mut root_y_return,
                    &mut win_x_return,
                    &mut win_y_return,
                    &mut mask_return,
                )
            };
            if success > 0 {
                return Ok(WindowHandle(XlibWindowHandle(child_return)));
            }
        }
        Err(XlibError::RootWindowNotFound)
    }

    /// Returns the handle of the default root.
    #[must_use]
    pub const fn get_default_root_handle(&self) -> WindowHandle<XlibWindowHandle> {
        WindowHandle(XlibWindowHandle(self.root))
    }

    /// Returns the default root.
    #[must_use]
    pub const fn get_default_root(&self) -> xlib::Window {
        self.root
    }

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window as a `XyhwChange`.
    #[must_use]
    pub fn get_hint_sizing_as_xyhw(&self, window: xlib::Window) -> Option<XyhwChange> {
        let hint = self.get_hint_sizing(window);
        if let Some(size) = hint {
            let mut xyhw = XyhwChange::default();

            if (size.flags & xlib::PSize) != 0 || (size.flags & xlib::USSize) != 0 {
                // These are obsolete but are still used sometimes.
                xyhw.w = Some(size.width);
                xyhw.h = Some(size.height);
            } else if (size.flags & xlib::PBaseSize) != 0 {
                xyhw.w = Some(size.base_width);
                xyhw.h = Some(size.base_height);
            }

            if (size.flags & xlib::PResizeInc) != 0 {
                xyhw.w = Some(size.width_inc);
                xyhw.h = Some(size.height_inc);
            }

            if (size.flags & xlib::PMaxSize) != 0 {
                xyhw.maxw = Some(size.max_width);
                xyhw.maxh = Some(size.max_height);
            }

            if (size.flags & xlib::PMinSize) != 0 {
                xyhw.minw = Some(size.min_width);
                xyhw.minh = Some(size.min_height);
            }
            // Make sure that width and height are not smaller than the min values.
            xyhw.w = std::cmp::max(xyhw.w, xyhw.minw);
            xyhw.h = std::cmp::max(xyhw.h, xyhw.minh);
            // Ignore the sizing if the sizing is set to 0.
            xyhw.w = xyhw.w.filter(|&w| w != 0);
            xyhw.h = xyhw.h.filter(|&h| h != 0);

            if (size.flags & xlib::PPosition) != 0 || (size.flags & xlib::USPosition) != 0 {
                // These are obsolete but are still used sometimes.
                xyhw.x = Some(size.x);
                xyhw.y = Some(size.y);
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
    // `XMaskEvent`: https://tronche.com/gui/x/xlib/event-handling/manipulating-event-queue/XMaskEvent.html
    #[must_use]
    pub fn get_mask_event(&self) -> xlib::XEvent {
        unsafe {
            let mut event: xlib::XEvent = std::mem::zeroed();
            (self.xlib.XMaskEvent)(
                self.display,
                MOUSEMASK | xlib::SubstructureRedirectMask | xlib::ExposureMask,
                &mut event,
            );
            event
        }
    }

    /// Returns the next `Xevent` of the xserver.
    // `XNextEvent`: https://tronche.com/gui/x/xlib/event-handling/manipulating-event-queue/XNextEvent.html
    #[must_use]
    pub fn get_next_event(&self) -> xlib::XEvent {
        unsafe {
            let mut event: xlib::XEvent = std::mem::zeroed();
            (self.xlib.XNextEvent)(self.display, &mut event);
            event
        }
    }

    /// Returns all the screens of the display.
    /// # Panics
    ///
    /// Panics if xorg cannot be contacted (xlib missing, not started, etc.)
    /// Also panics if window attrs cannot be obtained.
    #[must_use]
    pub fn get_screens(&self) -> Vec<Screen<XlibWindowHandle>> {
        use x11_dl::xinerama::XineramaScreenInfo;
        use x11_dl::xinerama::Xlib;
        use x11_dl::xrandr::Xrandr;
        let xlib = Xlib::open().expect("Couldn't not connect to Xorg Server");

        // Use randr for screen detection if possible, otherwise fall back to Xinerama.
        // Only randr supports screen names.
        if let Ok(xrandr) = Xrandr::open() {
            unsafe {
                let screen_resources = (xrandr.XRRGetScreenResources)(self.display, self.root);
                let outputs = slice::from_raw_parts(
                    (*screen_resources).outputs,
                    (*screen_resources).noutput as usize,
                );

                return outputs
                    .iter()
                    .map(|output| {
                        (xrandr.XRRGetOutputInfo)(self.display, screen_resources, *output)
                    })
                    .filter(|&output_info| (*output_info).crtc != 0)
                    .map(|output_info| {
                        let crtc_info = (xrandr.XRRGetCrtcInfo)(
                            self.display,
                            screen_resources,
                            (*output_info).crtc,
                        );
                        let mut s: Screen<XlibWindowHandle> =
                            XRRCrtcInfoIntoScreen(*crtc_info).into();
                        s.root = self.get_default_root_handle();
                        s.output = CStr::from_ptr((*output_info).name)
                            .to_string_lossy()
                            .into_owned();
                        s
                    })
                    .collect();
            }
        }

        let xinerama = unsafe { (xlib.XineramaIsActive)(self.display) } > 0;
        if xinerama {
            let root = self.get_default_root_handle();
            let mut screen_count = 0;
            let info_array_raw =
                unsafe { (xlib.XineramaQueryScreens)(self.display, &mut screen_count) };
            // Take ownership of the array.
            let xinerama_infos: &[XineramaScreenInfo] =
                unsafe { slice::from_raw_parts(info_array_raw, screen_count as usize) };
            xinerama_infos
                .iter()
                .map(|i| {
                    let mut s: Screen<XlibWindowHandle> = XineramaScreenInfoIntoScreen(i).into();
                    s.root = root;
                    s
                })
                .collect()
        } else {
            // NON-XINERAMA
            let roots: Result<Vec<xlib::XWindowAttributes>, _> =
                self.get_roots().map(|w| self.get_window_attrs(w)).collect();
            let roots = roots.expect("Error: No screen were detected");
            roots
                .iter()
                .map(|attrs| XWindowAttributesIntoScreen(attrs).into())
                .collect()
        }
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
    // `XGetTransientForHint`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetTransientForHint.html
    #[must_use]
    pub fn get_transient_for(&self, window: xlib::Window) -> Option<xlib::Window> {
        unsafe {
            let mut transient: xlib::Window = std::mem::zeroed();
            let status: c_int =
                (self.xlib.XGetTransientForHint)(self.display, window, &mut transient);
            if status > 0 {
                Some(transient)
            } else {
                None
            }
        }
    }

    /// Returns the atom actions of a window.
    // `XGetWindowProperty`: https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    #[must_use]
    pub fn get_window_actions_atoms(&self, window: xlib::Window) -> Vec<xlib::Atom> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut bytes_remaining: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                self.atoms.NetWMAction,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                xlib::XA_ATOM,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_remaining,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) && !prop_return.is_null() {
                #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
                let ptr = prop_return as *const c_ulong;
                let results: &[xlib::Atom] = slice::from_raw_parts(ptr, nitems_return as usize);
                return results.to_vec();
            }
            vec![]
        }
    }

    /// Returns the attributes of a window.
    /// # Errors
    ///
    /// Will error if window status is 0 (no attributes).
    // `XGetWindowAttributes`: https://tronche.com/gui/x/xlib/window-information/XGetWindowAttributes.html
    pub fn get_window_attrs(
        &self,
        window: xlib::Window,
    ) -> Result<xlib::XWindowAttributes, XlibError> {
        let mut attrs: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        let status = unsafe { (self.xlib.XGetWindowAttributes)(self.display, window, &mut attrs) };
        if status == 0 {
            return Err(XlibError::FailedStatus);
        }
        Ok(attrs)
    }

    /// Returns a windows class `WM_CLASS`
    // `XGetClassHint`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetClassHint.html
    #[must_use]
    pub fn get_window_class(&self, window: xlib::Window) -> Option<(String, String)> {
        unsafe {
            let mut class_return: xlib::XClassHint = std::mem::zeroed();
            let status = (self.xlib.XGetClassHint)(self.display, window, &mut class_return);
            if status == 0 {
                return None;
            }
            let Ok(res_name) =
                CString::from_raw(class_return.res_name.cast::<c_char>()).into_string()
            else {
                return None;
            };
            let Ok(res_class) =
                CString::from_raw(class_return.res_class.cast::<c_char>()).into_string()
            else {
                return None;
            };
            Some((res_name, res_class))
        }
    }

    /// Returns the geometry of a window as a `XyhwChange` struct.
    /// # Errors
    ///
    /// Errors if Xlib returns a status of 0.
    // `XGetGeometry`: https://tronche.com/gui/x/xlib/window-information/XGetGeometry.html
    pub fn get_window_geometry(&self, window: xlib::Window) -> Result<XyhwChange, XlibError> {
        let mut root_return: xlib::Window = 0;
        let mut x_return: c_int = 0;
        let mut y_return: c_int = 0;
        let mut width_return: c_uint = 0;
        let mut height_return: c_uint = 0;
        let mut border_width_return: c_uint = 0;
        let mut depth_return: c_uint = 0;
        unsafe {
            let status = (self.xlib.XGetGeometry)(
                self.display,
                window,
                &mut root_return,
                &mut x_return,
                &mut y_return,
                &mut width_return,
                &mut height_return,
                &mut border_width_return,
                &mut depth_return,
            );
            if status == 0 {
                return Err(XlibError::FailedStatus);
            }
        }
        Ok(XyhwChange {
            x: Some(x_return),
            y: Some(y_return),
            w: Some(width_return as i32),
            h: Some(height_return as i32),
            ..XyhwChange::default()
        })
    }

    /// Returns a windows name.
    #[must_use]
    pub fn get_window_name(&self, window: xlib::Window) -> Option<String> {
        if let Ok(text) = self.get_text_prop(window, self.atoms.NetWMName) {
            return Some(text);
        }
        if let Ok(text) = self.get_text_prop(window, xlib::XA_WM_NAME) {
            return Some(text);
        }
        None
    }

    /// Returns a `WM_NAME` (not `_NET`windows name).
    #[must_use]
    pub fn get_window_legacy_name(&self, window: xlib::Window) -> Option<String> {
        if let Ok(text) = self.get_text_prop(window, xlib::XA_WM_NAME) {
            return Some(text);
        }
        None
    }

    /// Returns a windows `_NET_WM_PID`.
    #[must_use]
    pub fn get_window_pid(&self, window: xlib::Window) -> Option<u32> {
        let (prop_return, _) = self
            .get_property(window, self.atoms.NetWMPid, xlib::XA_CARDINAL)
            .ok()?;
        unsafe {
            #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
            let pid = *prop_return.cast::<u32>();
            Some(pid)
        }
    }

    /// Returns the states of a window.
    #[must_use]
    pub fn get_window_states(&self, window: xlib::Window) -> Vec<WindowState> {
        let window_states_atoms = self.get_window_states_atoms(window);

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

        window_states
    }

    /// Returns the atom states of a window.
    // `XGetWindowProperty`: https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    #[must_use]
    pub fn get_window_states_atoms(&self, window: xlib::Window) -> Vec<xlib::Atom> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut bytes_remaining: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                self.atoms.NetWMState,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                xlib::XA_ATOM,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_remaining,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) && !prop_return.is_null() {
                #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
                let ptr = prop_return as *const c_ulong;
                let results: &[xlib::Atom] = slice::from_raw_parts(ptr, nitems_return as usize);
                return results.to_vec();
            }
            vec![]
        }
    }

    /// Returns structure of a window as a `DockArea`.
    #[must_use]
    pub fn get_window_strut_array(&self, window: xlib::Window) -> Option<DockArea> {
        // More modern structure.
        if let Some(d) = self.get_window_strut_array_strut_partial(window) {
            tracing::trace!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        // Older structure.
        if let Some(d) = self.get_window_strut_array_strut(window) {
            tracing::trace!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        None
    }

    /// Returns the type of a window.
    #[must_use]
    pub fn get_window_type(&self, window: xlib::Window) -> WindowType {
        let mut atom = None;
        if let Ok((prop_return, _)) =
            self.get_property(window, self.atoms.NetWMWindowType, xlib::XA_ATOM)
        {
            #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
            let atom_ = unsafe { *prop_return.cast::<xlib::Atom>() };
            atom = Some(atom_);
        }
        match atom {
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
    pub fn get_wmhints(&self, window: xlib::Window) -> Option<xlib::XWMHints> {
        unsafe {
            let hints_ptr: *const xlib::XWMHints = (self.xlib.XGetWMHints)(self.display, window);
            if hints_ptr.is_null() {
                return None;
            }
            let hints: xlib::XWMHints = *hints_ptr;
            Some(hints)
        }
    }

    /// Returns the `WM_STATE` of a window.
    #[must_use]
    pub fn get_wm_state(&self, window: xlib::Window) -> Option<c_long> {
        let (prop_return, nitems_return) = self
            .get_property(window, self.atoms.WMState, self.atoms.WMState)
            .ok()?;
        if nitems_return == 0 {
            return None;
        }
        #[allow(clippy::cast_ptr_alignment)]
        Some(unsafe { *prop_return.cast::<c_long>() })
    }

    /// Returns the name of a `XAtom`.
    /// # Errors
    ///
    /// Errors if `XAtom` is not valid.
    // `XGetAtomName`: https://tronche.com/gui/x/xlib/window-information/XGetAtomName.html
    pub fn get_xatom_name(&self, atom: xlib::Atom) -> Result<String, XlibError> {
        unsafe {
            let cstring = (self.xlib.XGetAtomName)(self.display, atom);
            if let Ok(s) = CString::from_raw(cstring).into_string() {
                return Ok(s);
            }
        };
        Err(XlibError::InvalidXAtom)
    }

    // Internal functions.

    /// Returns the `WM_SIZE_HINTS`/`WM_NORMAL_HINTS` of a window.
    // `XGetWMNormalHints`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetWMNormalHints.html
    #[must_use]
    fn get_hint_sizing(&self, window: xlib::Window) -> Option<xlib::XSizeHints> {
        let mut xsize: xlib::XSizeHints = unsafe { std::mem::zeroed() };
        let mut msize: c_long = xlib::PSize;
        let status =
            unsafe { (self.xlib.XGetWMNormalHints)(self.display, window, &mut xsize, &mut msize) };
        match status {
            0 => None,
            _ => Some(xsize),
        }
    }

    /// Returns a cardinal property of a window.
    /// # Errors
    ///
    /// Errors if window status = 0.
    // `XGetWindowProperty`: https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    fn get_property(
        &self,
        window: xlib::Window,
        property: xlib::Atom,
        r#type: xlib::Atom,
    ) -> Result<(*const c_uchar, c_ulong), XlibError> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut bytes_after_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                property,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                r#type,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) && !prop_return.is_null() {
                return Ok((prop_return, nitems_return));
            }
        };
        Err(XlibError::FailedStatus)
    }

    /// Returns all the roots of the display.
    // `XRootWindowOfScreen`: https://tronche.com/gui/x/xlib/display/screen-information.html#RootWindowOfScreen
    fn get_roots(&self) -> impl Iterator<Item = xlib::Window> + '_ {
        self.get_xscreens()
            .map(|mut s| unsafe { (self.xlib.XRootWindowOfScreen)(&mut s) })
    }

    /// Returns a text property for a window.
    /// # Errors
    ///
    /// Errors if window status = 0.
    // `XGetTextProperty`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetTextProperty.html
    // `XTextPropertyToStringList`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XTextPropertyToStringList.html
    // `XmbTextPropertyToTextList`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XmbTextPropertyToTextList.html
    fn get_text_prop(&self, window: xlib::Window, atom: xlib::Atom) -> Result<String, XlibError> {
        unsafe {
            let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
            let status: c_int =
                (self.xlib.XGetTextProperty)(self.display, window, &mut text_prop, atom);
            if status == 0 {
                return Err(XlibError::FailedStatus);
            }
            if let Ok(s) = CString::from_raw(text_prop.value.cast::<c_char>()).into_string() {
                return Ok(s);
            }
        };
        Err(XlibError::FailedStatus)
    }

    /// Returns the child windows of a root.
    /// # Errors
    ///
    /// Will error if unknown window status is returned.
    // `XQueryTree`: https://tronche.com/gui/x/xlib/window-information/XQueryTree.html
    fn get_windows_for_root<'w>(&self, root: xlib::Window) -> Result<&'w [xlib::Window], String> {
        unsafe {
            let mut root_return: xlib::Window = std::mem::zeroed();
            let mut parent_return: xlib::Window = std::mem::zeroed();
            let mut array: *mut xlib::Window = std::mem::zeroed();
            let mut length: c_uint = std::mem::zeroed();
            let status: xlib::Status = (self.xlib.XQueryTree)(
                self.display,
                root,
                &mut root_return,
                &mut parent_return,
                &mut array,
                &mut length,
            );
            let windows: &[xlib::Window] = slice::from_raw_parts(array, length as usize);
            match status {
                0 /* XcmsFailure */ => { Err("Could not load list of windows".to_string() ) }
                1 /* XcmsSuccess */ | 2 /* XcmsSuccessWithCompression */ => { Ok(windows) }
                _ => { Err("Unknown return status".to_string() ) }
            }
        }
    }

    /// Returns the `_NET_WM_STRUT` as a `DockArea`.
    fn get_window_strut_array_strut(&self, window: xlib::Window) -> Option<DockArea> {
        let (prop_return, nitems_return) = self
            .get_property(window, self.atoms.NetWMStrut, xlib::XA_CARDINAL)
            .ok()?;
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            let array_ptr = prop_return.cast::<c_long>();
            let slice = slice::from_raw_parts(array_ptr, nitems_return as usize);
            if slice.len() == 12 {
                return Some(SliceIntoDockArea(slice).into());
            }
            None
        }
    }

    /// Returns the `_NET_WM_STRUT_PARTIAL` as a `DockArea`.
    fn get_window_strut_array_strut_partial(&self, window: xlib::Window) -> Option<DockArea> {
        let (prop_return, nitems_return) = self
            .get_property(window, self.atoms.NetWMStrutPartial, xlib::XA_CARDINAL)
            .ok()?;
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            let array_ptr = prop_return.cast::<c_long>();
            let slice = slice::from_raw_parts(array_ptr, nitems_return as usize);
            if slice.len() == 12 {
                return Some(SliceIntoDockArea(slice).into());
            }
            None
        }
    }

    /// Returns all the xscreens of the display.
    // `XScreenCount`: https://tronche.com/gui/x/xlib/display/display-macros.html#ScreenCount
    // `XScreenOfDisplay`: https://tronche.com/gui/x/xlib/display/display-macros.html#ScreensOfDisplay
    fn get_xscreens(&self) -> impl Iterator<Item = xlib::Screen> + '_ {
        let screen_count = unsafe { (self.xlib.XScreenCount)(self.display) };

        let screen_ids = 0..screen_count;

        screen_ids
            .map(|screen_id| unsafe { *(self.xlib.XScreenOfDisplay)(self.display, screen_id) })
    }
}

struct XRRCrtcInfoIntoScreen(XRRCrtcInfo);

impl Into<Screen<XlibWindowHandle>> for XRRCrtcInfoIntoScreen {
    fn into(self) -> Screen<XlibWindowHandle> {
        Screen {
            bbox: BBox {
                x: self.0.x,
                y: self.0.y,
                width: self.0.width as i32,
                height: self.0.height as i32,
            },
            ..Default::default()
        }
    }
}

struct XineramaScreenInfoIntoScreen<'a>(&'a XineramaScreenInfo);

impl Into<Screen<XlibWindowHandle>> for XineramaScreenInfoIntoScreen<'_> {
    fn into(self) -> Screen<XlibWindowHandle> {
        Screen {
            bbox: BBox {
                height: self.0.height.into(),
                width: self.0.width.into(),
                x: self.0.x_org.into(),
                y: self.0.y_org.into(),
            },
            ..Default::default()
        }
    }
}

struct XWindowAttributesIntoScreen<'a>(&'a XWindowAttributes);

impl Into<Screen<XlibWindowHandle>> for XWindowAttributesIntoScreen<'_> {
    fn into(self) -> Screen<XlibWindowHandle> {
        Screen {
            root: WindowHandle(XlibWindowHandle(self.0.root)),
            bbox: BBox {
                height: self.0.height,
                width: self.0.width,
                x: self.0.x,
                y: self.0.y,
            },
            ..Default::default()
        }
    }
}

struct SliceIntoDockArea<'a>(&'a [i64]);

impl Into<DockArea> for SliceIntoDockArea<'_> {
    fn into(self) -> DockArea {
        DockArea {
            left: self.0[0] as i32,
            right: self.0[1] as i32,
            top: self.0[2] as i32,
            bottom: self.0[3] as i32,
            left_start_y: self.0[4] as i32,
            left_end_y: self.0[5] as i32,
            right_start_y: self.0[6] as i32,
            right_end_y: self.0[7] as i32,
            top_start_x: self.0[8] as i32,
            top_end_x: self.0[9] as i32,
            bottom_start_x: self.0[10] as i32,
            bottom_end_x: self.0[11] as i32,
        }
    }
}
