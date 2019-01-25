use super::DisplayServer;
use super::Window;
use super::Handle;

mod xwrap;
use xwrap::XWrap;

pub struct XlibDisplayServer{
    xw: XWrap
}
impl DisplayServer for XlibDisplayServer {

    fn new() -> XlibDisplayServer { 
        XlibDisplayServer{ xw: XWrap::new() }
    }


    fn find_all_windows(&self) -> Vec<Window> {

        match self.xw.get_all_windows() {
          Ok(handles) => {

            let mut list: Vec<Window> = Vec::new();
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
                    list.push( Window{ 
                        handle: Handle::XlibHandle(handle)
                    })
                }
            }
            list
          }
          Err(err) => {
              println!("ERROR: {}", err);
            return Vec::new();
          }
        }

    }

}


#[test]
fn it_should_be_able_to_get_a_list_of_windows(){
    let ds:XlibDisplayServer = DisplayServer::new();
    assert!(ds.find_all_windows().len() > 0, "wasn't able to get a list of windows")
}

