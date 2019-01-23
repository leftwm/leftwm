use x11_dl::xlib;

mod xwrap;

fn main() {

    let xw = xwrap::XWrap::new();
    let windows = xwrap::WaWindow::find_all(&xw);

    for window in windows {
        println!("WINDOW: {:#?} ", window);
    }

    xw.init();
    loop {
        let event = xw.get_next_event();
        match event.get_type() {
            xlib::ClientMessage => { 
                let xclient = xlib::XClientMessageEvent::from(event);
                println!("EVENT: {:#?} ", xclient);
            }
            _ => {
                println!("UNKNOWN EVENT: ");
            }
        }
    }



    //let windows = xw.get_all_windows().unwrap();
    ////pub fn get_window_attrs(&self, window: xlib::Window) -> xlib::XWindowAttributes {
    ////let attrs: Vec<xlib::XWindowAttributes> = windows.into_iter().map(|w| xw.get_window_attrs(w) ).collect();
    //let names: Vec<String> = windows.into_iter().map(|w| xw.get_window_name(w) ).collect();

    //for name in names {
    //    println!("WINDOW: {} ", name);
    //}
    //println!("DONE!");


}



//fn run_it() {
//
//
//    // Load Xlib library.
//    //let xlib = xlib::Xlib::open().unwrap();
//    //let display = unsafe{ (xlib.XOpenDisplay)(ptr::null()) };
//    //let root = unsafe{ (xlib.XDefaultRootWindow)(display) };
//
//    //get_windows(xlib, display, root);
//
//    //let key = CString::new("F1").expect("CString::new failed");
//    //let keysym  = unsafe { (xlib.XStringToKeysym)( key.as_ptr() ) };
//    //let keycode = unsafe { (xlib.XKeysymToKeycode)(display, keysym) };
//    
//    
//    
//    
//    unsafe{
//        //(xlib.XGrabKey)(display, 
//        //                keycode as c_int,
//        //                xlib::AnyModifier,
//        //                root, 
//        //                0,
//        //                xlib::GrabModeAsync, 
//        //                xlib::GrabModeAsync);
//        
//        //(xlib.XGrabKeyboard)(display,
//        //                     root,
//        //                     0,
//        //                     xlib::GrabModeAsync, 
//        //                     xlib::GrabModeAsync, 
//        //                     xlib::CurrentTime
//        //                     );
//
//
//
//        //(xlib.XGrabButton)(display, 
//        //                   xlib::Button1 as u32,//xlib::Button1, 
//        //                   xlib::AnyModifier, //xlib::Mod1Mask, 
//        //                   root, 
//        //                   1, 
//        //                   xlib::ButtonPressMask as u32, 
//        //                   xlib::GrabModeAsync, 
//        //                   xlib::GrabModeAsync, 
//        //                   0 as c_ulong,
//        //                   0 as c_ulong);
//
//
//    }
//
//
//    //unsafe{
//    //    let mut event: xlib::XEvent = std::mem::zeroed();
//    //    let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
//    //    loop {
//
//
//    //        (xlib.XNextEvent)(display, &mut event);
//
//    //        match event.get_type() {
//    //            xlib::ButtonPress => {
//    //                let bev = xlib::XButtonEvent::from(event);
//    //                (xlib.XGetWindowAttributes)(display, bev.subwindow, &mut attrs);
//    //                println!("WINDOW: {} SUB:{}", bev.window, bev.subwindow);
//    //                //println!("WINDOW XY : {} {}    w:{}  h:{}", attrs.x, attrs.y, attrs.width, attrs.height);
//    //                //(xlib.XRaiseWindow)(display, bev.subwindow);
//    //                (xlib.XSetInputFocus)(display, bev.subwindow, xlib::RevertToNone, xlib::CurrentTime);
//    //            }
//    //            _ =>{}
//    //        }
//
//
//    //    }
//    //}
//
//
//
//}
