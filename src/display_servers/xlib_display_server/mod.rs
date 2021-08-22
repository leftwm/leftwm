use crate::config::Config;
use crate::config::ThemeSetting;
use crate::display_action::DisplayAction;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::utils;
use crate::DisplayEvent;
use crate::DisplayServer;
use std::sync::Once;
use x11_dl::xlib;

mod event_translate;
mod event_translate_client_message;
mod event_translate_property_notify;
mod xatom;
mod xwrap;
pub use xwrap::XWrap;

use event_translate::XEvent;
mod xcursor;

static SETUP: Once = Once::new();

pub struct XlibDisplayServer<C: Config> {
    xw: XWrap,
    root: xlib::Window,
    config: C,
    theme: ThemeSetting,
}

impl<C> DisplayServer<C> for XlibDisplayServer<C>
where
    C: Config,
{
    fn new(config: C) -> Self {
        let theme = ThemeSetting::default();
        let mut wrap = XWrap::new();

        wrap.focus_behaviour = config.focus_behaviour();
        wrap.mouse_key_mask = utils::xkeysym_lookup::into_mod(config.mousekey());
        wrap.init(&config, &theme); //setup events masks

        let root = wrap.get_default_root();

        Self {
            xw: wrap,
            root,
            theme,
            config,
        }
    }

    fn update_theme_settings(&mut self, settings: ThemeSetting) {
        self.theme = settings;
        self.xw.load_colors(&self.theme);
    }

    fn update_windows(&self, windows: Vec<&Window>, focused_window: Option<&Window>) {
        for window in windows {
            let is_focused = match focused_window {
                Some(f) => f.handle == window.handle,
                None => false,
            };
            self.xw.update_window(window, is_focused);
            if window.is_fullscreen() {
                self.xw.move_to_top(&window.handle);
            }
        }
    }

    fn update_workspaces(&self, _workspaces: Vec<&Workspace>, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            focused
                .tags
                .iter()
                .for_each(|tag| self.xw.set_current_desktop(tag));
        }
    }

    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        let mut events = vec![];
        SETUP.call_once(|| {
            for e in self.initial_events() {
                (&mut events).push(e);
            }
        });

        let event_in_queue = self.xw.queue_len();

        for _ in 0..event_in_queue {
            let xlib_event = self.xw.get_next_event();
            let event = XEvent(&self.xw, xlib_event).into();
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
            DisplayAction::KillWindow(w) => {
                self.xw.kill_window(&w);
                None
            }
            DisplayAction::AddedWindow(w, follow_mouse) => {
                self.xw.setup_managed_window(w, follow_mouse)
            }
            DisplayAction::MoveMouseOver(handle) => {
                if let WindowHandle::XlibHandle(win) = handle {
                    let _ = self.xw.move_cursor_to_window(win);
                }
                None
            }
            DisplayAction::MoveMouseOverPoint(point) => {
                let _ = self.xw.move_cursor_to_point(point);
                None
            }
            DisplayAction::DestroyedWindow(w) => {
                self.xw.teardown_managed_window(&w);
                None
            }
            DisplayAction::WindowTakeFocus(w) => {
                self.xw.window_take_focus(&w);
                None
            }
            DisplayAction::Unfocus => {
                self.xw.unfocus();
                None
            }
            DisplayAction::SetFullScreen(w, fullscreen) => {
                self.xw.set_fullscreen(&w, fullscreen);
                None
            }
            DisplayAction::SetWindowOrder(wins) => {
                // get all the windows are aren't managing.
                // They should be in front of our windows.
                let unmanged: Vec<WindowHandle> = self
                    .xw
                    .get_all_windows()
                    .unwrap_or_default()
                    .iter()
                    .filter(|&x| *x != self.root)
                    .map(|x| WindowHandle::XlibHandle(*x))
                    .filter(|h| !wins.contains(h))
                    .collect();
                let all: Vec<WindowHandle> = unmanged.iter().chain(wins.iter()).copied().collect();
                self.xw.restack(all);
                None
            }
            DisplayAction::FocusWindowUnderCursor => {
                let point = self.xw.get_cursor_point().ok()?;
                let evt = DisplayEvent::MoveFocusTo(point.0, point.1);
                Some(evt)
            }
            DisplayAction::StartMovingWindow(w) => {
                self.xw.set_mode(Mode::MovingWindow(w));
                None
            }
            DisplayAction::StartResizingWindow(w) => {
                self.xw.set_mode(Mode::ResizingWindow(w));
                None
            }
            DisplayAction::NormalMode => {
                self.xw.set_mode(Mode::Normal);
                None
            }
            DisplayAction::SetCurrentTags(tags) => {
                self.xw.set_current_desktop(&tags);
                None
            }
            DisplayAction::SetWindowTags(handle, tag) => {
                if let WindowHandle::XlibHandle(window) = handle {
                    self.xw.set_window_desktop(window, &tag);
                }
                None
            }
        };
        if event.is_some() {
            log::trace!("DisplayEvent: {:?}", event);
        }
        event
    }
}

impl<C> XlibDisplayServer<C>
where
    C: Config,
{
    /// Return a vec of events for setting up state of WM.
    fn initial_events(&self) -> Vec<DisplayEvent> {
        let mut events = vec![];
        if let Some(workspaces) = self.config.workspaces() {
            if workspaces.is_empty() {
                // tell manager about existing screens
                self.xw.get_screens().into_iter().for_each(|screen| {
                    let e = DisplayEvent::ScreenCreate(screen);
                    events.push(e);
                });
            } else {
                for wsc in workspaces.iter() {
                    let mut screen = Screen::from(wsc);
                    screen.root = WindowHandle::XlibHandle(self.root);
                    let e = DisplayEvent::ScreenCreate(screen);
                    events.push(e);
                }
            }
        }

        // tell manager about existing windows
        self.find_all_windows().into_iter().for_each(|w| {
            let cursor = self.xw.get_cursor_point().ok().unwrap_or_default();
            let e = DisplayEvent::WindowCreate(w, cursor.0, cursor.1);
            events.push(e);
        });

        events
    }

    pub fn verify_focused_window(&mut self) -> Vec<DisplayEvent> {
        self.verify_focused_window_work().unwrap_or_default()
    }

    fn verify_focused_window_work(&mut self) -> Option<Vec<DisplayEvent>> {
        let point = self.xw.get_cursor_point().ok()?;
        Some(vec![DisplayEvent::VerifyFocusedAt(point.0, point.1)])
    }

    fn find_all_windows(&self) -> Vec<Window> {
        let mut all: Vec<Window> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => handles.into_iter().for_each(|handle| {
                let attrs = match self.xw.get_window_attrs(handle) {
                    Ok(x) => x,
                    Err(_) => return,
                };
                let managed = match self.xw.get_transient_for(handle) {
                    Some(_) => attrs.map_state == 2,
                    None => attrs.override_redirect <= 0 && attrs.map_state == 2,
                };
                if managed {
                    let name = self.xw.get_window_name(handle);
                    let pid = self.xw.get_window_pid(handle);
                    let w = Window::new(WindowHandle::XlibHandle(handle), name, pid);
                    all.push(w);
                }
            }),
            Err(err) => {
                println!("ERROR: {}", err);
            }
        }
        all
    }

    pub async fn wait_readable(&mut self) {
        self.xw.wait_readable().await;
    }

    pub fn flush(&self) {
        self.xw.flush();
    }
}
