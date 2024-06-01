use std::{io::IoSlice, os::fd::AsRawFd, sync::Arc, time::Duration};

use leftwm_core::{
    models::{FocusBehaviour, WindowHandle},
    utils::{self, modmask_lookup::ModMask},
    Config, Mode, Window,
};
use tokio::sync::{oneshot, Notify};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{
        randr,
        xproto::{self, ChangeWindowAttributesAux},
    },
    resource_manager::Database,
    rust_connection::RustConnection,
    wrapper::ConnectionExt,
    x11_utils::Serialize,
};

use crate::{error::ErrorKind, xatom::AtomCollection, xcursors::XCursor, X11rbWindowHandle};

use crate::error::Result;

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

/// IDs of colors used across `LeftWM`
pub struct Colors {
    normal: u32,
    floating: u32,
    active: u32,
    background: u32,
}

/// Contains Xserver information and origins.
pub(crate) struct XWrap {
    conn: RustConnection,
    display: usize,
    root: xproto::Window,
    cursors: XCursor,
    pub atoms: AtomCollection,

    colors: Colors,
    pub managed_windows: Vec<xproto::Window>,
    pub focused_window: xproto::Window,
    pub tag_labels: Vec<String>,
    pub mode: Mode<X11rbWindowHandle>,
    pub focus_behaviour: FocusBehaviour,
    pub mouse_key_mask: ModMask,
    pub mode_origin: (i32, i32),

    #[allow(unused)]
    task_guard: oneshot::Receiver<()>,
    pub task_notify: Arc<Notify>,
    pub motion_event_limiter: u32,
    pub refresh_rate: u32,
}

impl XWrap {
    pub fn new() -> Self {
        const SERVER: mio::Token = mio::Token(0);
        let (conn, display) = x11rb::connect(None).expect("Couldn't not connect to Xorg Server");

        let fd = conn.stream().as_raw_fd();

        let (guard, task_guard) = oneshot::channel::<()>();
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
                tracing::info!("x11rb socket closed");
                return;
            }

            if let Err(err) = poll.poll(&mut events, Some(timeout)) {
                tracing::warn!("x11rb socket poll failed with {:?}", err);
                continue;
            }

