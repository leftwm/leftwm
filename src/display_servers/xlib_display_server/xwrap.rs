//! A wrapper around many WM features
//We allow this _ because if we don't we'll receive an error that it isn't read on _task_guard.
#![allow(clippy::used_underscore_binding)]
//We allow this so that extern "C" functions are not flagged as confusing. The current placement
//allows for easy reading.
#![allow(clippy::items_after_statements)]
//We allow this because _y_ and _x_ are intentionally similar. Changing it makes the code noisy.
#![allow(clippy::similar_names)]
use super::utils;
use super::xatom::XAtom;
use super::xcursor::XCursor;
use super::Config;
use super::Screen;
use super::Window;
use super::WindowHandle;
use crate::config::Keybind;
use crate::models::DockArea;
use crate::models::Mode;
use crate::models::WindowChange;
use crate::models::WindowState;
use crate::models::WindowType;
use crate::models::Xyhw;
use crate::models::XyhwChange;
use crate::utils::xkeysym_lookup::ModMask;
use crate::DisplayEvent;
use crate::{config::ThemeSetting, models::FocusBehaviour};
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};
use std::ptr;
use std::slice;
use std::sync::Arc;
use tokio::sync::{oneshot, Notify};
use tokio::time::Duration;
use x11_dl::xlib;

//type WindowStateConst = u8;
//const WITHDRAWN_STATE: WindowStateConst = 0;
//const NORMAL_STATE: WindowStateConst = 1;
//const ICONIC_STATE: WindowStateConst = 2;
const MAX_PROPERTY_VALUE_LEN: c_long = 4096;

const BUTTONMASK: c_long = xlib::ButtonPressMask | xlib::ButtonReleaseMask;
const MOUSEMASK: c_long = BUTTONMASK | xlib::PointerMotionMask;

pub struct Colors {
    normal: c_ulong,
    floating: c_ulong,
    active: c_ulong,
}

#[derive(Debug, Clone)]
pub enum XlibError {
    FailedStatus,
    RootWindowNotFound,
    InvalidXAtom,
}

/// Contains Xserver information and origins.
pub struct XWrap {
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
    root: xlib::Window,
    pub atoms: XAtom,
    cursors: XCursor,
    colors: Colors,
    managed_windows: Vec<xlib::Window>,
    pub tags: Vec<String>,
    pub mode: Mode,
    pub focus_behaviour: FocusBehaviour,
    pub mouse_key_mask: ModMask,
    pub mode_origin: (i32, i32),
    _task_guard: oneshot::Receiver<()>,
    task_notify: Arc<Notify>,
}

impl Default for XWrap {
    fn default() -> Self {
        Self::new()
    }
}

impl XWrap {
    /// # Panics
    ///
    /// Can panic if unable to contact xorg.
    #[must_use]
    pub fn new() -> Self {
        const SERVER: mio::Token = mio::Token(0);
        let xlib = xlib::Xlib::open().expect("Couldn't not connect to Xorg Server");
        let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
        assert!(!display.is_null(), "Null pointer in display");

        let fd = unsafe { (xlib.XConnectionNumber)(display) };

        let (guard, _task_guard) = oneshot::channel();
        let notify = Arc::new(Notify::new());
        let task_notify = notify.clone();

        let mut poll = mio::Poll::new().expect("Unable to boot Mio");
        let mut events = mio::Events::with_capacity(1);
        poll.registry()
            .register(
                &mut mio::unix::SourceFd(&fd),
                SERVER,
                mio::Interest::READABLE,
            )
            .expect("Unable to boot Mio");
        let timeout = Duration::from_millis(100);
        tokio::task::spawn_blocking(move || loop {
            if guard.is_closed() {
                return;
            }

            if let Err(err) = poll.poll(&mut events, Some(timeout)) {
                log::warn!("Xlib socket poll failed with {:?}", err);
                continue;
            }

            events
                .iter()
                .filter(|event| SERVER == event.token())
                .for_each(|_| notify.notify_one());
        });

        let atoms = XAtom::new(&xlib, display);
        let cursors = XCursor::new(&xlib, display);
        let root = unsafe { (xlib.XDefaultRootWindow)(display) };

        let colors = Colors {
            normal: 0,
            floating: 0,
            active: 0,
        };

        let xw = Self {
            xlib,
            display,
            root,
            atoms,
            cursors,
            colors,
            managed_windows: vec![],
            tags: vec![],
            mode: Mode::Normal,
            focus_behaviour: FocusBehaviour::Sloppy,
            mouse_key_mask: 0,
            mode_origin: (0, 0),
            _task_guard,
            task_notify,
        };

        //check that another WM is not running
        extern "C" fn startup_check_for_other_wm(
            _: *mut xlib::Display,
            _: *mut xlib::XErrorEvent,
        ) -> c_int {
            eprintln!("ERROR: another window manager is already running");
            ::std::process::exit(-1);
        }
        unsafe {
            (xw.xlib.XSetErrorHandler)(Some(startup_check_for_other_wm));
            (xw.xlib.XSelectInput)(xw.display, root, xlib::SubstructureRedirectMask);
            (xw.xlib.XSync)(xw.display, xlib::False);
        };

        // This is allowed for now as const extern fns
        // are not yet stable (1.56.0, 16 Sept 2021)
        // see issue #64926 <https://github.com/rust-lang/rust/issues/64926> for more information
        #[allow(clippy::missing_const_for_fn)]
        extern "C" fn on_error_from_xlib(
            _: *mut xlib::Display,
            er: *mut xlib::XErrorEvent,
        ) -> c_int {
            let err = unsafe { *er };
            //ignore bad window errors
            if err.error_code == xlib::BadWindow {
                return 0;
            }
            1
        }

        // setup cached keymap/modifier information, otherwise MappingNotify might never be called
        // from:
        // https://stackoverflow.com/questions/35569562/how-to-catch-keyboard-layout-change-event-and-get-current-new-keyboard-layout-on
        xw.keysym_to_keycode(x11_dl::keysym::XK_F1);

        unsafe {
            (xw.xlib.XSetErrorHandler)(Some(on_error_from_xlib));
            (xw.xlib.XSync)(xw.display, xlib::False);
        };
        xw
    }

