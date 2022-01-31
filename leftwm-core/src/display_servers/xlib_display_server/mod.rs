use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::TagId;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::WindowState;
use crate::models::Workspace;
use crate::utils;
use crate::DisplayEvent;
use crate::DisplayServer;
use crate::Keybind;
use futures::prelude::*;
use std::pin::Pin;
use x11_dl::xlib;

mod event_translate;
mod event_translate_client_message;
mod event_translate_property_notify;
mod xatom;
mod xwrap;
pub use xwrap::XWrap;

use event_translate::XEvent;

use self::xwrap::ICONIC_STATE;
mod xcursor;

pub struct XlibDisplayServer {
    xw: XWrap,
    root: xlib::Window,
    initial_events: Option<Vec<DisplayEvent>>,
}

impl DisplayServer for XlibDisplayServer {
    fn new(config: &impl Config) -> Self {
        let mut wrap = XWrap::new();

        wrap.init(config); //setup events masks

        let root = wrap.get_default_root();
        let instance = Self {
            xw: wrap,
            root,
            initial_events: None,
        };
        let initial_events = instance.initial_events(config);

        Self {
            initial_events: Some(initial_events),
            ..instance
        }
    }

    fn load_config(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle>>,
        windows: &[Window],
    ) {
        self.xw.load_config(config, focused, windows);
    }

    fn update_windows(&self, windows: Vec<&Window>) {
        for window in &windows {
            self.xw.update_window(window);
        }
    }

