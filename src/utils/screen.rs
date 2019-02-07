use std::convert::From;
use x11_dl::xlib;

#[derive(Debug, Clone)]
pub struct Screen {
    pub height: i32,
    pub width: i32,
}

impl Screen {
    pub fn new(height: i32, width: i32) -> Screen {
        Screen { height, width }
    }
}

impl From<&xlib::Screen> for Screen {
    fn from(xscreen: &xlib::Screen) -> Self {
        Screen {
            height: xscreen.height,
            width: xscreen.width,
        }
    }
}
