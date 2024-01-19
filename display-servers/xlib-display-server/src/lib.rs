// allow casting types
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

mod event_translate;
mod event_translate_client_message;
mod event_translate_property_notify;
mod xatom;
mod xcursor;
mod xwrap;

use serde::{Serialize, Deserialize};
pub use xwrap::XWrap;

use self::xwrap::ICONIC_STATE;
use event_translate::XEvent;
use futures::prelude::*;
use leftwm_core::config::Config;
use leftwm_core::models::{
    Handle, Mode, Screen, TagId, Window, WindowHandle, WindowState, Workspace,
};
use leftwm_core::utils;
use leftwm_core::{DisplayAction, DisplayEvent, DisplayServer};
use std::os::raw::c_uint;
use std::pin::Pin;

use x11_dl::xlib;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct XlibWindowHandle(xlib::Window);
impl Handle for XlibWindowHandle {}

pub struct XlibDisplayServer {
    xw: XWrap,
    root: xlib::Window,
    initial_events: Vec<DisplayEvent<XlibWindowHandle>>,
}

impl DisplayServer<XlibWindowHandle> for XlibDisplayServer {
    fn new(config: &impl Config) -> Self {
        let mut wrap = XWrap::new();

        wrap.init(config); // setup events masks

        let root = wrap.get_default_root();
        let instance = Self {
            xw: wrap,
            root,
            initial_events: Vec::new(),
        };
        let initial_events = instance.initial_events(config);

        Self {
            initial_events,
            ..instance
        }
    }

    fn load_config(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle<XlibWindowHandle>>>,
        windows: &[Window<XlibWindowHandle>],
    ) {
        self.xw.load_config(config, focused, windows);
    }

    fn update_windows(&self, windows: Vec<&Window<XlibWindowHandle>>) {
        for window in &windows {
            self.xw.update_window(window);
        }
    }

    fn update_workspaces(&self, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            self.xw.set_current_desktop(focused.tag);
        }
    }

    fn get_next_events(&mut self) -> Vec<DisplayEvent<XlibWindowHandle>> {
        let mut events = std::mem::take(&mut self.initial_events);

        let events_in_queue = self.xw.queue_len();
        for _ in 0..events_in_queue {
            let xlib_event = self.xw.get_next_event();
            let event = XEvent(&mut self.xw, xlib_event).into();
            if let Some(e) = event {
                tracing::trace!("DisplayEvent: {:?}", e);
                events.push(e);
            }
        }

        for event in &events {
            if let DisplayEvent::WindowDestroy(WindowHandle(XlibWindowHandle(w))) = event {
                self.xw.force_unmapped(*w);
            }
        }

        events
    }

    fn execute_action(
        &mut self,
        act: DisplayAction<XlibWindowHandle>,
    ) -> Option<DisplayEvent<XlibWindowHandle>> {
        tracing::trace!("DisplayAction: {:?}", act);
        let xw = &mut self.xw;
        let event: Option<DisplayEvent<XlibWindowHandle>> = match act {
            DisplayAction::KillWindow(h) => from_kill_window(xw, h),
            DisplayAction::AddedWindow(h, f, fm) => from_added_window(xw, h, f, fm),
            DisplayAction::MoveMouseOver(h, f) => from_move_mouse_over(xw, h, f),
            DisplayAction::MoveMouseOverPoint(p) => from_move_mouse_over_point(xw, p),
            DisplayAction::DestroyedWindow(h) => from_destroyed_window(xw, h),
            DisplayAction::Unfocus(h, f) => from_unfocus(xw, h, f),
            DisplayAction::ReplayClick(h, b) => from_replay_click(xw, h, b.bits().into()),
            DisplayAction::SetState(h, t, s) => from_set_state(xw, h, t, s),
            DisplayAction::SetWindowOrder(ws) => from_set_window_order(xw, ws),
            DisplayAction::MoveToTop(h) => from_move_to_top(xw, h),
            DisplayAction::ReadyToMoveWindow(h) => from_ready_to_move_window(xw, h),
            DisplayAction::ReadyToResizeWindow(h) => from_ready_to_resize_window(xw, h),
            DisplayAction::SetCurrentTags(t) => from_set_current_tags(xw, t),
            DisplayAction::SetWindowTag(h, t) => from_set_window_tag(xw, h, t),
            DisplayAction::ConfigureXlibWindow(w) => from_configure_xlib_window(xw, &w),

            DisplayAction::WindowTakeFocus {
                window,
                previous_window,
            } => from_window_take_focus(xw, &window, &previous_window),

            DisplayAction::FocusWindowUnderCursor => from_focus_window_under_cursor(xw),
            DisplayAction::NormalMode => from_normal_mode(xw),
        };
        if event.is_some() {
            tracing::trace!("DisplayEvent: {:?}", event);
        }
        event
    }

    fn wait_readable(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        let task_notify = self.xw.task_notify.clone();
        Box::pin(async move {
            task_notify.notified().await;
        })
    }

    fn flush(&self) {
        self.xw.flush();
    }

    /// Creates a verify focus event for the cursors current window.
    fn generate_verify_focus_event(&self) -> Option<DisplayEvent<XlibWindowHandle>> {
        let handle = self.xw.get_cursor_window().ok()?;
        Some(DisplayEvent::VerifyFocusedAt(handle))
    }
}

