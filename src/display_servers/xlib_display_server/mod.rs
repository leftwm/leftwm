use super::DisplayServer;
use super::Window;
use super::Handle;
use super::DisplayEventHandler;
//use super::Screen;
//use super::config;
use std::sync::{Arc, Mutex};
use std::thread;

mod xwrap;
mod event_dispatch;
use xwrap::XWrap;


#[derive(Clone)]
pub struct XlibDisplayServer{
    xw: XWrap
}


impl DisplayServer for XlibDisplayServer {


    fn new() -> XlibDisplayServer { 
        XlibDisplayServer{ 
            xw: XWrap::new(),
        }
    }

    fn watch_events<DEH: DisplayEventHandler>(&self, handler_mutex: Arc<Mutex<DEH>>){

        { //in block to relock the mutex
            let h_for_windows = handler_mutex.clone();
            let mut handler = h_for_windows.lock().unwrap();
            let windows = self.find_all_windows();
            for w in windows {
                handler.on_new_window(w);
            }
        }


        let hander_for_thread = handler_mutex.clone();
        let child = thread::spawn( || {
            // some work here
            loop{
                //let raw_event = self.xw.get_next_event();
                //let handler = hander_for_thread.lock().unwrap();
            }
        });
        
        ////subscribe to WM type events

        //let screens: Vec<Screen> = self.xw.get_screens().into_iter().map(|s|{
        //    let ss = unsafe{ *s };
        //    Screen::new(ss.height, ss.width)
        //}).collect();
        //config::load_config( &mut self.manager, screens );
        //self.xw.init();

        //loop{
        //    //will block waiting for the next xlib event.
        //    let raw_event = self.xw.get_next_event();
        //    event_dispatch::dispatch( self, raw_event);
        //}
    }




    
}



impl XlibDisplayServer {


    fn find_all_windows(&self) -> Vec<Window> {
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
                    let w = Window::new( Handle::XlibHandle(handle), name );
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



