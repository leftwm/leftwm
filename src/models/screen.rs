use super::WindowHandle;
use std::convert::From;
use x11_dl::xlib;

#[derive(Debug, Clone)]
pub struct Screen {
    pub root: WindowHandle,
    pub height: i32,
    pub width: i32,
    pub x: i32,
    pub y: i32,
}

impl Screen {
    pub fn new(height: i32, width: i32, x: i32, y: i32) -> Screen {
        Screen {
            root: WindowHandle::MockHandle(0),
            height,
            width,
            x,
            y,
        }
    }
}

impl From<&xlib::XWindowAttributes> for Screen {
    fn from(root: &xlib::XWindowAttributes) -> Self {
        Screen {
            root: WindowHandle::XlibHandle(root.root),
            height: root.height,
            width: root.width,
            x: root.x,
            y: root.y,
        }
    }
}

impl From<&x11_dl::xinerama::XineramaScreenInfo> for Screen {
    fn from(root: &x11_dl::xinerama::XineramaScreenInfo) -> Self {
        Screen {
            root: WindowHandle::MockHandle(0),
            height: root.height.into(),
            width: root.width.into(),
            x: root.x_org.into(),
            y: root.y_org.into(),
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            root: WindowHandle::MockHandle(0),
            height: 600,
            width: 800,
            x: 0,
            y: 0,
        }
    }
}
