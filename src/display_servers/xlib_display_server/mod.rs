use super::DisplayServer;
use super::event_queue;
use super::event_queue::EventQueueItem;
use super::utils::window::Window;
use super::utils;
use std::thread;

mod xwrap;
mod event_translate;
use xwrap::XWrap;


pub struct XlibDisplayServer{
    xw: XWrap
}


impl DisplayServer for XlibDisplayServer {


    fn new() -> XlibDisplayServer { 
        XlibDisplayServer{ 
            xw: XWrap::new(),
        }
    }

    fn update_windows(&self, windows: Vec<&utils::window::Window> ){
        for window in windows {
            self.xw.update_window(&window)
        }
    }


    fn watch_events(&self, queue: event_queue::EventQueue) {
        // before starting the watching thread find all existing windows
        {
            for w in &self.find_all_windows() {
                let e = EventQueueItem::WindowCreate(w.clone());
                queue.lock().unwrap().push_back(e);
            }
        }
        thread::spawn( move || {
            //NOTE: we need another connection to XLIB to handle watching to events
            //this is to prevent locking and other threading issues
            let xw = XWrap::new();
            xw.init(); //setup events masks
            loop{
                let xlib_event = xw.get_next_event();
                let event = event_translate::from_xevent(&xw, xlib_event);
                if let Some(e) = event {
                    queue.lock().unwrap().push_back(e);
                }
            }
        });
    }




    
}



impl XlibDisplayServer {



    fn find_all_windows(&self) -> Vec<utils::window::Window> {
        use utils::window::Window;
        use utils::window::WindowHandle;
        let mut all :Vec<Window> = Vec::new();
        match self.xw.get_all_windows() {
          Ok(handles) => {
            for handle in handles {
                let attrs = self.xw.get_window_attrs(handle).unwrap();
                let transient = self.xw.get_transient_for(handle);
                let managed : bool;
                match transient {
                    Some(_) => { 
                        managed = attrs.map_state == 2
                    },
                    _ => {
                        managed = !(attrs.override_redirect > 0) && attrs.map_state == 2
                    }
                }
                if managed {
                    let name = self.xw.get_window_name(handle);
                    let w = Window::new( WindowHandle::XlibHandle(handle), name );
                    all.push(w);
                }
            }
          }
          Err(err) => {
              println!("ERROR: {}", err);
          }
        }
        return all;
    }



}



