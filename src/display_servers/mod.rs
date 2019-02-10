use super::config;
use super::event_queue;
use super::utils;
use super::utils::screen::Screen;
mod mock_display_server;
mod xlib_display_server;

pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;

pub trait DisplayServer {
    fn new(config: &config::Config) -> Self;
    //fn find_all_windows(&mut self);
    fn get_next_events(&self) -> Vec<event_queue::EventQueueItem>;
    fn update_windows(&self, windows: Vec<&utils::window::Window>);
}

//#[test]
//fn it_should_alert_for_new_windows(){
//    struct H {}
//    impl Events for H{
//        fn on_new_window(&self, w:Window){
//            assert!(true);
//        }
//    }
//    let mut ds:MockDisplayServer = DisplayServer::new();
//    let handler = H{};
//    ds.event_handler( &handler );
//    ds.create_mock_window();
//}
//
