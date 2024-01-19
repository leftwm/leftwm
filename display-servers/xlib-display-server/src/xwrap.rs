//! A wrapper around calls to xlib and X related functions.
// We allow this _ because if we don't we'll receive an error that it isn't read on _task_guard.
#![allow(clippy::used_underscore_binding)]
// We allow this so that extern "C" functions are not flagged as confusing. The current placement
// allows for easy reading.
#![allow(clippy::items_after_statements)]
// We allow this because _y_ and _x_ are intentionally similar. Changing it makes the code noisy.
#![allow(clippy::similar_names)]
use crate::XlibWindowHandle;

use super::xatom::XAtom;
use super::xcursor::XCursor;
use super::{utils, Screen, Window, WindowHandle};
use leftwm_core::config::Config;
use leftwm_core::models::{FocusBehaviour, Mode};
use leftwm_core::utils::modmask_lookup::ModMask;
use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_long, c_short, c_ulong};
use std::sync::Arc;
use std::{ptr, slice};
use tokio::sync::{oneshot, Notify};
use tokio::time::Duration;

use x11_dl::xlib;
use x11_dl::xrandr::Xrandr;

mod getters;
mod mouse;
mod setters;
mod window;

type WindowStateConst = c_long;
pub const WITHDRAWN_STATE: WindowStateConst = 0;
pub const NORMAL_STATE: WindowStateConst = 1;
pub const ICONIC_STATE: WindowStateConst = 2;
const MAX_PROPERTY_VALUE_LEN: c_long = 4096;

pub const ROOT_EVENT_MASK: c_long = xlib::SubstructureRedirectMask
    | xlib::SubstructureNotifyMask
    | xlib::ButtonPressMask
    | xlib::PointerMotionMask
    | xlib::StructureNotifyMask;

const BUTTONMASK: c_long = xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::ButtonMotionMask;
const MOUSEMASK: c_long = BUTTONMASK | xlib::PointerMotionMask;

const X_CONFIGUREWINDOW: u8 = 12;
const X_GRABBUTTON: u8 = 28;
const X_GRABKEY: u8 = 33;
const X_SETINPUTFOCUS: u8 = 42;
const X_COPYAREA: u8 = 62;
const X_POLYSEGMENT: u8 = 66;
const X_POLYFILLRECTANGLE: u8 = 70;
const X_POLYTEXT8: u8 = 74;

// This is allowed for now as const extern fns
// are not yet stable (1.56.0, 16 Sept 2021)
// see issue #64926 <https://github.com/rust-lang/rust/issues/64926> for more information.
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn on_error_from_xlib(_: *mut xlib::Display, er: *mut xlib::XErrorEvent) -> c_int {
    let err = unsafe { *er };
    let ec = err.error_code;
    let rc = err.request_code;
    let ba = ec == xlib::BadAccess;
    let bd = ec == xlib::BadDrawable;
    let bm = ec == xlib::BadMatch;

    if ec == xlib::BadWindow
        || (rc == X_CONFIGUREWINDOW && bm)
        || (rc == X_GRABBUTTON && ba)
        || (rc == X_GRABKEY && ba)
        || (rc == X_SETINPUTFOCUS && bm)
        || (rc == X_COPYAREA && bd)
        || (rc == X_POLYSEGMENT && bd)
        || (rc == X_POLYFILLRECTANGLE && bd)
        || (rc == X_POLYTEXT8 && bd)
    {
        return 0;
    }
    1
}

pub extern "C" fn on_error_from_xlib_dummy(
    _: *mut xlib::Display,
    _: *mut xlib::XErrorEvent,
) -> c_int {
    1
}

pub struct Colors {
    normal: c_ulong,
    floating: c_ulong,
    active: c_ulong,
    background: c_ulong,
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
    pub managed_windows: Vec<xlib::Window>,
    pub focused_window: xlib::Window,
    pub tag_labels: Vec<String>,
    pub mode: Mode<XlibWindowHandle>,
    pub focus_behaviour: FocusBehaviour,
    pub mouse_key_mask: ModMask,
    pub mode_origin: (i32, i32),
    _task_guard: oneshot::Receiver<()>,
    pub task_notify: Arc<Notify>,
    pub motion_event_limiter: c_ulong,
    pub refresh_rate: c_short,
}

impl Default for XWrap {
    fn default() -> Self {
        Self::new()
    }
}

