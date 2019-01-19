
use x11::xlib;
use x11::xlib::{ Window };
use std::os::raw::{ c_int };
use std::ffi::{ CString };
use std::ptr;

fn main() {
    let display = unsafe { xlib::XOpenDisplay( ptr::null() ) };
    let screen_num = unsafe{ xlib::XDefaultScreen(display) };
    let root:Window = unsafe{ xlib::XRootWindow(display, screen_num) };

    let ptr     = CString::new("F1").expect("CString::new failed").as_ptr() ;
    let keysym  = unsafe { xlib::XStringToKeysym( ptr ) };
    let keycode = unsafe { xlib::XKeysymToKeycode(display, keysym) };
    unsafe{
        xlib::XGrabKey(display, keycode as c_int, xlib::Mod1Mask, root, 1, xlib::GrabModeAsync, xlib:: GrabModeAsync );
    }


    unsafe{
        let mut event: xlib::XEvent = std::mem::zeroed() ;
        loop {
            xlib::XNextEvent(display, &mut event);
            println!("EVENT TYPE: {}", event.get_type() );
        }
    }


}

