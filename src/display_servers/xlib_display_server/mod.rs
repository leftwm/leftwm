use super::DisplayServer;
use super::Window;
use super::Handle;
use super::Manager;
use super::Screen;

mod xwrap;
mod event_dispatch;
use xwrap::XWrap;


pub struct XlibDisplayServer{
    //let game = Arc::new(Mutex::new( game::Game::new(games_outs) ));
    xw: XWrap,
    manager: Manager,
}


impl DisplayServer for XlibDisplayServer {


    fn new() -> XlibDisplayServer { 
        XlibDisplayServer{ 
            xw: XWrap::new(),
            manager: Manager::new()
        }
    }

    fn find_all_windows(&mut self) {
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
                    let w = Window{ 
                        name: self.xw.get_window_name(handle),
                        handle: Handle::XlibHandle(handle)
                    };
                    self.manager.on_new_window(w);
                }
            }
          }
          Err(err) => {
              println!("ERROR: {}", err);
          }
        }
    }


}



impl XlibDisplayServer {

    pub fn start_event_loop(&mut self){
        //subscribe to WM type events
        self.find_all_windows();

        for s in self.xw.get_screens() {
            let ss = unsafe{ *s };
            let screen = Screen::new(ss.height, ss.width);
            self.manager.add_screen(screen);
        }

        self.xw.init();

        loop{
            //will block waiting for the next xlib event.
            let raw_event = self.xw.get_next_event();
            event_dispatch::dispatch( &mut self.manager, &self.xw, raw_event);
        }
    }

}