            events
                .iter()
                .filter(|event| SERVER == event.token())
                .for_each(|_| notify.notify_one());
        });

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
        let reply: xproto::GetPropertyReply = conn
            .send_request_with_reply(slice, fd)
            .expect("Unable to request resource database status")
            .reply()
            .expect("Parsing reply failed.");
        let db = Database::new_from_default(&reply, "localhost".into());
        let cursors = XCursor::new(&conn, display, &db).expect("Unable to load cursors");

        let colors = Colors {
            normal: 0,
            floating: 0,
            active: 0,
            background: 0,
        };

        let refresh_rate = get_refresh_rate(&conn, root.root).unwrap_or(60);
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
            mouse_key_mask: ModMask::Zero,
            mode_origin: (0, 0),

            task_guard,
            task_notify,
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
        xw.sync().expect("Unable to sync the connection");

        xw
    }

    pub fn load_config(&mut self, config: &impl Config) -> Result<()> {
        self.focus_behaviour = config.focus_behaviour();
        self.mouse_key_mask = utils::modmask_lookup::into_modmask(&config.mousekey());
        self.tag_labels = config.create_list_of_tag_labels();
        self.colors = Colors {
            normal: self.get_color(&config.default_border_color())?,
            floating: self.get_color(&config.floating_border_color())?,
            active: self.get_color(&config.focused_border_color())?,
            background: self.get_color(&config.background_color())?,
        };
        Ok(())
    }

    /// Load the colors of our theme.
    pub fn update_colors(
        &mut self,
        focused: Option<WindowHandle<X11rbWindowHandle>>,
        windows: &[Window<X11rbWindowHandle>],
    ) -> Result<()> {
        // Update all the windows with the new colors.
        for window in windows {
            let WindowHandle(X11rbWindowHandle(handle)) = window.handle;
            let color: u32 = if focused == Some(window.handle) {
                self.colors.active
            } else if window.floating() {
                self.colors.floating
            } else {
                self.colors.normal
            };
            self.set_window_border_color(handle, color)?;
        }
        self.set_background_color(self.colors.background)?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        let root = self.root;

        xproto::change_window_attributes(
            &self.conn,
            root,
            &ChangeWindowAttributesAux::new()
                .cursor(self.cursors.normal)
                .event_mask(root_event_mask()),
        )?;

        // EWMH compliance.
        let supported: Vec<xproto::Atom> = self.atoms.net_supported();
        self.replace_property_u32(
            root,
            self.atoms.NetSupported,
            xproto::AtomEnum::ATOM.into(),
            &supported,
        )?;
        xproto::delete_property(&self.conn, root, self.atoms.NetClientList)?;

        // EWMH compliance for desktops.
        self.init_desktops_hints()?;

        self.sync()?;
        Ok(())
    }

    /// EWMH support used for bars such as polybar.
    pub fn init_desktops_hints(&self) -> Result<()> {
        let tag_labels = &self.tag_labels;
        let tag_length = tag_labels.len();

        // Set the number of desktop.
        self.set_desktop_prop(
            &[u32::try_from(tag_length)?],
            self.atoms.NetNumberOfDesktops,
        )?;

        // Set a current desktop.
        self.set_desktop_prop(&[0_u32, x11rb::CURRENT_TIME], self.atoms.NetCurrentDesktop)?;

        // Set desktop names.
        //
        // Convert the list of tag names string into a valid list of strings for an atom,
        // which is a null terminated string containing null terminated strings for each value.
        // This essecially replicates what this function does:
        // `Xutf8TextListToTextProperty`: https://linux.die.net/man/3/xutf8textlisttotextproperty
        let concat_str = tag_labels
            .iter()
            .fold(String::default(), |acc, x| format!("{acc}{x}\0"));
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
            u32::try_from(bytes.len())? - 1,
            &bytes[..bytes.len() - 1],
        )?;

        // Set the WM NAME.
        self.set_desktop_prop_string("LeftWM", self.atoms.NetWMName, self.atoms.UTF8String)?;

        self.set_desktop_prop_string(
            "LeftWM",
            self.atoms.WMClass,
            xproto::AtomEnum::STRING.into(),
        )?;

        self.set_desktop_prop_u32(
            self.root,
            self.atoms.NetSupportingWmCheck,
            xproto::AtomEnum::WINDOW.into(),
        )?;

        // Set a viewport.
        self.set_desktop_prop(&[0_u32, 0_u32], self.atoms.NetDesktopViewport)?;
        Ok(())
    }

    /// Send a xevent atom for a window to X.
    fn send_xevent_atom(&self, window: xproto::Window, atom: xproto::Atom) -> Result<bool> {
        if self.can_send_xevent_atom(window, atom)? {
            let mut msg: xproto::ClientMessageEvent = unsafe { std::mem::zeroed() };
            msg.response_type = xproto::CLIENT_MESSAGE_EVENT;
            msg.type_ = self.atoms.WMProtocols;
            msg.window = window;
            msg.format = 32;

            let mut data = [0u32; 5];
            data[0] = atom;
            data[1] = x11rb::CURRENT_TIME;
            msg.data = data.into();

            self.send_xevent(window, false, xproto::EventMask::NO_EVENT, &msg.serialize())?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Send a xevent for a window to X.
    pub fn send_xevent(
        &self,
        window: xproto::Window,
        propagate: bool,
        mask: xproto::EventMask,
        event: &[u8],
    ) -> Result<()> {
        let mut data = [0u8; 32];
        data[..event.len()].copy_from_slice(event);
        xproto::send_event(&self.conn, propagate, window, mask, data)?;
        self.sync()?;
        Ok(())
    }

    /// Returns whether a window can recieve a xevent atom.
    fn can_send_xevent_atom(&self, window: xproto::Window, atom: xproto::Atom) -> Result<bool> {
        let reply = xproto::get_property(
            &self.conn,
            false,
            window,
            self.atoms.WMProtocols,
            xproto::AtomEnum::ATOM,
            0,
            MAX_PROPERTY_VALUE_LEN / 4,
        )?
        .reply()?;

        Ok(reply
            .value32()
            .is_some_and(|v| v.collect::<Vec<xproto::Atom>>().contains(&atom)))
    }

    /// Sets the mode within our xwrapper.
    pub fn set_mode(&mut self, mode: Mode<X11rbWindowHandle>) -> Result<()> {
        match mode {
            // Prevent resizing and moving of root.
            Mode::MovingWindow(h)
            | Mode::ResizingWindow(h)
            | Mode::ReadyToMove(h)
            | Mode::ReadyToResize(h)
                if h == self.get_default_root_handle() => {}
            Mode::ReadyToMove(_) | Mode::ReadyToResize(_) if self.mode == Mode::Normal => {
                self.mode = mode;
                match self.get_cursor_point() {
                    Ok(loc) => self.mode_origin = loc,
                    Err(e) if e.kind == ErrorKind::RootWindowNotFound => {
                        return Err(e);
                    }
                    _ => (),
                }
                let cursor = match mode {
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => self.cursors.resize,
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => self.cursors.move_,
                    Mode::Normal => self.cursors.normal,
                };
                self.grab_pointer(cursor)?;
            }
            Mode::MovingWindow(h) | Mode::ResizingWindow(h)
                if self.mode == Mode::ReadyToMove(h) || self.mode == Mode::ReadyToResize(h) =>
            {
                self.ungrab_pointer()?;
                self.mode = mode;
                let cursor = match mode {
                    Mode::ReadyToResize(_) | Mode::ResizingWindow(_) => self.cursors.resize,
                    Mode::ReadyToMove(_) | Mode::MovingWindow(_) => self.cursors.move_,
                    Mode::Normal => self.cursors.normal,
                };
                self.grab_pointer(cursor)?;
            }
            Mode::Normal => {
                self.ungrab_pointer()?;
                self.mode = mode;
            }
            _ => {}
        };
        Ok(())
    }

    /// Flush and sync the xserver.
    pub fn sync(&self) -> Result<()> {
        self.conn.sync()?;
        Ok(())
    }

    /// Flush the xserver.
    pub fn flush(&self) -> Result<()> {
        self.conn.flush()?;
        Ok(())
    }
}

fn get_refresh_rate(conn: &RustConnection, root: xproto::Window) -> Result<u32> {
    let screen_resources = randr::get_screen_resources(conn, root)?.reply()?;
    let active_modes: Vec<u32> = screen_resources
        .crtcs
        .iter()
        .map(|crtc| randr::get_crtc_info(conn, *crtc, screen_resources.config_timestamp))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(x11rb::cookie::Cookie::reply)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|crtc_info| crtc_info.mode)
        .collect();

    Ok(screen_resources
        .modes
        .iter()
        .filter(|mode_info| active_modes.contains(&mode_info.id))
        .map(|mode_info| {
            mode_info.dot_clock / (u32::from(mode_info.htotal) * u32::from(mode_info.vtotal))
        })
        .max()
        .unwrap_or(60))
}
