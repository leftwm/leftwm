use super::DisplayServer;
use super::Window;
use super::Handle;
use super::Manager;
use super::Screen;
use super::config;

mod xwrap;
mod event_dispatch;
use xwrap::XWrap;


#[derive(Clone)]
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

    fn get_manager(&self) -> &Manager {
        &self.manager
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
                    let name = self.xw.get_window_name(handle);
                    let w = Window::new( Handle::XlibHandle(handle), name );
                    self.manager.on_new_window(&self.clone(), w);
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

        let screens: Vec<Screen> = self.xw.get_screens().into_iter().map(|s|{
            let ss = unsafe{ *s };
            Screen::new(ss.height, ss.width)
        }).collect();
        config::load_config( &mut self.manager, screens );
        self.xw.init();

        loop{
            //will block waiting for the next xlib event.
            let raw_event = self.xw.get_next_event();
            event_dispatch::dispatch( self, raw_event);
        }
    }


}



