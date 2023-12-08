use std::io::IoSlice;

use leftwm_core::{
    models::{FocusBehaviour, WindowHandle},
    utils::{self, modmask_lookup::ModMask},
    Config, Mode, Window,
};
use x11rb::{
    connection::{Connection, RequestConnection},
    cursor::Handle as CursorHandle,
    protocol::{
        randr::ConnectionExt as RandrConnectionExtention,
        xproto::{self, ChangeWindowAttributesAux},
    },
    resource_manager::Database,
    rust_connection::RustConnection,
    x11_utils::Serialize,
};

use crate::xatom::AtomCollection;

mod getters;
mod mouse;
mod setters;
mod window;

const MAX_PROPERTY_VALUE_LEN: u32 = 4096;

#[inline]
pub fn root_event_mask() -> xproto::EventMask {
    xproto::EventMask::SUBSTRUCTURE_REDIRECT
        | xproto::EventMask::SUBSTRUCTURE_NOTIFY
        | xproto::EventMask::BUTTON_PRESS
        | xproto::EventMask::POINTER_MOTION
        | xproto::EventMask::STRUCTURE_NOTIFY
}

#[inline]
pub fn button_event_mask() -> xproto::EventMask {
    xproto::EventMask::BUTTON_PRESS
        | xproto::EventMask::BUTTON_RELEASE
        | xproto::EventMask::BUTTON_MOTION
}

#[inline]
pub fn mouse_event_mask() -> xproto::EventMask {
    button_event_mask() | xproto::EventMask::POINTER_MOTION
}

pub struct Colors {
    normal: u32,
    floating: u32,
    active: u32,
    background: u32,
}

/// Contains Xserver information and origins.
pub struct XWrap {
    conn: RustConnection,
    display: usize,
    root: xproto::Window,
    cursors: CursorHandle,
    pub atoms: AtomCollection,

    colors: Colors,
    pub managed_windows: Vec<xproto::Window>,
    pub focused_window: xproto::Window,
    pub tag_labels: Vec<String>,
    pub mode: Mode,
    pub focus_behaviour: FocusBehaviour,
    pub mouse_key_mask: ModMask,
    pub mode_origin: (i32, i32),

    pub motion_event_limiter: u32,
    pub refresh_rate: u32,
}

impl XWrap {
    pub fn new() -> Self {
        // const SERVER: mio::Token = mio::Token(0);
        let (conn, display) = x11rb::connect(None).expect("Couldn't not connect to Xorg Server");

        // let fd = unsafe { (xlib.XConnectionNumber)(display) };

        // let (guard, _task_guard) = oneshot::channel();
        // let notify = Arc::new(Notify::new());
        // let task_notify = notify.clone();

        //TODO: Figure out what this is doing
        // let mut poll = mio::Poll::new().expect("Unable to boot Mio");
        // let mut events = mio::Events::with_capacity(1);
        // poll.registry()
        //     .register(
        //         &mut mio::unix::SourceFd(&fd),
        //         SERVER,
        //         mio::Interest::READABLE,
        //     )
        //     .expect("Unable to boot Mio");
        // let timeout = Duration::from_millis(100);
        // tokio::task::spawn_blocking(move || loop {
        //     if guard.is_closed() {
        //         return;
        //     }
        //
        //     if let Err(err) = poll.poll(&mut events, Some(timeout)) {
        //         tracing::warn!("Xlib socket poll failed with {:?}", err);
        //         continue;
        //     }
        //
        //     events
        //         .iter()
        //         .filter(|event| SERVER == event.token())
        //         .for_each(|_| notify.notify_one());
        // });

        let atoms = AtomCollection::new(&conn)
            .expect("Unable to load atoms")
            .reply()
            .unwrap();
        let root = &conn.setup().roots[display];
        let root_handle = root.root;
        let mut req = Database::GET_RESOURCE_DATABASE;
        req.window = root_handle;

        let (bytes, fd) = req.serialize();
        let slice = &[IoSlice::new(&bytes[0])];
        let res = conn
            .send_request_with_reply(slice, fd)
            .expect("Unable to request resource database status");
        let reply: xproto::GetPropertyReply = res.reply().unwrap();
        let db = Database::new_from_default(&reply, "localhost".into());
        let cursors = CursorHandle::new(&conn, display, &db)
            .expect("Unable to get cursors")
            .reply()
            .unwrap();

        let colors = Colors {
            normal: 0,
            floating: 0,
            active: 0,
            background: 0,
        };

        fn get_refresh_rate(
            conn: &RustConnection,
            root: xproto::Window,
        ) -> Result<u32, Box<dyn std::error::Error>> {
            let screen_resources =
                <RustConnection as RandrConnectionExtention>::randr_get_screen_resources(
                    &conn, root,
                )?
                .reply()?;
            // RandrConnectionExtention::randr_get_screen_resources(&conn, root)?.reply()?;
            let active_modes: Vec<u32> = screen_resources
                .crtcs
                .iter()
                .map(|crtc| {
                    <RustConnection as RandrConnectionExtention>::randr_get_crtc_info(
                        &conn,
                        *crtc,
                        screen_resources.config_timestamp,
                    )
                })
                .filter_map(Result::ok)
                .map(|res| res.reply())
                .filter_map(Result::ok)
                .filter(|crtc| crtc.mode != 0)
                .map(|crtc_info| crtc_info.mode)
                .collect();
            Ok(screen_resources
                .modes
                .iter()
                .filter(|mode_info| active_modes.contains(&mode_info.id))
                .map(|mode_info| mode_info.dot_clock / (mode_info.htotal * mode_info.vtotal) as u32)
                .max()
                .unwrap_or(60))
        }
        let refresh_rate = match get_refresh_rate(&conn, root.root) {
            Ok(r) => r,
            Err(_) => 60,
        };
        tracing::debug!("Refresh Rate: {}", refresh_rate);

        let xw = Self {
            conn,
            display,
            root: root_handle,
            cursors,
            atoms,

            colors,
            managed_windows: vec![],
            focused_window: root_handle,
            tag_labels: vec![],
            mode: Mode::Normal,
            focus_behaviour: FocusBehaviour::Sloppy,
            mouse_key_mask: 0,
            mode_origin: (0, 0),

            motion_event_limiter: 0,
            refresh_rate,
        };

        //TODO: Do we need to check if another WM is running ?
        xproto::change_window_attributes(
            &xw.conn,
            xw.root,
            &xproto::ChangeWindowAttributesAux::new()
                .event_mask(xproto::EventMask::PROPERTY_CHANGE),
        )
        .unwrap();
        xw.sync();

        xw
    }

