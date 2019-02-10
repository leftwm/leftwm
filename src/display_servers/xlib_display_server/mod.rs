use super::config::Config;
use super::event_queue;
use super::event_queue::EventQueueItem;
use super::utils;
use super::utils::command::CommandBuilder;
use super::DisplayServer;
use std::sync::Once;

mod event_translate;
mod xwrap;
use xwrap::XWrap;

static SETUP: Once = Once::new();

pub struct XlibDisplayServer {
    xw: XWrap,
    command_builder: CommandBuilder,
}

impl DisplayServer for XlibDisplayServer {
    fn new(config: &Config) -> XlibDisplayServer {
        let me = XlibDisplayServer { 
            xw: XWrap::new(),
            command_builder: CommandBuilder::new(config)
        };
        me.xw.init(); //setup events masks
        me
    }

    fn update_windows(&self, windows: Vec<&utils::window::Window>) {
        for window in windows {
            self.xw.update_window(&window)
        }
    }

    fn get_next_events(&self) -> Vec<event_queue::EventQueueItem> {
        let mut events = vec![];
        SETUP.call_once(|| {
            for e in self.initial_events() {
                (&mut events).push(e);
            }
        });
        let xlib_event = self.xw.get_next_event();
        let event = event_translate::from_xevent(&self.xw, &self.command_builder ,xlib_event);
        if let Some(e) = event {
            //if we have a new windows go ahead and subscribe to its events
            if let EventQueueItem::WindowCreate(new_win) = &e {
                self.xw.subscribe_to_window_events(new_win);
            }
            events.push(e)
        }
        events
    }
}

impl XlibDisplayServer {
    /**
     * return a vec of events for setting up state of WM
     */
    fn initial_events(&self) -> Vec<event_queue::EventQueueItem> {
        let mut events = vec![];
        // tell manager about existing screens
        for s in self.xw.get_screens() {
            let screen = utils::screen::Screen::from(&s);
            let e = EventQueueItem::ScreenCreate(screen);
            events.push(e);
        }
        // tell manager about existing windows
        for w in &self.find_all_windows() {
            let e = EventQueueItem::WindowCreate(w.clone());
            events.push(e);
        }
        events
    }

    fn find_all_windows(&self) -> Vec<utils::window::Window> {
        use utils::window::Window;
        use utils::window::WindowHandle;
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
