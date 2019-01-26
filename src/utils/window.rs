
use x11_dl::xlib;


type MockHandle = i32;
#[derive(Debug, Clone)]
pub enum Handle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window)
}

#[derive(Debug, Clone)]
pub struct Window {
    pub handle: Handle
}


impl Window{
}