impl XWrap {
    /// # Panics
    ///
    /// Panics if unable to contact xorg.
    // TODO: Split this function up.
    // `XOpenDisplay`: https://tronche.com/gui/x/xlib/display/opening.html
    // `XConnectionNumber`: https://tronche.com/gui/x/xlib/display/display-macros.html#ConnectionNumber
    // `XDefaultRootWindow`: https://tronche.com/gui/x/xlib/display/display-macros.html#DefaultRootWindow
    // `XSetErrorHandler`: https://tronche.com/gui/x/xlib/event-handling/protocol-errors/XSetErrorHandler.html
    // `XSelectInput`: https://tronche.com/gui/x/xlib/event-handling/XSelectInput.html
    #[must_use]
    #[allow(clippy::too_many_lines)]
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
                tracing::warn!("Xlib socket poll failed with {:?}", err);
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
            background: 0,
        };

        let refresh_rate = match Xrandr::open() {
            // Get the current refresh rate from xrandr if available.
            Ok(xrandr) => unsafe {
                let screen_resources = (xrandr.XRRGetScreenResources)(display, root);
                let crtcs = slice::from_raw_parts(
                    (*screen_resources).crtcs,
                    (*screen_resources).ncrtc as usize,
                );
                let active_modes: Vec<c_ulong> = crtcs
                    .iter()
                    .map(|crtc| (xrandr.XRRGetCrtcInfo)(display, screen_resources, *crtc))
                    .filter(|&crtc_info| (*crtc_info).mode != 0)
                    .map(|crtc_info| (*crtc_info).mode)
                    .collect();
                let modes = slice::from_raw_parts(
                    (*screen_resources).modes,
                    (*screen_resources).nmode as usize,
                );
                modes
                    .iter()
                    .filter(|mode_info| active_modes.contains(&mode_info.id))
                    .map(|mode_info| {
                        (mode_info.dotClock as c_double
                            / c_double::from(mode_info.hTotal * mode_info.vTotal))
                            as c_short
                    })
                    .max()
                    .unwrap_or(60)
            },
            Err(_) => 60,
        };

        tracing::debug!("Refresh Rate: {}", refresh_rate);

        let xw = Self {
            xlib,
            display,
            root,
            atoms,
            cursors,
            colors,
            managed_windows: vec![],
            focused_window: root,
            tag_labels: vec![],
            mode: Mode::Normal,
            focus_behaviour: FocusBehaviour::Sloppy,
            mouse_key_mask: ModMask::Zero,
            mode_origin: (0, 0),
            _task_guard,
            task_notify,
            motion_event_limiter: 0,
            refresh_rate,
        };

        // Check that another WM is not running.
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
        };
        xw.sync();

        unsafe { (xw.xlib.XSetErrorHandler)(Some(on_error_from_xlib)) };
        xw.sync();
        xw
    }

    pub fn load_config(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle<XlibWindowHandle>>>,
        windows: &[Window<XlibWindowHandle>],
    ) {
        self.focus_behaviour = config.focus_behaviour();
        self.mouse_key_mask = utils::modmask_lookup::into_modmask(&config.mousekey());
        self.load_colors(config, focused, Some(windows));
        self.tag_labels = config.create_list_of_tag_labels();
    }

    /// Initialize the xwrapper.
    // `XChangeWindowAttributes`: https://tronche.com/gui/x/xlib/window/XChangeWindowAttributes.html
    // `XDeleteProperty`: https://tronche.com/gui/x/xlib/window-information/XDeleteProperty.html
    // TODO: split into smaller functions
    pub fn init(&mut self, config: &impl Config) {
        self.focus_behaviour = config.focus_behaviour();
        self.mouse_key_mask = utils::modmask_lookup::into_modmask(&config.mousekey());

        let root = self.root;
        self.load_colors(config, None, None);

        let mut attrs: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attrs.cursor = self.cursors.normal;
        attrs.event_mask = ROOT_EVENT_MASK;

        unsafe {
            (self.xlib.XChangeWindowAttributes)(
                self.display,
                self.root,
                xlib::CWEventMask | xlib::CWCursor,
                &mut attrs,
            );
        }

        self.subscribe_to_event(root, ROOT_EVENT_MASK);

        // EWMH compliance.
        unsafe {
            let supported: Vec<c_long> = self
                .atoms
                .net_supported()
                .iter()
                .map(|&atom| atom as c_long)
                .collect();
            self.replace_property_long(root, self.atoms.NetSupported, xlib::XA_ATOM, &supported);
            std::mem::forget(supported);
            // Cleanup the client list.
            (self.xlib.XDeleteProperty)(self.display, root, self.atoms.NetClientList);
        }

        // EWMH compliance for desktops.
        self.tag_labels = config.create_list_of_tag_labels();
        self.init_desktops_hints();

        self.sync();
    }

    /// EWMH support used for bars such as polybar.
    ///  # Panics
    ///
    ///  Panics if a new Cstring cannot be formed
    // `Xutf8TextListToTextProperty`: https://linux.die.net/man/3/xutf8textlisttotextproperty
    // `XSetTextProperty`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XSetTextProperty.html
    pub fn init_desktops_hints(&self) {
        let tag_labels = &self.tag_labels;
        let tag_length = tag_labels.len();
        // Set the number of desktop.
        let data = vec![tag_length as u32];
        self.set_desktop_prop(&data, self.atoms.NetNumberOfDesktops);
        // Set a current desktop.
        let data = vec![0_u32, xlib::CurrentTime as u32];
        self.set_desktop_prop(&data, self.atoms.NetCurrentDesktop);
        // Set desktop names.
        let mut text: xlib::XTextProperty = unsafe { std::mem::zeroed() };
        unsafe {
            let mut clist_tags: Vec<*mut c_char> = tag_labels
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
                self.root,
                &mut text,
                self.atoms.NetDesktopNames,
            );
        }

        // Set the WM NAME.
        self.set_desktop_prop_string("LeftWM", self.atoms.NetWMName, self.atoms.UTF8String);

        self.set_desktop_prop_string("LeftWM", self.atoms.WMClass, xlib::XA_STRING);

        self.set_desktop_prop_c_ulong(
            self.root as c_ulong,
            self.atoms.NetSupportingWmCheck,
            xlib::XA_WINDOW,
        );

        // Set a viewport.
        let data = vec![0_u32, 0_u32];
        self.set_desktop_prop(&data, self.atoms.NetDesktopViewport);
    }

    /// Send a xevent atom for a window to X.
    // `XSendEvent`: https://tronche.com/gui/x/xlib/event-handling/XSendEvent.html
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
            self.send_xevent(window, 0, xlib::NoEventMask, &mut ev);
            return true;
        }
        false
    }

    /// Send a xevent for a window to X.
    // `XSendEvent`: https://tronche.com/gui/x/xlib/event-handling/XSendEvent.html
    pub fn send_xevent(
        &self,
        window: xlib::Window,
        propogate: i32,
        mask: c_long,
        event: &mut xlib::XEvent,
    ) {
        unsafe { (self.xlib.XSendEvent)(self.display, window, propogate, mask, event) };
        self.sync();
    }

    /// Returns whether a window can recieve a xevent atom.
    // `XGetWMProtocols`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetWMProtocols.html
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

    /// Load the colors of our theme.
    pub fn load_colors(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle<XlibWindowHandle>>>,
        windows: Option<&[Window<XlibWindowHandle>]>,
    ) {
        self.colors = Colors {
            normal: self.get_color(config.default_border_color()),
            floating: self.get_color(config.floating_border_color()),
            active: self.get_color(config.focused_border_color()),
            background: self.get_color(config.background_color()),
        };
        // Update all the windows with the new colors.
        if let Some(windows) = windows {
            for window in windows {
                if let WindowHandle(XlibWindowHandle(handle)) = window.handle {
                    let is_focused =
                        matches!(focused, Some(&Some(focused)) if focused == window.handle);
                    let color: c_ulong = if is_focused {
                        self.colors.active
                    } else if window.floating() {
                        self.colors.floating
                    } else {
                        self.colors.normal
                    };
                    self.set_window_border_color(handle, color);
                }
            }
        }
        self.set_background_color(self.colors.background);
    }

    /// Sets the mode within our xwrapper.
    pub fn set_mode(&mut self, mode: Mode<XlibWindowHandle>) {
        match mode {
            // Prevent resizing and moving of root.
            Mode::MovingWindow(h)
            | Mode::ResizingWindow(h)
            | Mode::ReadyToMove(h)
            | Mode::ReadyToResize(h)
                if h == self.get_default_root_handle() => {}
            Mode::ReadyToMove(_) | Mode::ReadyToResize(_) if self.mode == Mode::Normal => {
                self.mode = mode;
                if let Ok(loc) = self.get_cursor_point() {
                    self.mode_origin = loc;
                }
                let cursor = match mode {
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => self.cursors.resize,
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => self.cursors.move_,
                    Mode::Normal => self.cursors.normal,
                };
                self.grab_pointer(cursor);
            }
            Mode::MovingWindow(h) | Mode::ResizingWindow(h)
                if self.mode == Mode::ReadyToMove(h) || self.mode == Mode::ReadyToResize(h) =>
            {
                self.ungrab_pointer();
                self.mode = mode;
                let cursor = match mode {
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => self.cursors.resize,
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => self.cursors.move_,
                    Mode::Normal => self.cursors.normal,
                };
                self.grab_pointer(cursor);
            }
            Mode::Normal => {
                self.ungrab_pointer();
                self.mode = mode;
            }
            _ => {}
        }
    }

    /// Wait until readable.
    pub async fn wait_readable(&mut self) {
        self.task_notify.notified().await;
    }

    /// Flush and sync the xserver.
    // `XSync`: https://tronche.com/gui/x/xlib/event-handling/XSync.html
    pub fn sync(&self) {
        unsafe { (self.xlib.XSync)(self.display, xlib::False) };
    }

    /// Flush the xserver.
    // `XFlush`: https://tronche.com/gui/x/xlib/event-handling/XFlush.html
    pub fn flush(&self) {
        unsafe { (self.xlib.XFlush)(self.display) };
    }

    /// Returns how many events are waiting.
    // `XPending`: https://tronche.com/gui/x/xlib/event-handling/XPending.html
    #[must_use]
    pub fn queue_len(&self) -> i32 {
        unsafe { (self.xlib.XPending)(self.display) }
    }
}
