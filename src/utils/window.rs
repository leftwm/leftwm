
use x11_dl::xlib;


type MockHandle = i32;
#[derive(Debug)]
pub enum Handle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window)
}

#[derive(Debug)]
pub struct Window {
    pub handle: Handle
}


impl Window{
}
