use super::DisplayServer;
use super::Window;
use super::Handle;
use super::event_handler;

pub struct MockDisplayServer<'a>{
    events: Vec<&'a event_handler::Events>
}

impl<'a> DisplayServer<'a> for MockDisplayServer<'a>  {

    fn new() -> MockDisplayServer<'a> {
        MockDisplayServer{
            events: Vec::new()
        }
    }

    fn event_handler(&mut self, handler: &'a event_handler::Events){
        self.events.push( handler );
    }


    fn find_all_windows(&self) -> Vec<Window> {
        let mut list: Vec<Window> = Vec::new();
        for i in 0..10 {
            list.push( Window{
                handle: Handle::MockHandle(i)
            });
        }
        list
    }



}

impl<'a> MockDisplayServer<'a>  {

    pub fn create_mock_window(&self){
        for h in self.events.clone() {
            let w = Window{ handle: Handle::MockHandle(1) };
            h.on_new_window(w);
        }
    }
}


#[test]
fn it_should_be_able_to_get_a_list_of_windows(){
    let ds:MockDisplayServer = DisplayServer::new();
    assert!(ds.find_all_windows().len() == 10, "wasn't able to get a list of windows")
}

