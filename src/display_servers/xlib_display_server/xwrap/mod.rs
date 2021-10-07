//! A wrapper around calls to xlib and X related functions.
// We allow this _ because if we don't we'll receive an error that it isn't read on _task_guard.
#![allow(clippy::used_underscore_binding)]
// We allow this so that extern "C" functions are not flagged as confusing. The current placement
// allows for easy reading.
#![allow(clippy::items_after_statements)]
// We allow this because _y_ and _x_ are intentionally similar. Changing it makes the code noisy.
#![allow(clippy::similar_names)]
use super::xatom::XAtom;
use super::xcursor::XCursor;
use super::{utils, Screen, Window, WindowHandle};
use crate::config::Config;
use crate::models::{FocusBehaviour, Mode};
use crate::utils::xkeysym_lookup::ModMask;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_ulong};
use std::sync::Arc;
use std::{ptr, slice};
use tokio::sync::{oneshot, Notify};
use tokio::time::Duration;
use x11_dl::xlib;

mod getters;
mod keyboard;
mod mouse;
mod setters;
mod window;

type WindowStateConst = u8;
// const WITHDRAWN_STATE: WindowStateConst = 0;
const NORMAL_STATE: WindowStateConst = 1;
// const ICONIC_STATE: WindowStateConst = 2;
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
    pub task_notify: Arc<Notify>,
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
            (xw.xlib.XSync)(xw.display, xlib::False);
        };

        // This is allowed for now as const extern fns
        // are not yet stable (1.56.0, 16 Sept 2021)
        // see issue #64926 <https://github.com/rust-lang/rust/issues/64926> for more information.
        #[allow(clippy::missing_const_for_fn)]
        extern "C" fn on_error_from_xlib(
            _: *mut xlib::Display,
            er: *mut xlib::XErrorEvent,
        ) -> c_int {
            let err = unsafe { *er };
            // Ignore bad window errors.
            if err.error_code == xlib::BadWindow {
                return 0;
            }
            1
        }

        // Setup cached keymap/modifier information, otherwise MappingNotify might never be called
        // from:
        // https://stackoverflow.com/questions/35569562/how-to-catch-keyboard-layout-change-event-and-get-current-new-keyboard-layout-on
        xw.keysym_to_keycode(x11_dl::keysym::XK_F1);

        unsafe {
            (xw.xlib.XSetErrorHandler)(Some(on_error_from_xlib));
            (xw.xlib.XSync)(xw.display, xlib::False);
        };
        xw
    }

    /// Initialize the xwrapper.
    /// TODO: split into smaller functions
    pub fn init(&mut self, config: &impl Config) {
        let root_event_mask: c_long = xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::ButtonPressMask
            | xlib::PointerMotionMask
            | xlib::EnterWindowMask
            | xlib::LeaveWindowMask
            | xlib::StructureNotifyMask
            | xlib::PropertyChangeMask;

        let root = self.root;
        self.load_colors(config);

        let mut attrs: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attrs.cursor = self.cursors.normal;
        attrs.event_mask = root_event_mask;

        unsafe {
            (self.xlib.XChangeWindowAttributes)(
                self.display,
                self.root,
                xlib::CWEventMask | xlib::CWCursor,
                &mut attrs,
            );
        }

        self.subscribe_to_event(root, root_event_mask);

        // EWMH compliance.
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
            // Cleanup the client list.
            (self.xlib.XDeleteProperty)(self.display, root, self.atoms.NetClientList);
        }

        // EWMH compliance for desktops.
        self.tags = config.create_list_of_tags();
        self.init_desktops_hints();

        self.reset_grabs(&config.mapped_bindings());

        unsafe {
            (self.xlib.XSync)(self.display, 0);
        }
    }

    /// EWMH support used for bars such as polybar.
    ///  # Panics
    ///
    ///  Panics if a new Cstring cannot be formed
    pub fn init_desktops_hints(&self) {
        let tags = &self.tags;
        let tag_length = tags.len();
        // Set the number of desktop.
        let data = vec![tag_length as u32];
        self.set_desktop_prop(&data, self.atoms.NetNumberOfDesktops);
        // Set a current desktop.
        let data = vec![0_u32, xlib::CurrentTime as u32];
        self.set_desktop_prop(&data, self.atoms.NetCurrentDesktop);
        // Set desktop names.
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
                self.root,
                &mut text,
                self.atoms.NetDesktopNames,
            );
        }

        // Set the WM NAME.
        self.set_desktop_prop_string("LeftWM", self.atoms.NetWMName);

        self.set_desktop_prop_c_ulong(
            self.root as c_ulong,
            self.atoms.NetSupportingWmCheck,
            xlib::XA_WINDOW,
        );

        // Set a viewport.
        let data = vec![0_u32, 0_u32];
        self.set_desktop_prop(&data, self.atoms.NetDesktopViewport);
    }

    /// Send a `XConfigureEvent` for a window to X.
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

    /// Send a xevent atom for a window to X.
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

    /// Returns whether a window can recieve a xevent atom.
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
    pub fn load_colors(&mut self, config: &impl Config) {
        self.colors = Colors {
            normal: self.get_color(config.default_border_color()),
            floating: self.get_color(config.floating_border_color()),
            active: self.get_color(config.focused_border_color()),
        };
    }

    /// Sets the mode within our xwrapper.
    pub fn set_mode(&mut self, mode: Mode) {
        // Prevent resizing and moving of root.
        match &mode {
            Mode::MovingWindow(h) | Mode::ResizingWindow(h) => {
                if h == &self.get_default_root_handle() {
                    return;
                }
            }
            Mode::Normal => {}
        }
        if self.mode == Mode::Normal && mode != Mode::Normal {
            self.mode = mode;
            // Safe at this point as the move/resize has started.
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

    /// Wait until readable.
    pub async fn wait_readable(&mut self) {
        self.task_notify.notified().await;
    }

    /// Flush the xserver.
    pub fn flush(&self) {
        unsafe { (self.xlib.XFlush)(self.display) };
    }

    /// Returns how many events are waiting.
    #[must_use]
    pub fn queue_len(&self) -> i32 {
        unsafe { (self.xlib.XPending)(self.display) }
    }
}
