
use x11_dl::xlib;


type MockHandle = i32;
#[derive(Debug, Clone, PartialEq)]
pub enum Handle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window)
}




#[derive(Debug, Clone)]
pub struct Window {
    pub handle: Handle,
    pub name: Option<String>,
    pub tags: Vec<String>,
}


impl Window{

    pub fn new(h: Handle, name: Option<String>) -> Window{
        Window{
            handle:h,
            name:name,
            tags: Vec::new()
        }
    }

}
