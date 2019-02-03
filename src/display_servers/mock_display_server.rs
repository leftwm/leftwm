use super::DisplayServer;
use super::Window;
use super::Handle;
use super::DisplayEventHandler;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockDisplayServer{
}

impl DisplayServer for MockDisplayServer  {

    fn new() -> MockDisplayServer {
        MockDisplayServer{ }
    }

    fn watch_events<DEH: DisplayEventHandler>(&self, handler: Arc<Mutex<DEH>>){
    }


    //fn find_all_windows(&mut self) {
    //    for i in 0..10 {
    //        let mut name: String = "MOCK: ".to_owned();
    //        name.push_str( &(i.to_string())[..] );
    //        let w = Window::new( Handle::MockHandle(i), Some(name));
    //        self.manager.on_new_window( &self.clone(), w);
    //    }
    //}

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

