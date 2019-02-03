use super::DisplayServer;
use super::Window;
use super::Handle;
use super::Screen;
use super::DisplayEventHandler;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockDisplayServer{
    pub screens: Vec<Screen>
}

impl DisplayServer for MockDisplayServer  {

    fn new() -> MockDisplayServer {
        MockDisplayServer{
            screens: vec![]
        }
    }

    //testing a couple mock event
    fn watch_events<DEH: DisplayEventHandler>(&self, h: Arc<Mutex<DEH>>){
        if let Ok(ref mut handler) = h.lock() {
            //new mock window
            let w = Window::new( Handle::MockHandle(1), Some("MOCK".to_owned() ));
            handler.on_new_window(w);
            //new mock screen
            let s = Screen::new(600,800);
            handler.on_new_screen(s);
        } else {
            panic!("MUTEX FAILED!")
        }
    }

}

impl MockDisplayServer  {


    //pub fn start_event_loop(&mut self){
    //}
}


//#[test]
//fn it_should_be_able_to_update_the_list_of_windows(){
//    let mut ds:MockDisplayServer = DisplayServer::new();
//    ds.find_all_windows();
//    assert!(ds.manager.windows.len() == 10, "wasn't able to get a list of windows")
//}