impl XlibDisplayServer {
    /// Return a vec of events for setting up state of WM.
    fn initial_events(&self, config: &impl Config) -> Vec<DisplayEvent<XlibWindowHandle>> {
        let mut events = vec![];
        if let Some(workspaces) = config.workspaces() {
            let screens = self.xw.get_screens();
            for (i, wsc) in workspaces.iter().enumerate() {
                let mut screen = Screen::from(wsc);
                screen.root = WindowHandle(XlibWindowHandle(self.root));
                // If there is a screen corresponding to the given output, create the workspace
                match screens.iter().find(|i| i.output == wsc.output) {
                    Some(output_match) => {
                        if wsc.relative.unwrap_or(false) {
                            screen.bbox.add(output_match.bbox);
                        }
                        screen.id = Some(i + 1);
                    }
                    None => continue,
                }
                let e = DisplayEvent::ScreenCreate(screen);
                events.push(e);
            }

            let auto_derive_workspaces: bool = if config.auto_derive_workspaces() {
                true
            } else if events.is_empty() {
                tracing::warn!("No Workspace in Workspace config matches connected screen. Falling back to \"auto_derive_workspaces: true\".");
                true
            } else {
                false
            };

            let mut next_id = workspaces.len() + 1;

            // If there is no hardcoded workspace layout, add every screen not mentioned in the config.
            if auto_derive_workspaces {
                screens
                    .iter()
                    .filter(|screen| !workspaces.iter().any(|wsc| wsc.output == screen.output))
                    .for_each(|screen| {
                        let mut s = screen.clone();
                        s.id = Some(next_id);
                        next_id += 1;
                        events.push(DisplayEvent::ScreenCreate(s));
                    });
            }
        }

        // Tell manager about existing windows.
        events.append(&mut self.find_all_windows());

        events
    }

    fn find_all_windows(&self) -> Vec<DisplayEvent<XlibWindowHandle>> {
        let mut all: Vec<DisplayEvent<XlibWindowHandle>> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => handles.into_iter().for_each(|handle| {
                let Ok(attrs) = self.xw.get_window_attrs(handle) else {
                    return;
                };
                let Some(state) = self.xw.get_wm_state(handle) else {
                    return;
                };
                if attrs.map_state == xlib::IsViewable || state == ICONIC_STATE {
                    if let Some(event) = self.xw.setup_window(handle) {
                        all.push(event);
                    }
                }
            }),
            Err(err) => {
                println!("ERROR: {err}");
            }
        }
        all
    }
}

// Display actions.
fn from_kill_window(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.kill_window(&handle);
    None
}

fn from_added_window(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
    floating: bool,
    follow_mouse: bool,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.setup_managed_window(handle, floating, follow_mouse)
}

fn from_move_mouse_over(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
    force: bool,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    let WindowHandle(XlibWindowHandle(window)) = handle;
    match xw.get_cursor_window() {
        Ok(WindowHandle(XlibWindowHandle(cursor_window))) if force || cursor_window != window => {
            _ = xw.move_cursor_to_window(window);
        }
        _ => {}
    }
    None
}

fn from_move_mouse_over_point(
    xw: &mut XWrap,
    point: (i32, i32),
) -> Option<DisplayEvent<XlibWindowHandle>> {
    _ = xw.move_cursor_to_point(point);
    None
}

