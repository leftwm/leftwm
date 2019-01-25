use super::DisplayServer;
use super::Window;
use super::Handle;

pub struct MockDisplayServer{
}

impl DisplayServer for MockDisplayServer {

    fn new() -> MockDisplayServer {
        MockDisplayServer{}
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


#[test]
fn it_should_be_able_to_get_a_list_of_windows(){
    let ds:MockDisplayServer = DisplayServer::new();
    assert!(ds.find_all_windows().len() == 10, "wasn't able to get a list of windows")
}

