
use x11_dl::xlib;
use std::os::raw::{ c_int, c_ulong };
use std::ffi::{ CString };
use std::ptr;

fn main() {

    // Load Xlib library.
    let xlib = xlib::Xlib::open().unwrap();
    let display = unsafe{ (xlib.XOpenDisplay)(ptr::null()) };
    let screen = unsafe{ (xlib.XDefaultScreen)(display) };
    let root = unsafe{ (xlib.XRootWindow)(display, screen) };

    let ptr     = CString::new("F1").expect("CString::new failed").as_ptr() ;
    let keysym = unsafe { (xlib.XStringToKeysym)(ptr) };
    let keycode = unsafe { (xlib.XKeysymToKeycode)(display, keysym) };
    unsafe{
        //(xlib.XGrabKey)(display, keycode as c_int, xlib::ShiftMask, root, 0, xlib::GrabModeAsync, xlib:: GrabModeAsync );
        (xlib.XGrabButton)(display, 
                           xlib::Button1, 
                           xlib::ShiftMask, 
                           root, 
                           0, 
                           xlib::ButtonPressMask as u32, 
                           //xlib::NoEventMask as u32, 
                           xlib::GrabModeAsync, 
                           xlib::GrabModeAsync,0 as c_ulong, 0 as c_ulong);
    }


    unsafe{
        let mut event: xlib::XEvent = std::mem::zeroed() ;
        loop {
            (xlib.XNextEvent)(display, &mut event);
            println!("EVENT TYPE: {}", event.get_type() );
        }
    }


}

