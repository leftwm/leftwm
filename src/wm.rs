use x11_dl::xlib;

mod xwrap;
use xwrap::WaWindow;
use xwrap::XWrap;

fn main() {

    let xw = XWrap::new();

    //for window in windows {
    //    println!("WINDOW: {:#?} ", window);
    //}

    xw.init();
    let mut windows = WaWindow::find_all(&xw);

    loop {
        let raw_event = xw.get_next_event();
        match raw_event.get_type() {
            xlib::ClientMessage => { 
                let event = xlib::XClientMessageEvent::from(raw_event);
                let new_window = WaWindow::build(&xw, event.window);
                windows.push( new_window );
                println!("ClientMessage: {:#?} ", event);
            }
            xlib::ButtonPress => { 
                let event = xlib::XButtonPressedEvent::from(raw_event);
                println!("ButtonPress: {:#?} ", event);
            }
            xlib::EnterNotify => {
                let event = xlib::XEnterWindowEvent::from(raw_event);
                println!("EnterNotify: {:#?} ", event);
            },
            xlib::LeaveNotify => {
                let event = xlib::XLeaveWindowEvent::from(raw_event);
                println!("LeaveNotify: {:#?} ", event);
            },
            xlib::PropertyNotify => {
                let event = xlib::XPropertyEvent::from(raw_event);
                println!("PropertyNotify: {:#?} ", event);
            },

            xlib::MapNotify => {
                let event = xlib::XMappingEvent::from(raw_event);
                println!("MapNotify: {:#?} ", event);
            },

            //xlib::ButtonPress => d.field("button", &self.button),
            //xlib::ClientMessage => d.field("client_message", &self.client_message),
            
            xlib::KeyPress => println!("KeyPress"),
            xlib::KeyRelease => println!("KeyRelease"),
            xlib::ButtonRelease => println!("ButtonRelease"),
            //xlib::MotionNotify => println!("MotionNotify"),
            xlib::MotionNotify => {},
            xlib::FocusIn => println!("FocusIn"),
            xlib::FocusOut => println!("FocusOut"),
            xlib::KeymapNotify => println!("KeymapNotify"),
            xlib::Expose => println!("Expose"),
            xlib::GraphicsExpose => println!("GraphicsExpose"),
            xlib::NoExpose => println!("NoExpose"),
            xlib::VisibilityNotify => println!("VisibilityNotify"),
            xlib::CreateNotify => println!("CreateNotify"),
            xlib::DestroyNotify => println!("DestroyNotify"),
            xlib::UnmapNotify => println!("UnmapNotify"),
            xlib::MapRequest => println!("MapRequest"),
            xlib::ReparentNotify => println!("ReparentNotify"),
            xlib::ConfigureNotify => println!("ConfigureNotify"),
            xlib::ConfigureRequest => println!("ConfigureRequest"),
            xlib::GravityNotify => println!("GravityNotify"),
            xlib::ResizeRequest => println!("ResizeRequest"),
            xlib::CirculateNotify => println!("CirculateNotify"),
            xlib::CirculateRequest => println!("CirculateRequest"),
            xlib::SelectionClear => println!("SelectionClear"),
            xlib::SelectionRequest => println!("SelectionRequest"),
            xlib::SelectionNotify => println!("SelectionNotify"),
            xlib::ColormapNotify => println!("ColormapNotify"),
            xlib::MappingNotify => println!("MappingNotify"),
            xlib::GenericEvent => println!("GenericEvent"),

            _ => {
                println!("UNKNOWN EVENT: {}", raw_event.get_type() );
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