fn from_destroyed_window(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.teardown_managed_window(&handle, true);
    None
}

fn from_unfocus(
    xw: &mut XWrap,
    handle: Option<WindowHandle<XlibWindowHandle>>,
    floating: bool,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.unfocus(handle, floating);
    None
}

fn from_replay_click(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
    button: c_uint,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    if let WindowHandle(XlibWindowHandle(handle)) = handle {
        xw.replay_click(handle, button);
    }
    None
}

fn from_set_state(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
    toggle_to: bool,
    window_state: WindowState,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    // TODO: impl from for windowstate and xlib::Atom
    let state = match window_state {
        WindowState::Modal => xw.atoms.NetWMStateModal,
        WindowState::Sticky => xw.atoms.NetWMStateSticky,
        WindowState::MaximizedVert => xw.atoms.NetWMStateMaximizedVert,
        WindowState::MaximizedHorz => xw.atoms.NetWMStateMaximizedHorz,
        WindowState::Shaded => xw.atoms.NetWMStateShaded,
        WindowState::SkipTaskbar => xw.atoms.NetWMStateSkipTaskbar,
        WindowState::SkipPager => xw.atoms.NetWMStateSkipPager,
        WindowState::Hidden => xw.atoms.NetWMStateHidden,
        WindowState::Fullscreen => xw.atoms.NetWMStateFullscreen,
        WindowState::Above => xw.atoms.NetWMStateAbove,
        WindowState::Below => xw.atoms.NetWMStateBelow,
        WindowState::Maximized => {
            xw.set_state(handle, toggle_to, xw.atoms.NetWMStateMaximizedVert);
            xw.set_state(handle, toggle_to, xw.atoms.NetWMStateMaximizedHorz);
            return None;
        }
    };
    xw.set_state(handle, toggle_to, state);
    None
}

fn from_set_window_order(
    xw: &mut XWrap,
    windows: Vec<WindowHandle<XlibWindowHandle>>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    // Unmanaged windows.
    let unmanaged: Vec<WindowHandle<XlibWindowHandle>> = xw
        .get_all_windows()
        .unwrap_or_default()
        .iter()
        .filter(|&w| *w != xw.get_default_root())
        .map(|&w| WindowHandle(XlibWindowHandle(w)))
        .filter(|h| !windows.iter().any(|w| w == h))
        .collect();
    // Unmanaged windows on top.
    xw.restack([unmanaged, windows].concat());
    None
}

fn from_move_to_top(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.move_to_top(&handle);
    None
}

fn from_ready_to_move_window(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.set_mode(Mode::ReadyToMove(handle));
    None
}

fn from_ready_to_resize_window(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.set_mode(Mode::ReadyToResize(handle));
    None
}

fn from_set_current_tags(
    xw: &mut XWrap,
    tag: Option<TagId>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.set_current_desktop(tag);
    None
}

fn from_set_window_tag(
    xw: &mut XWrap,
    handle: WindowHandle<XlibWindowHandle>,
    tag: Option<TagId>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    let WindowHandle(XlibWindowHandle(window)) = handle;
    let tag = tag?;
    xw.set_window_desktop(window, &tag);
    None
}

fn from_configure_xlib_window(
    xw: &mut XWrap,
    window: &Window<XlibWindowHandle>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.configure_window(window);
    None
}

fn from_window_take_focus(
    xw: &mut XWrap,
    window: &Window<XlibWindowHandle>,
    previous_window: &Option<Window<XlibWindowHandle>>,
) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.window_take_focus(window, previous_window.as_ref());
    None
}

fn from_focus_window_under_cursor(xw: &mut XWrap) -> Option<DisplayEvent<XlibWindowHandle>> {
    if let Ok(mut window) = xw.get_cursor_window() {
        if window == WindowHandle(XlibWindowHandle(0)) {
            window = xw.get_default_root_handle();
        }
        return Some(DisplayEvent::WindowTakeFocus(window));
    }
    let point = xw.get_cursor_point().ok()?;
    let evt = DisplayEvent::MoveFocusTo(point.0, point.1);
    Some(evt)
}

fn from_normal_mode(xw: &mut XWrap) -> Option<DisplayEvent<XlibWindowHandle>> {
    xw.set_mode(Mode::Normal);
    None
}