    pub fn load_config(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle>>,
        windows: &[Window],
    ) {
        self.focus_behaviour = config.focus_behaviour();
        self.mouse_key_mask = utils::modmask_lookup::into_modmask(&config.mousekey());
        self.load_colors(config, focused, Some(windows));
        self.tag_labels = config.create_list_of_tag_labels();
    }

    /// Load the colors of our theme.
    pub fn load_colors(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle>>,
        windows: Option<&[Window]>,
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
                if let WindowHandle::X11rbHandle(handle) = window.handle {
                    let is_focused =
                        matches!(focused, Some(&Some(focused)) if focused == window.handle);
                    let color: u32 = if is_focused {
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

    pub fn init(&mut self, config: &impl Config) {
        self.focus_behaviour = config.focus_behaviour();
        self.mouse_key_mask = utils::modmask_lookup::into_modmask(&config.mousekey());

        let root = self.root;
        self.load_colors(config, None, None);

        let cursor = self.cursors.load_cursor(&self.conn, "normal").unwrap();
        xproto::change_window_attributes(
            &self.conn,
            root,
            &ChangeWindowAttributesAux::new()
                .cursor(cursor)
                .event_mask(root_event_mask()),
        )
        .unwrap();

        // EWMH compliance.
        let supported: Vec<xproto::Atom> = self.atoms.net_supported();
        self.replace_property_u32(
            root,
            self.atoms.NetSupported,
            xproto::AtomEnum::ATOM.into(),
            &supported,
        );
        xproto::delete_property(&self.conn, root, self.atoms.NetClientList).unwrap();

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
        self.set_desktop_prop(&[tag_length as u32], self.atoms.NetNumberOfDesktops);

        // Set a current desktop.
        self.set_desktop_prop(&[0_u32, x11rb::CURRENT_TIME], self.atoms.NetCurrentDesktop);

        // Set desktop names.
        //
        // Convert the list of tag names string into a valid list of strings for an atom,
        // which is a null terminated string containing null terminated strings for each value.
        // This essecially replicates what this function does:
        // `Xutf8TextListToTextProperty`: https://linux.die.net/man/3/xutf8textlisttotextproperty
        let concat_str = tag_labels
            .iter()
            .fold(String::default(), |acc, x| format!("{}{}\0", acc, x));
        let bytes = concat_str.as_bytes();

        xproto::change_property(
            &self.conn,
            xproto::PropMode::REPLACE,
            self.root,
            self.atoms.NetDesktopNames,
            self.atoms.UTF8String,
            8,
            // Removing the last null byte because `CString::from_vec_unchecked` adds a trailing
            // null byte
            bytes.len() as u32 - 1,
            &bytes[..bytes.len() - 1],
        )
        .unwrap();

        // Set the WM NAME.
        self.set_desktop_prop_string("LeftWM", self.atoms.NetWMName, self.atoms.UTF8String);

        self.set_desktop_prop_string(
            "LeftWM",
            self.atoms.WMClass,
            xproto::AtomEnum::STRING.into(),
        );

        self.set_desktop_prop_u32(
            self.root,
            self.atoms.NetSupportingWmCheck,
            xproto::AtomEnum::STRING.into(),
        );

        // Set a viewport.
        self.set_desktop_prop(&[0_u32, 0_u32], self.atoms.NetDesktopViewport);
    }

    /// Send a xevent atom for a window to X.
    // `XSendEvent`: https://tronche.com/gui/x/xlib/event-handling/XSendEvent.html
    fn send_xevent_atom(&self, window: xproto::Window, atom: xproto::Atom) -> bool {
        if self.can_send_xevent_atom(window, atom) {
            let mut msg: xproto::ClientMessageEvent = unsafe { std::mem::zeroed() };
            msg.type_ = self.atoms.WMProtocols;
            msg.window = window;
            msg.format = 32;

            let mut data = [0u32; 5];
            data[0] = atom;
            data[1] = x11rb::CURRENT_TIME;
            msg.data = data.into();

            self.send_xevent(window, false, xproto::EventMask::NO_EVENT, &msg.serialize());
            return true;
        }
        false
    }

    /// Send a xevent for a window to X.
    // `XSendEvent`: https://tronche.com/gui/x/xlib/event-handling/XSendEvent.html
    pub fn send_xevent(
        &self,
        window: xproto::Window,
        propagate: bool,
        mask: xproto::EventMask,
        event: &[u8],
    ) {
        let mut data = [0u8; 32];
        for i in 0..event.len() {
            data[i] = event[i];
        }
        xproto::send_event(&self.conn, propagate, window, mask, data).unwrap();
        self.sync();
    }

    /// Returns whether a window can recieve a xevent atom.
    // `XGetWMProtocols`: https://tronche.com/gui/x/xlib/ICC/client-to-window-manager/XGetWMProtocols.html
    fn can_send_xevent_atom(&self, window: xproto::Window, atom: xproto::Atom) -> bool {
        let Ok(cookie) = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.WMProtocols,
            xproto::AtomEnum::ATOM,
            0,
            MAX_PROPERTY_VALUE_LEN / 4,
        ) else {
            return false;
        };

        let Ok(res) = cookie.reply() else {
            return false;
        };

        let Some(values) = res.value32() else {
            return false;
        };

        values.collect::<Vec<xproto::Atom>>().contains(&atom)
    }

    /// Sets the mode within our xwrapper.
    pub fn set_mode(&mut self, mode: Mode) {
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
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => {
                        self.cursors.load_cursor(&self.conn, "resize").unwrap()
                    }
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => {
                        self.cursors.load_cursor(&self.conn, "move").unwrap()
                    }
                    Mode::Normal => self.cursors.load_cursor(&self.conn, "normal").unwrap(),
                };
                self.grab_pointer(cursor);
            }
            Mode::MovingWindow(h) | Mode::ResizingWindow(h)
                if self.mode == Mode::ReadyToMove(h) || self.mode == Mode::ReadyToResize(h) =>
            {
                self.ungrab_pointer();
                self.mode = mode;
                let cursor = match mode {
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => {
                        self.cursors.load_cursor(&self.conn, "resize").unwrap()
                    }
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => {
                        self.cursors.load_cursor(&self.conn, "move").unwrap()
                    }
                    Mode::Normal => self.cursors.load_cursor(&self.conn, "normal").unwrap(),
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
    // pub async fn wait_readable(&mut self) {
    //     self.task_notify.notified().await;
    // }

    /// Flush and sync the xserver.
    pub fn sync(&self) {
        <RustConnection as x11rb::wrapper::ConnectionExt>::sync(&self.conn).unwrap();
    }

    /// Flush the xserver.
    // `XFlush`: https://tronche.com/gui/x/xlib/event-handling/XFlush.html
    pub fn flush(&self) {
        self.conn.flush().unwrap();
    }

    // /// Returns how many events are waiting.
    // // `XPending`: https://tronche.com/gui/x/xlib/event-handling/XPending.html
    // #[must_use]
    // pub fn queue_len(&self) -> i32 {
    //     unsafe { (self.xlib.XPending)(self.display) }
    // }
}
