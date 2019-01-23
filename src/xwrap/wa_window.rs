use x11_dl::xlib;
use super::xwrap::XWrap;

#[derive(Debug)]
pub struct WaWindow {
    pub handle: xlib::Window,
    pub transient: Option<xlib::Window>,
    pub name: String,
    pub is_managed: bool 
}




impl WaWindow{

    pub fn build(xw: &XWrap, handle: xlib::Window ) -> WaWindow {
        let attrs = xw.get_window_attrs(handle).unwrap();
        let transient = xw.get_transient_for(handle);
        let managed : bool;
        match transient {
            Some(_) => { 
                managed = attrs.map_state == 2
            },
            _ => {
                managed = !(attrs.override_redirect > 0) && attrs.map_state == 2
            }
        }
        let name = xw.get_window_title(handle);
        WaWindow { 
            handle: handle,
            name: if let Ok(n) = name { n } else { "".to_string() },
            transient: transient,
            is_managed: managed,
        }
    }

    pub fn find_all(xw: &XWrap) -> Vec<WaWindow> {
        match xw.get_all_windows() {
          Ok(handles) => {
            handles.into_iter().map(|handle| { 
                WaWindow::build(xw, handle) 
            }).filter(|w| w.is_managed ).collect()
          }
          Err(err) => {
              println!("ERROR: {}", err);
            return Vec::new();
          }
        }
    }

}


