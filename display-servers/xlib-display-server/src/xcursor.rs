use std::os::raw::{c_uint, c_ulong};
use x11_dl::xlib;

//#![allow(non_snake_case)]
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct XCursor {
    pub normal: c_ulong,
    pub resize: c_ulong,
    pub move_: c_ulong,
}

//pointer def can be found at https://tronche.com/gui/x/xlib/appendix/b/
const LEFT_PTR: c_uint = 68;
const SIZING: c_uint = 120;
const FLEUR: c_uint = 52;

impl XCursor {
    pub fn new(xlib: &xlib::Xlib, dpy: *mut xlib::Display) -> Self {
        unsafe {
            Self {
                normal: (xlib.XCreateFontCursor)(dpy, LEFT_PTR),
                resize: (xlib.XCreateFontCursor)(dpy, SIZING),
                move_: (xlib.XCreateFontCursor)(dpy, FLEUR),
            }
        }
    }
}