    fn update_workspaces(&self, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            self.xw.set_current_desktop(&focused.tags);
        }
    }

    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        let mut events = vec![];

        if let Some(initial_events) = self.initial_events.take() {
            for e in initial_events {
                (&mut events).push(e);
            }
        }

        let events_in_queue = self.xw.queue_len();

        for _ in 0..events_in_queue {
            let xlib_event = self.xw.get_next_event();
            let event = XEvent(&mut self.xw, xlib_event).into();
            if let Some(e) = event {
                log::trace!("DisplayEvent: {:?}", e);
                events.push(e);
            }
        }

        for event in &events {
            if let DisplayEvent::WindowDestroy(WindowHandle::XlibHandle(w)) = event {
                self.xw.force_unmapped(*w);
            }
        }

        events
    }

    fn execute_action(&mut self, act: DisplayAction) -> Option<DisplayEvent> {
        log::trace!("DisplayAction: {:?}", act);
        let event: Option<DisplayEvent> = match act {
            DisplayAction::KillWindow(h) => self.from_kill_window(h),
            DisplayAction::AddedWindow(h, fl, fm) => self.from_added_window(h, fl, fm),
            DisplayAction::MoveMouseOver(h) => self.from_move_mouse_over(h),
            DisplayAction::MoveMouseOverPoint(p) => self.from_move_mouse_over_point(p),
            DisplayAction::DestroyedWindow(h) => self.from_destroyed_window(h),
            DisplayAction::Unfocus(h) => self.from_unfocus(h),
            DisplayAction::SetState(h, t, s) => self.from_set_state(h, t, s),
            DisplayAction::SetWindowOrder(ws) => self.from_set_window_order(&ws),
            DisplayAction::MoveToTop(h) => self.from_move_to_top(h),
            DisplayAction::ReadyToMoveWindow(h) => self.from_ready_to_move_window(h),
            DisplayAction::ReadyToResizeWindow(h) => self.from_ready_to_resize_window(h),
            DisplayAction::SetCurrentTags(ts) => self.from_set_current_tags(&ts),
            DisplayAction::SetWindowTags(h, ts) => self.from_set_window_tags(h, &ts),
            DisplayAction::ReloadKeyGrabs(ks) => self.from_reload_key_grabs(&ks),
            DisplayAction::ConfigureXlibWindow(w) => self.from_configure_xlib_window(w),

            DisplayAction::WindowTakeFocus {
                window,
                previous_window,
            } => self.from_window_take_focus(window, previous_window),

            DisplayAction::FocusWindowUnderCursor => self.from_focus_window_under_cursor(),
            DisplayAction::NormalMode => self.from_normal_mode(),
        };
        if event.is_some() {
            log::trace!("DisplayEvent: {:?}", event);
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
    fn generate_verify_focus_event(&self) -> Option<DisplayEvent> {
        let handle = self.xw.get_cursor_window().ok()?;
        Some(DisplayEvent::VerifyFocusedAt(handle))
    }
}

impl XlibDisplayServer {
    /// Return a vec of events for setting up state of WM.
    fn initial_events(&self, config: &impl Config) -> Vec<DisplayEvent> {
        let mut events = vec![];
        if let Some(workspaces) = config.workspaces() {
            if workspaces.is_empty() {
                // tell manager about existing screens
                self.xw.get_screens().into_iter().for_each(|screen| {
                    let e = DisplayEvent::ScreenCreate(screen);
                    events.push(e);
                });
            } else {
                for wsc in &workspaces {
                    let mut screen = Screen::from(wsc);
                    screen.root = WindowHandle::XlibHandle(self.root);
                    let e = DisplayEvent::ScreenCreate(screen);
                    events.push(e);
                }
            }
        }

        // Tell manager about existing windows.
        events.append(&mut self.find_all_windows());

        events
    }

    fn find_all_windows(&self) -> Vec<DisplayEvent> {
        let mut all: Vec<DisplayEvent> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => handles.into_iter().for_each(|handle| {
                let attrs = match self.xw.get_window_attrs(handle) {
                    Ok(x) => x,
                    Err(_) => return,
                };
                let state = match self.xw.get_wm_state(handle) {
                    Some(state) => state,
                    None => return,
                };
                if attrs.map_state == xlib::IsViewable || state == ICONIC_STATE {
                    if let Some(event) = self.xw.setup_window(handle) {
                        all.push(event);
                    }
                }
            }),
            Err(err) => {
                println!("ERROR: {}", err);
            }
        }
        all
    }

    // Display actions.
    fn from_kill_window(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        self.xw.kill_window(&handle);
        None
    }

    fn from_added_window(
        &mut self,
        handle: WindowHandle,
        floating: bool,
        follow_mouse: bool,
    ) -> Option<DisplayEvent> {
        self.xw.setup_managed_window(handle, floating, follow_mouse)
    }

    fn from_move_mouse_over(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        if let WindowHandle::XlibHandle(win) = handle {
            let _ = self.xw.move_cursor_to_window(win);
        }
        None
    }

    fn from_move_mouse_over_point(&mut self, point: (i32, i32)) -> Option<DisplayEvent> {
        let _ = self.xw.move_cursor_to_point(point);
        None
    }

    fn from_destroyed_window(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        self.xw.teardown_managed_window(&handle);
        None
    }

    fn from_unfocus(&mut self, handle: Option<WindowHandle>) -> Option<DisplayEvent> {
        self.xw.unfocus(handle);
        None
    }

    fn from_set_state(
        &mut self,
        handle: WindowHandle,
        toggle_to: bool,
        window_state: WindowState,
    ) -> Option<DisplayEvent> {
        // TODO: impl from for windowstate and xlib::Atom
        let state = match window_state {
            WindowState::Modal => self.xw.atoms.NetWMStateModal,
            WindowState::Sticky => self.xw.atoms.NetWMStateSticky,
            WindowState::MaximizedVert => self.xw.atoms.NetWMStateMaximizedVert,
            WindowState::MaximizedHorz => self.xw.atoms.NetWMStateMaximizedHorz,
            WindowState::Shaded => self.xw.atoms.NetWMStateShaded,
            WindowState::SkipTaskbar => self.xw.atoms.NetWMStateSkipTaskbar,
            WindowState::SkipPager => self.xw.atoms.NetWMStateSkipPager,
            WindowState::Hidden => self.xw.atoms.NetWMStateHidden,
            WindowState::Fullscreen => self.xw.atoms.NetWMStateFullscreen,
            WindowState::Above => self.xw.atoms.NetWMStateAbove,
            WindowState::Below => self.xw.atoms.NetWMStateBelow,
        };
        self.xw.set_state(handle, toggle_to, state);
        None
    }

    fn from_set_window_order(&mut self, windows: &[Window]) -> Option<DisplayEvent> {
        // The windows we are managing should be behind unmanaged windows. Unless they are
        // fullscreen, or their children.
        let (fullscreen_windows, other): (Vec<&Window>, Vec<&Window>) =
            windows.iter().partition(|w| w.is_fullscreen());
        // Fullscreen windows.
        let level2: Vec<WindowHandle> = fullscreen_windows.iter().map(|w| w.handle).collect();
        let (fullscreen_children, other): (Vec<&Window>, Vec<&Window>) = other
            .iter()
            .partition(|w| level2.contains(&w.transient.unwrap_or(WindowHandle::XlibHandle(0))));
        // Fullscreen windows children.
        let level1: Vec<WindowHandle> = fullscreen_children.iter().map(|w| w.handle).collect();
        // Left over managed windows.
        let level4: Vec<WindowHandle> = other.iter().map(|w| w.handle).collect();
        // Unmanaged windows.
        let level3: Vec<WindowHandle> = self
            .xw
            .get_all_windows()
            .unwrap_or_default()
            .iter()
            .filter(|&w| *w != self.root)
            .map(|w| WindowHandle::XlibHandle(*w))
            .filter(|&h| !windows.iter().any(|w| w.handle == h))
            .collect();
        let all: Vec<WindowHandle> = level1
            .iter()
            .chain(level2.iter())
            .chain(level3.iter())
            .chain(level4.iter())
            .copied()
            .collect();
        self.xw.restack(all);
        None
    }

    fn from_move_to_top(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        self.xw.move_to_top(&handle);
        None
    }

    fn from_ready_to_move_window(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        self.xw.set_mode(Mode::ReadyToMove(handle));
        None
    }

    fn from_ready_to_resize_window(&mut self, handle: WindowHandle) -> Option<DisplayEvent> {
        self.xw.set_mode(Mode::ReadyToResize(handle));
        None
    }

    fn from_set_current_tags(&mut self, tags: &[TagId]) -> Option<DisplayEvent> {
        self.xw.set_current_desktop(&tags);
        None
    }

    fn from_set_window_tags(
        &mut self,
        handle: WindowHandle,
        tags: &[TagId],
    ) -> Option<DisplayEvent> {
        if let WindowHandle::XlibHandle(window) = handle {
            self.xw.set_window_desktop(window, &tags);
        }
        None
    }

    fn from_reload_key_grabs(&mut self, keybinds: &[Keybind]) -> Option<DisplayEvent> {
        self.xw.reset_grabs(&keybinds);
        None
    }

    fn from_configure_xlib_window(&mut self, window: Window) -> Option<DisplayEvent> {
        self.xw.configure_window(&window);
        None
    }

    fn from_window_take_focus(
        &mut self,
        window: Window,
        previous_window: Option<Window>,
    ) -> Option<DisplayEvent> {
        self.xw.window_take_focus(&window, previous_window.as_ref());
        None
    }

    fn from_focus_window_under_cursor(&mut self) -> Option<DisplayEvent> {
        let point = self.xw.get_cursor_point().ok()?;
        let evt = DisplayEvent::MoveFocusTo(point.0, point.1);
        Some(evt)
    }

    fn from_normal_mode(&mut self) -> Option<DisplayEvent> {
        self.xw.set_mode(Mode::Normal);
        None
    }
}
