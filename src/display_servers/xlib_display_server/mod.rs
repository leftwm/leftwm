use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::models::Screen;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::utils;
use crate::DisplayEvent;
use crate::DisplayServer;
use std::sync::Once;
use x11_dl::xlib;

mod event_translate;
mod xatom;
mod xwrap;
use xwrap::XWrap;

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
        let me = XlibDisplayServer {
            xw: wrap,
            root,
            config: config.clone(),
        };
        me.xw.init(config); //setup events masks
        me
    }

    fn update_windows(&self, windows: Vec<&Window>) {
        for window in windows {
            self.xw.update_window(&window)
        }
    }

    fn get_next_events(&self) -> Vec<DisplayEvent> {
        let mut events = vec![];
        SETUP.call_once(|| {
            for e in self.initial_events() {
                (&mut events).push(e);
            }
        });
        let xlib_event = self.xw.get_next_event();
        let event = event_translate::from_xevent(&self.xw, xlib_event);
        if let Some(e) = event {
            events.push(e)
        }
        events
    }

    fn execute_action(&self, act: DisplayAction) -> Result<(), Box<std::error::Error>> {
        match act {
            DisplayAction::KillWindow(w) => self.xw.kill_window(w),
            DisplayAction::AddedWindow(w) => self.xw.setup_managed_window(w),
            DisplayAction::WindowTakeFocus(w) => self.xw.window_take_focus(w),
        }
        Ok(())
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
                let mut screen = Screen::from( wsc );
                screen.root = WindowHandle::XlibHandle( self.root.clone() );
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
                    let transient = self.xw.get_transient_for(handle);
                    let managed: bool;
                    match transient {
                        Some(_) => managed = attrs.map_state == 2,
                        _ => managed = attrs.override_redirect <= 0 && attrs.map_state == 2,
                    }
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
