
use super::utils::window::Handle;
use super::utils::window::Window;
mod mock_display_server;
mod xlib_display_server;
mod event_handler;

pub use self::mock_display_server::MockDisplayServer;
pub use self::xlib_display_server::XlibDisplayServer;
pub use self::event_handler::Events;


pub trait DisplayServer {
    fn new() -> Self;
    fn find_all_windows(&self) -> Vec<Window>;

    //fn event_handler(&mut self, handler: &Events){
    //    self.handler = handler;
    //}

}




#[test]
fn they_should_be_able_to_woot(){
    MockDisplayServer::woot();
    XlibDisplayServer::woot();
    assert!(true);
}

