use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Screen;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::utils;
use crate::DisplayEvent;
use crate::DisplayServer;
use crate::DisplayServerMode;
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

pub struct XlibDisplayServer {
    xw: XWrap,
    root: xlib::Window,
    config: Config,
}

impl DisplayServer for XlibDisplayServer {
    fn new(config: &Config) -> XlibDisplayServer {
        let wrap = XWrap::new();
        let root = wrap.get_default_root();
        let mut me = XlibDisplayServer {
            xw: wrap,
            root,
            config: config.clone(),
        };

        me.xw.mod_key_mask = utils::xkeysym_lookup::into_mod(&config.modkey);
        me.xw.init(config); //setup events masks
        me
    }

    fn update_windows(&self, windows: Vec<&Window>, focused_window: Option<&Window>) {
        for window in windows {
            let is_focused = match focused_window {
                Some(f) => f.handle == window.handle,
                None => false,
            };
            self.xw.update_window(&window, is_focused);
        }
    }

    fn update_workspaces(&self, workspaces: Vec<&Workspace>, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            for tag in &focused.tags {
                self.xw.set_current_desktop( tag );
            }
        }
        let mut tags: Vec<&String> = vec![];
        for w in workspaces{
            for t in &w.tags{
                tags.push( t );
            }
        }
        self.xw.set_current_viewport( tags );
    }

    fn get_next_events(&self) -> Vec<DisplayEvent> {
        let mut events = vec![];
        SETUP.call_once(|| {
            for e in self.initial_events() {
                (&mut events).push(e);
            }
        });
        let xlib_event = self.xw.get_next_event();
        let event = XEvent(&self.xw, xlib_event).into();

        if let Some(e) = event {
            events.push(e)
        }
        events
    }

    fn execute_action(&mut self, act: DisplayAction) -> Option<DisplayEvent> {
        match act {
            DisplayAction::KillWindow(w) => self.xw.kill_window(w),
            DisplayAction::AddedWindow(w) => {
                return self.xw.setup_managed_window(w);
            }
            DisplayAction::DestroyedWindow(w) => self.xw.teardown_managed_window(w),
            DisplayAction::WindowTakeFocus(w) => self.xw.window_take_focus(w),
            DisplayAction::MoveToTop(w) => self.xw.move_to_top(w),
            DisplayAction::StartMovingWindow(w) => {
                self.xw.set_mode(DisplayServerMode::MovingWindow(w))
            }
            DisplayAction::StartResizingWindow(w) => {
                self.xw.set_mode(DisplayServerMode::ResizingWindow(w))
            }
            DisplayAction::NormalMode => self.xw.set_mode(DisplayServerMode::NormalMode),
            DisplayAction::SetCurrentTags(tags) => self.xw.set_current_desktop(&tags),
        }
        None
    }
}

impl XlibDisplayServer {
    /**
     * return a vec of events for setting up state of WM
     */
    fn initial_events(&self) -> Vec<DisplayEvent> {
        let mut events = vec![];
        if self.config.workspace.is_empty() {
            // tell manager about existing screens
            for screen in self.xw.get_screens() {
                let e = DisplayEvent::ScreenCreate(screen);
                events.push(e);
            }
        } else {
            for wsc in &self.config.workspace {
                let mut screen = Screen::from(wsc);
                screen.root = WindowHandle::XlibHandle(self.root);
                let e = DisplayEvent::ScreenCreate(screen);
                events.push(e);
            }
        }
        // tell manager about existing windows
        for w in &self.find_all_windows() {
            let e = DisplayEvent::WindowCreate(w.clone());
            events.push(e);
        }
        events
    }

    fn find_all_windows(&self) -> Vec<Window> {
        let mut all: Vec<Window> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => {
                for handle in handles {
                    let attrs = self.xw.get_window_attrs(handle).unwrap();

                    let managed = match self.xw.get_transient_for(handle) {
                        Some(_) => attrs.map_state == 2,
                        _ => attrs.override_redirect <= 0 && attrs.map_state == 2,
                    };
                    if managed {
                        let name = self.xw.get_window_name(handle);
                        let w = Window::new(WindowHandle::XlibHandle(handle), name);
                        all.push(w);
                    }
                }
            }
            Err(err) => {
                println!("ERROR: {}", err);
            }
        }
        all
    }
}