    //returns all the screens the display
    /// # Panics
    ///
    /// Panics if xorg cannot be contacted (xlib missing, not started, etc.)
    /// Also panics if window attrs cannot be obtained.
    #[must_use]
    pub fn get_screens(&self) -> Vec<Screen> {
        use x11_dl::xinerama::XineramaScreenInfo;
        use x11_dl::xinerama::Xlib;
        let xlib = Xlib::open().expect("Couldn't not connect to Xorg Server");
        let xinerama = unsafe { (xlib.XineramaIsActive)(self.display) } > 0;
        if xinerama {
            let root = self.get_default_root_handle();
            let mut screen_count = 0;
            let info_array_raw =
                unsafe { (xlib.XineramaQueryScreens)(self.display, &mut screen_count) };
            //take ownership of the array
            let xinerama_infos: &[XineramaScreenInfo] =
                unsafe { slice::from_raw_parts(info_array_raw, screen_count as usize) };
            xinerama_infos
                .iter()
                .map(|i| {
                    let mut s = Screen::from(i);
                    s.root = root;
                    s
                })
                .collect()
        } else {
            //NON-XINERAMA
            let roots: Result<Vec<xlib::XWindowAttributes>, _> = self
                .get_roots()
                .iter()
                .map(|w| self.get_window_attrs(*w))
                .collect();
            let roots = roots.expect("Error: No screen were detected");
            roots.iter().map(Screen::from).collect()
        }
    }

    //returns all the screens the display
    #[must_use]
    pub fn get_xscreens(&self) -> Vec<xlib::Screen> {
        let mut screens = Vec::new();
        let screen_count = unsafe { (self.xlib.XScreenCount)(self.display) };
        for screen_num in 0..(screen_count) {
            let screen = unsafe { *(self.xlib.XScreenOfDisplay)(self.display, screen_num) };
            screens.push(screen);
        }
        screens
    }

    //returns all the screens the display
    #[must_use]
    pub const fn get_default_root_handle(&self) -> WindowHandle {
        WindowHandle::XlibHandle(self.get_default_root())
    }

    #[must_use]
    pub const fn get_default_root(&self) -> xlib::Window {
        self.root
    }

    //returns all the roots the display
    #[must_use]
    pub fn get_roots(&self) -> Vec<xlib::Window> {
        self.get_xscreens()
            .into_iter()
            .map(|mut s| unsafe { (self.xlib.XRootWindowOfScreen)(&mut s) })
            .collect()
    }

    #[must_use]
    pub fn keycode_to_keysym(&self, keycode: u32) -> utils::xkeysym_lookup::XKeysym {
        // Not using XKeysymToKeycode because deprecated
        let sym = unsafe { (self.xlib.XkbKeycodeToKeysym)(self.display, keycode as u8, 0, 0) };
        sym as u32
    }

    pub fn keysym_to_keycode(&self, keysym: utils::xkeysym_lookup::XKeysym) -> u32 {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, keysym.into()) };
        u32::from(code)
    }

    /// # Errors
    /// Will error if updating the keyboard failed
    pub fn refresh_keyboard(&self, evt: &mut xlib::XMappingEvent) -> Result<(), XlibError> {
        let status = unsafe { (self.xlib.XRefreshKeyboardMapping)(evt) };
        if status == 0 {
            Err(XlibError::FailedStatus)
        } else {
            Ok(())
        }
    }

    /// # Errors
    ///
    /// Will error if unknown window status is returned.
    //returns all the windows under a root windows
    pub fn get_windows_for_root<'w>(
        &self,
        root: xlib::Window,
    ) -> Result<&'w [xlib::Window], String> {
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

    /// # Errors
    ///
    /// Will error if window status is 0 (no attributes).
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

    #[must_use]
    pub fn get_atom_prop_value(
        &self,
        window: xlib::Window,
        prop: xlib::Atom,
    ) -> Option<xlib::Atom> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                prop,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                xlib::XA_ATOM,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut nitems_return,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) && !prop_return.is_null() {
                #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
                let atom = *(prop_return as *const xlib::Atom);
                return Some(atom);
            }
            None
        }
    }

    #[must_use]
    pub fn get_window_type(&self, window: xlib::Window) -> WindowType {
        match self.get_atom_prop_value(window, self.atoms.NetWMWindowType) {
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

    pub fn set_window_states_atoms(&self, window: xlib::Window, states: &[xlib::Atom]) {
        let data: Vec<u32> = states.iter().map(|x| *x as u32).collect();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.NetWMState,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
            std::mem::forget(data);
        }
    }

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
                //let result = *(prop_return as *const u32);
                let ptr = prop_return as *const c_ulong;
                let results: &[xlib::Atom] = slice::from_raw_parts(ptr, nitems_return as usize);
                return results.to_vec();
            }
            vec![]
        }
    }

    #[must_use]
    pub fn get_window_states(&self, window: xlib::Window) -> Vec<WindowState> {
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

    /// EWMH support used for bars such as polybar.
    ///  # Panics
    ///
    ///  Panics if a new Cstring cannot be formed
    pub fn init_desktops_hints(&self) {
        let tags = &self.tags;
        let tag_length = tags.len();
        //set the number of desktop
        let data = vec![tag_length as u32];
        self.set_desktop_prop(&data, self.atoms.NetNumberOfDesktops);
        //set a current desktop
        let data = vec![0_u32, xlib::CurrentTime as u32];
        self.set_desktop_prop(&data, self.atoms.NetCurrentDesktop);
        //set desktop names
        let mut text: xlib::XTextProperty = unsafe { std::mem::zeroed() };
        unsafe {
            let mut clist_tags: Vec<*mut c_char> = tags
                .iter()
                .map(|x| CString::new(x.clone()).unwrap_or_default().into_raw())
                .collect();
            let ptr = clist_tags.as_mut_ptr();
            (self.xlib.Xutf8TextListToTextProperty)(
                self.display,
                ptr,
                clist_tags.len() as i32,
                xlib::XUTF8StringStyle,
                &mut text,
            );
            std::mem::forget(clist_tags);
            (self.xlib.XSetTextProperty)(
                self.display,
                self.get_default_root(),
                &mut text,
                self.atoms.NetDesktopNames,
            );
        }

        //set the WM NAME
        self.set_desktop_prop_string("LeftWM", self.atoms.NetWMName);

        self.set_desktop_prop_c_ulong(
            self.get_default_root() as c_ulong,
            self.atoms.NetSupportingWmCheck,
            xlib::XA_WINDOW,
        );

        //set a viewport
        let data = vec![0_u32, 0_u32];
        self.set_desktop_prop(&data, self.atoms.NetDesktopViewport);
    }

    fn set_desktop_prop_c_ulong(&self, value: c_ulong, atom: c_ulong, type_: c_ulong) {
        let data = vec![value as u32];
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                self.get_default_root(),
                atom,
                type_,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                1_i32,
            );
            std::mem::forget(data);
        }
    }

    fn set_desktop_prop_string(&self, value: &str, atom: c_ulong) {
        if let Ok(cstring) = CString::new(value) {
            unsafe {
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.get_default_root(),
                    atom,
                    xlib::XA_CARDINAL,
                    8,
                    xlib::PropModeReplace,
                    cstring.as_ptr().cast::<u8>(),
                    value.len() as i32,
                );
                std::mem::forget(cstring);
            }
        }
    }

    fn set_desktop_prop(&self, data: &[u32], atom: c_ulong) {
        let x_data = data.to_owned();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                self.get_default_root(),
                atom,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                x_data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
            std::mem::forget(x_data);
        }
    }

    pub fn set_current_viewport(&self, tags: Vec<&String>) {
        let mut indexes: Vec<u32> = vec![];
        for tag in tags {
            for (i, mytag) in self.tags.iter().enumerate() {
                if tag.contains(mytag) {
                    indexes.push(i as u32);
                }
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.set_desktop_prop(&indexes, self.atoms.NetDesktopViewport);
    }

    pub fn set_window_desktop(&self, window: xlib::Window, current_tags: &str) {
        let mut indexes: Vec<u32> = vec![];
        for (i, tag) in self.tags.iter().enumerate() {
            if current_tags.contains(tag) {
                let tag = i as u32;
                indexes.push(tag);
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.NetWMDesktop,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                indexes.as_ptr().cast::<u8>(),
                indexes.len() as i32,
            );
            std::mem::forget(indexes);
        }
    }

    pub fn set_current_desktop(&self, current_tags: &str) {
        let mut indexes: Vec<u32> = vec![];
        for (i, tag) in self.tags.iter().enumerate() {
            if current_tags.contains(tag) {
                indexes.push(i as u32);
            }
        }
        if indexes.is_empty() {
            indexes.push(0);
        }
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    pub fn set_fullscreen(&self, window: &Window, fullscreen: bool) {
        if let WindowHandle::XlibHandle(h) = window.handle {
            let atom = self.atoms.NetWMStateFullscreen;
            let mut states = self.get_window_states_atoms(h);
            if fullscreen {
                if states.contains(&atom) {
                    return;
                }
                states.push(atom);
            } else if !fullscreen {
                let index = match states.iter().position(|s| s == &atom) {
                    Some(i) => i,
                    None => return,
                };
                states.remove(index);
            }
            self.set_window_states_atoms(h, &states);
        }
    }

    pub fn update_window(&self, window: &Window, is_focused: bool, hide_offset: i32) {
        if let WindowHandle::XlibHandle(h) = window.handle {
            if window.visible() {
                // If type dock we only need to move it
                // Also fixes issues with eww
                if window.is_unmanaged() {
                    unsafe {
                        (self.xlib.XMoveWindow)(self.display, h, window.x(), window.y());
                    }
                    return;
                }
                let mut changes = xlib::XWindowChanges {
                    x: window.x(),
                    y: window.y(),
                    width: window.width(),
                    height: window.height(),
                    border_width: window.border(),
                    sibling: 0,    //not unlocked
                    stack_mode: 0, //not unlocked
                };
                let unlock =
                    xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth;
                unsafe {
                    (self.xlib.XConfigureWindow)(self.display, h, u32::from(unlock), &mut changes);
                    (self.xlib.XSync)(self.display, 0);
                    let rw: u32 = window.width() as u32;
                    let rh: u32 = window.height() as u32;
                    (self.xlib.XMoveResizeWindow)(self.display, h, window.x(), window.y(), rw, rh);

                    let mut color: c_ulong = if is_focused {
                        self.colors.active
                    } else if window.floating() {
                        self.colors.floating
                    } else {
                        self.colors.normal
                    };
                    //Force border opacity to 0xff
                    let mut bytes = color.to_le_bytes();
                    bytes[3] = 0xff;
                    color = c_ulong::from_le_bytes(bytes);

                    (self.xlib.XSetWindowBorder)(self.display, h, color);
                }
                if !is_focused && self.focus_behaviour == FocusBehaviour::ClickTo {
                    self.ungrab_buttons(h);
                    self.grab_buttons(h, xlib::Button1, xlib::AnyModifier);
                }
                self.send_config(window);
            } else {
                unsafe {
                    //if not visible x is <---- way over there <----
                    (self.xlib.XMoveWindow)(self.display, h, hide_offset, window.y());
                }
            }
        }
    }

    //this code is run once when a window is added to the managers list of windows
    pub fn setup_managed_window(
        &mut self,
        h: WindowHandle,
        follow_mouse: bool,
    ) -> Option<DisplayEvent> {
        self.subscribe_to_window_events(&h);
        if let WindowHandle::XlibHandle(handle) = h {
            self.managed_windows.push(handle);

            //make sure the window is mapped
            unsafe {
                (self.xlib.XMapWindow)(self.display, handle);
            }

            unsafe {
                //let Xlib know we are managing this window
                let list = vec![handle];
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.get_default_root(),
                    self.atoms.NetClientList,
                    xlib::XA_WINDOW,
                    32,
                    xlib::PropModeAppend,
                    list.as_ptr().cast::<u8>(),
                    1,
                );
                std::mem::forget(list);
            }

            unsafe {
                (self.xlib.XSync)(self.display, 0);
            }

            let type_ = self.get_window_type(handle);
            if type_ == WindowType::Dock || type_ == WindowType::Desktop {
                if let Some(dock_area) = self.get_window_strut_array(handle) {
                    let dems = self.screens_area_dimensions();
                    let screen = self
                        .get_screens()
                        .iter()
                        .find(|s| s.contains_dock_area(dock_area, dems))?
                        .clone();

                    if let Some(xyhw) = dock_area.as_xyhw(dems.0, dems.1, &screen) {
                        let mut change = WindowChange::new(h);
                        change.strut = Some(xyhw.into());
                        change.type_ = Some(type_);
                        return Some(DisplayEvent::WindowChange(change));
                    }
                } else if let Ok(geo) = self.get_window_geometry(handle) {
                    let mut xyhw = Xyhw::default();
                    geo.update(&mut xyhw);
                    let mut change = WindowChange::new(h);
                    change.strut = Some(xyhw.into());
                    change.type_ = Some(type_);
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
            //make sure there is at least an empty list of _NET_WM_STATE
            let states = self.get_window_states_atoms(handle);
            self.set_window_states_atoms(handle, &states);
            self.set_wm_states(handle, &[1]);
        }
        None
    }

    pub fn set_wm_states(&self, window: xlib::Window, states: &[c_long]) {
        let data: Vec<u32> = states.iter().map(|x| *x as u32).collect();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                window,
                self.atoms.WMState,
                self.atoms.WMState,
                32,
                xlib::PropModeReplace,
                data.as_ptr().cast::<u8>(),
                data.len() as i32,
            );
            std::mem::forget(data);
        }
    }

    fn grab_mouse_clicks(&self, handle: xlib::Window) {
        self.ungrab_buttons(handle);
        //just watchout for these mouse combos so we can act on them
        self.grab_buttons(handle, xlib::Button1, self.mouse_key_mask);
        self.grab_buttons(handle, xlib::Button1, self.mouse_key_mask | xlib::ShiftMask);
        self.grab_buttons(handle, xlib::Button3, self.mouse_key_mask);
        self.grab_buttons(handle, xlib::Button3, self.mouse_key_mask | xlib::ShiftMask);
    }

    fn ungrab_buttons(&self, handle: xlib::Window) {
        //cleanup all old watches
        unsafe {
            (self.xlib.XUngrabButton)(
                self.display,
                xlib::AnyButton as u32,
                xlib::AnyModifier,
                handle,
            );
        }
    }

    /// # Errors
    ///
    /// Will error if unable to obtain window attributes. See `get_window_attrs`.
    pub fn move_cursor_to_window(&self, window: xlib::Window) -> Result<(), XlibError> {
        let attrs = self.get_window_attrs(window)?;
        let point = (attrs.x + (attrs.width / 2), attrs.y + (attrs.height / 2));
        self.move_cursor_to_point(point)
    }

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
                    self.get_default_root(),
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

    /// # Errors
    ///
    /// Will error if root window cannot be found.
    pub fn get_cursor_point(&self) -> Result<(i32, i32), XlibError> {
        let roots = self.get_roots(); //each screen
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

    #[must_use]
    pub fn screens_area_dimensions(&self) -> (i32, i32) {
        let mut height = 0;
        let mut width = 0;
        for s in self.get_screens() {
            height = std::cmp::max(height, s.bbox.height + s.bbox.y);
            width = std::cmp::max(width, s.bbox.width + s.bbox.x);
        }
        (height, width)
    }

    #[must_use]
    pub fn get_window_strut_array(&self, window: xlib::Window) -> Option<DockArea> {
        if let Some(d) = self.get_window_strut_array_strut_partial(window) {
            log::debug!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        if let Some(d) = self.get_window_strut_array_strut(window) {
            log::debug!("STRUT:[{:?}] {:?}", window, d);
            return Some(d);
        }
        None
    }

    //new way to get strut
    fn get_window_strut_array_strut_partial(&self, window: xlib::Window) -> Option<DockArea> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut bytes_after_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                self.atoms.NetWMStrutPartial,
                0,
                MAX_PROPERTY_VALUE_LEN,
                xlib::False,
                xlib::XA_CARDINAL,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) {
                #[allow(clippy::cast_ptr_alignment)]
                let array_ptr = prop_return as *const c_long;
                let slice = slice::from_raw_parts(array_ptr, nitems_return as usize);
                if slice.len() == 12 {
                    return Some(DockArea::from(slice));
                }
                None
            } else {
                None
            }
        }
    }

    //old way to get strut
    fn get_window_strut_array_strut(&self, window: xlib::Window) -> Option<DockArea> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut bytes_after_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                self.atoms.NetWMStrut,
                0,
                MAX_PROPERTY_VALUE_LEN,
                xlib::False,
                xlib::XA_CARDINAL,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) {
                #[allow(clippy::cast_ptr_alignment)]
                let array_ptr = prop_return as *const c_long;
                let slice = slice::from_raw_parts(array_ptr, nitems_return as usize);
                if slice.len() == 12 {
                    return Some(DockArea::from(slice));
                }
                None
            } else {
                None
            }
        }
    }

    //this code is run once when a window is destroyed
    pub fn teardown_managed_window(&mut self, h: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            unsafe {
                (self.xlib.XGrabServer)(self.display);

                //remove this window from the list of managed windows
                self.managed_windows.retain(|x| *x != *handle);
                self.update_client_list();

                //ungrab all buttons for this window
                (self.xlib.XUngrabButton)(
                    self.display,
                    xlib::AnyButton as u32,
                    xlib::AnyModifier,
                    *handle,
                );
                (self.xlib.XSync)(self.display, 0);
                (self.xlib.XUngrabServer)(self.display);
            }
        }
    }

    pub fn force_unmapped(&mut self, window: xlib::Window) {
        let managed = self.managed_windows.contains(&window);
        if managed {
            self.managed_windows.retain(|x| *x != window);
            self.update_client_list();
        }
    }

    fn update_client_list(&self) {
        unsafe {
            (self.xlib.XDeleteProperty)(
                self.display,
                self.get_default_root(),
                self.atoms.NetClientList,
            );
            for w in &self.managed_windows {
                let list = vec![*w];
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.get_default_root(),
                    self.atoms.NetClientList,
                    xlib::XA_WINDOW,
                    32,
                    xlib::PropModeAppend,
                    list.as_ptr().cast::<u8>(),
                    1,
                );
                std::mem::forget(list);
            }
        }
    }

    /// Used to send and `XConfigureEvent` for a changed window to the xserver .
    pub fn send_config(&self, window: &Window) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            let config = xlib::XConfigureEvent {
                type_: xlib::ConfigureNotify,
                serial: 0, //not used
                send_event: 0,
                display: self.display,
                event: handle,
                window: handle,
                x: window.x(),
                y: window.y(),
                width: window.width(),
                height: window.height(),
                border_width: window.border(),
                above: 0,
                override_redirect: 0,
            };
            unsafe {
                let mut event: xlib::XEvent = xlib::XConfigureEvent::into(config);
                (self.xlib.XSendEvent)(
                    self.display,
                    handle,
                    0,
                    xlib::StructureNotifyMask,
                    &mut event,
                );
            }
        }
    }

    fn send_xevent_atom(&self, window: xlib::Window, atom: xlib::Atom) -> bool {
        if self.can_send_xevent_atom(window, atom) {
            let mut msg: xlib::XClientMessageEvent = unsafe { std::mem::zeroed() };
            msg.type_ = xlib::ClientMessage;
            msg.window = window;
            msg.message_type = self.atoms.WMProtocols;
            msg.format = 32;
            msg.data.set_long(0, atom as c_long);
            msg.data.set_long(1, xlib::CurrentTime as c_long);
            let mut ev: xlib::XEvent = msg.into();
            unsafe { (self.xlib.XSendEvent)(self.display, window, 0, xlib::NoEventMask, &mut ev) };
            return true;
        }
        false
    }

    //return true if the underlying window exsepts this type of atom:protocal
    fn can_send_xevent_atom(&self, window: xlib::Window, atom: xlib::Atom) -> bool {
        unsafe {
            let mut array: *mut xlib::Atom = std::mem::zeroed();
            let mut length: c_int = std::mem::zeroed();
            let status: xlib::Status =
                (self.xlib.XGetWMProtocols)(self.display, window, &mut array, &mut length);
            let protocols: &[xlib::Atom] = slice::from_raw_parts(array, length as usize);
            status > 0 && protocols.contains(&atom)
        }
    }

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

    #[must_use]
    pub fn get_window_pid(&self, window: xlib::Window) -> Option<u32> {
        if let Ok(id) = self.get_cardinal_prop(window, self.atoms.NetWMPid) {
            return Some(id);
        }
        None
    }

    /// Get the `WMPid` of a window
    /// # Errors
    ///
    /// Errors if window status = 0.
    pub fn get_cardinal_prop(
        &self,
        window: xlib::Window,
        atom: xlib::Atom,
    ) -> Result<u32, XlibError> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::zeroed() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                atom,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                xlib::XA_CARDINAL,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut nitems_return,
                &mut prop_return,
            );
            if status == i32::from(xlib::Success) && !prop_return.is_null() {
                #[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
                let pid = *(prop_return as *const u32);
                return Ok(pid);
            }
        };
        Err(XlibError::FailedStatus)
    }

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
    /// Get the `WMName` of a window
    /// # Errors
    ///
    /// Errors if window status = 0.
    pub fn get_text_prop(
        &self,
        window: xlib::Window,
        atom: xlib::Atom,
    ) -> Result<String, XlibError> {
        unsafe {
            let mut ptr: *mut *mut c_char = std::mem::zeroed();
            let mut ptr_len: c_int = 0;
            let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
            let status: c_int =
                (self.xlib.XGetTextProperty)(self.display, window, &mut text_prop, atom);
            if status == 0 {
                return Err(XlibError::FailedStatus);
            }
            if text_prop.encoding == xlib::XA_STRING {
                (self.xlib.XTextPropertyToStringList)(&mut text_prop, &mut ptr, &mut ptr_len);
            } else {
                (self.xlib.XmbTextPropertyToTextList)(
                    self.display,
                    &mut text_prop,
                    &mut ptr,
                    &mut ptr_len,
                );
            }
            for _i in 0..ptr_len {
                if let Ok(s) = CString::from_raw(*ptr).into_string() {
                    return Ok(s);
                }
            }
        };
        Err(XlibError::FailedStatus)
    }

    /// Get the `XAtom` name
    /// # Errors
    ///
    /// Errors if `XAtom` is not valid.
    pub fn get_xatom_name(&self, atom: xlib::Atom) -> Result<String, XlibError> {
        unsafe {
            let cstring = (self.xlib.XGetAtomName)(self.display, atom);
            if let Ok(s) = CString::from_raw(cstring).into_string() {
                return Ok(s);
            }
        };
        Err(XlibError::InvalidXAtom)
    }

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

    pub fn move_to_top(&self, handle: &WindowHandle) {
        if let WindowHandle::XlibHandle(window) = handle {
            unsafe {
                (self.xlib.XRaiseWindow)(self.display, *window);
            }
        }
    }

    /// Obtains window geometry in an `XyhwChange`struct from `Xlib`.
    /// # Errors
    ///
    /// Errors if Xlib returns a status of 0.
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

    pub fn window_take_focus(&self, window: &Window) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            self.grab_mouse_clicks(handle);

            if !window.never_focus {
                //mark this window as the NetActiveWindow
                unsafe {
                    (self.xlib.XSetInputFocus)(
                        self.display,
                        handle,
                        xlib::RevertToPointerRoot,
                        xlib::CurrentTime,
                    );
                    let list = vec![handle];
                    (self.xlib.XChangeProperty)(
                        self.display,
                        self.get_default_root(),
                        self.atoms.NetActiveWindow,
                        xlib::XA_WINDOW,
                        32,
                        xlib::PropModeReplace,
                        list.as_ptr().cast::<u8>(),
                        1,
                    );
                    std::mem::forget(list);
                }
            }

            //tell the window to take focus
            self.send_xevent_atom(handle, self.atoms.WMTakeFocus);
        }
    }

    pub fn unfocus(&self) {
        let handle = self.get_default_root();
        unsafe {
            (self.xlib.XSetInputFocus)(self.display, handle, xlib::RevertToNone, xlib::CurrentTime);
            (self.xlib.XChangeProperty)(
                self.display,
                self.get_default_root(),
                self.atoms.NetActiveWindow,
                xlib::XA_WINDOW,
                32,
                xlib::PropModeReplace,
                vec![c_ulong::MAX].as_ptr().cast::<u8>(),
                1,
            );
        }
    }

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

    pub fn subscribe_to_event(&self, window: xlib::Window, mask: c_long) {
        unsafe {
            (self.xlib.XSelectInput)(self.display, window, mask);
        }
    }

    pub fn subscribe_to_window_events(&self, handle: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = handle {
            let mask = xlib::EnterWindowMask
                | xlib::FocusChangeMask
                | xlib::PropertyChangeMask
                | xlib::StructureNotifyMask;
            self.subscribe_to_event(*handle, mask);
        }
    }

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

    #[must_use]
    pub fn get_hint_sizing(&self, window: xlib::Window) -> Option<xlib::XSizeHints> {
        let mut xsize: xlib::XSizeHints = unsafe { std::mem::zeroed() };
        let mut msize: c_long = xlib::PSize;
        let status =
            unsafe { (self.xlib.XGetWMNormalHints)(self.display, window, &mut xsize, &mut msize) };
        match status {
            0 => None,
            _ => Some(xsize),
        }
    }

    #[must_use]
    pub fn get_hint_sizing_as_xyhw(&self, window: xlib::Window) -> Option<XyhwChange> {
        let hint = self.get_hint_sizing(window);
        if let Some(size) = hint {
            let mut xyhw = XyhwChange::default();

            if (size.flags & xlib::PBaseSize) != 0 {
                xyhw.w = Some(size.base_width);
                xyhw.h = Some(size.base_height);
            } else if (size.flags & xlib::PMinSize) != 0 {
                xyhw.minw = Some(size.min_width);
                xyhw.minh = Some(size.min_height);
            }

            if size.flags & xlib::PResizeInc != 0 {
                xyhw.w = Some(size.width_inc);
                xyhw.h = Some(size.height_inc);
            }

            if size.flags & xlib::PMaxSize != 0 {
                xyhw.maxw = Some(size.max_width);
                xyhw.maxh = Some(size.max_height);
            }

            if size.flags & xlib::PMinSize != 0 {
                xyhw.minw = Some(size.min_width);
                xyhw.minh = Some(size.min_height);
            } else if size.flags & xlib::PBaseSize != 0 {
                xyhw.w = Some(size.base_width);
                xyhw.h = Some(size.base_height);
            }

            //TODO: support min/max aspect
            //if size.flags & xlib::PAspect != 0 {
            //    //c->mina = (float)size.min_aspect.y / size.min_aspect.x;
            //    //c->maxa = (float)size.max_aspect.x / size.max_aspect.y;
            //}

            return Some(xyhw);
        }
        None
    }

    pub fn grab_buttons(&self, window: xlib::Window, button: u32, modifiers: u32) {
        //grab the buttons with and without numlock (Mod2)
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

    pub fn grab_keys(&self, root: xlib::Window, keysym: u32, modifiers: u32) {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, c_ulong::from(keysym)) };
        //grab the keys with and without numlock (Mod2)
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

    pub fn load_colors(&mut self, theme: &ThemeSetting) {
        self.colors = Colors {
            normal: self.get_color(&theme.default_border_color),
            floating: self.get_color(&theme.floating_border_color),
            active: self.get_color(&theme.focused_border_color),
        };
    }

    fn get_color(&self, color: &str) -> c_ulong {
        let screen = unsafe { (self.xlib.XDefaultScreen)(self.display) };
        let cmap: xlib::Colormap = unsafe { (self.xlib.XDefaultColormap)(self.display, screen) };
        let color_cstr = CString::new(color).unwrap_or_default().into_raw();
        let mut color: xlib::XColor = unsafe { std::mem::zeroed() };
        unsafe {
            (self.xlib.XAllocNamedColor)(self.display, cmap, color_cstr, &mut color, &mut color);
        }
        color.pixel
    }

    // TODO: split into smaller functions
    pub fn init(&mut self, config: &impl Config, theme: &ThemeSetting) {
        let root_event_mask: c_long = xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::ButtonPressMask
            | xlib::PointerMotionMask
            | xlib::EnterWindowMask
            | xlib::LeaveWindowMask
            | xlib::StructureNotifyMask
            | xlib::PropertyChangeMask;

        let root = self.get_default_root();
        self.load_colors(theme);

        let mut attrs: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attrs.cursor = self.cursors.normal;
        attrs.event_mask = root_event_mask;

        unsafe {
            (self.xlib.XChangeWindowAttributes)(
                self.display,
                self.get_default_root(),
                xlib::CWEventMask | xlib::CWCursor,
                &mut attrs,
            );
        }

        self.subscribe_to_event(root, root_event_mask);

        //EWMH junk
        unsafe {
            let supported = self.atoms.net_supported();
            let supported_ptr: *const xlib::Atom = supported.as_ptr();
            let size = supported.len() as i32;
            (self.xlib.XChangeProperty)(
                self.display,
                root,
                self.atoms.NetSupported,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                supported_ptr.cast::<u8>(),
                size,
            );
            std::mem::forget(supported);
            //cleanup the client list
            (self.xlib.XDeleteProperty)(self.display, root, self.atoms.NetClientList);
        }

        //EWMH stuff for desktops
        self.tags = config.create_list_of_tags();
        self.init_desktops_hints();

        self.reset_grabs(&config.mapped_bindings());

        unsafe {
            (self.xlib.XSync)(self.display, 0);
        }
    }

    /// Cleans first all old keygrabs and then reaplies them from the config
    pub fn reset_grabs(&self, keybinds: &[Keybind]) {
        //cleanup grabs
        unsafe {
            (self.xlib.XUngrabKey)(
                self.display,
                xlib::AnyKey,
                xlib::AnyModifier,
                self.get_default_root(),
            );
        }

        //grab all the key combos from the config file
        for kb in keybinds {
            if let Some(keysym) = utils::xkeysym_lookup::into_keysym(&kb.key) {
                let modmask = utils::xkeysym_lookup::into_modmask(&kb.modifier);
                self.grab_keys(self.get_default_root(), keysym, modmask);
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        //prevent resizing and moveing or root
        match &mode {
            Mode::MovingWindow(h) | Mode::ResizingWindow(h) => {
                if h == &self.get_default_root_handle() {
                    return;
                }
            }
            Mode::Normal => {}
        }
        if self.mode == Mode::Normal && mode != Mode::Normal {
            self.mode = mode.clone();
            //safe this point as the start of the move/resize
            if let Ok(loc) = self.get_cursor_point() {
                self.mode_origin = loc;
            }
            let cursor = match mode {
                Mode::ResizingWindow(_) => self.cursors.resize,
                Mode::MovingWindow(_) => self.cursors.move_,
                Mode::Normal => self.cursors.normal,
            };
            self.grab_pointer(cursor);
        }
        if mode == Mode::Normal {
            self.ungrab_pointer();
            self.mode = mode;
        }
    }

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

    pub fn ungrab_pointer(&self) {
        unsafe {
            //release the mouse grab
            (self.xlib.XUngrabPointer)(self.display, xlib::CurrentTime);
        }
    }

    pub fn replay_click(&self, mod_mask: ModMask) {
        // Only replay the click when in ClickToFocus and we are not trying to move/resize the
        // window
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

    #[must_use]
    pub fn get_next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe { std::mem::zeroed() };
        unsafe {
            (self.xlib.XNextEvent)(self.display, &mut event);
        };
        event
    }

    pub async fn wait_readable(&mut self) {
        self.task_notify.notified().await;
    }

    pub fn flush(&self) {
        unsafe { (self.xlib.XFlush)(self.display) };
    }

    #[must_use]
    pub fn queue_len(&self) -> i32 {
        unsafe { (self.xlib.XPending)(self.display) }
    }
}
