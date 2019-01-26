use x11_dl::xlib;
use super::Handle;
use super::Window;
use super::Manager;



pub fn dispatch(manager: &mut Manager, raw_event: xlib::XEvent){

    match raw_event.get_type() {

        xlib::ClientMessage => { 
            let event = xlib::XClientMessageEvent::from(raw_event);
            let w = Window{ 
                handle: Handle::XlibHandle(event.window)
            };
            manager.on_new_window(w);
        }

        //xlib::ButtonPress => { 
        //    let event = xlib::XButtonPressedEvent::from(raw_event);
        //    println!("ButtonPress: {:#?} ", event);
        //}
        //xlib::EnterNotify => {
        //    let event = xlib::XEnterWindowEvent::from(raw_event);
        //    println!("EnterNotify: {:#?} ", event);
        //},
        //xlib::LeaveNotify => {
        //    let event = xlib::XLeaveWindowEvent::from(raw_event);
        //    println!("LeaveNotify: {:#?} ", event);
        //},
        //xlib::PropertyNotify => {
        //    let event = xlib::XPropertyEvent::from(raw_event);
        //    println!("PropertyNotify: {:#?} ", event);
        //},

        //xlib::MapNotify => {
        //    let event = xlib::XMappingEvent::from(raw_event);
        //    println!("MapNotify: {:#?} ", event);
        //},


        //xlib::KeyPress => println!("KeyPress"),
        //xlib::KeyRelease => println!("KeyRelease"),
        //xlib::ButtonRelease => println!("ButtonRelease"),
        //xlib::MotionNotify => {},
        //xlib::FocusIn => println!("FocusIn"),
        //xlib::FocusOut => println!("FocusOut"),
        //xlib::KeymapNotify => println!("KeymapNotify"),
        //xlib::Expose => println!("Expose"),
        //xlib::GraphicsExpose => println!("GraphicsExpose"),
        //xlib::NoExpose => println!("NoExpose"),
        //xlib::VisibilityNotify => println!("VisibilityNotify"),
        //xlib::CreateNotify => println!("CreateNotify"),
        //xlib::DestroyNotify => println!("DestroyNotify"),
        //xlib::UnmapNotify => println!("UnmapNotify"),
        //xlib::MapRequest => println!("MapRequest"),
        //xlib::ReparentNotify => println!("ReparentNotify"),
        //xlib::ConfigureNotify => println!("ConfigureNotify"),
        //xlib::ConfigureRequest => println!("ConfigureRequest"),
        //xlib::GravityNotify => println!("GravityNotify"),
        //xlib::ResizeRequest => println!("ResizeRequest"),
        //xlib::CirculateNotify => println!("CirculateNotify"),
        //xlib::CirculateRequest => println!("CirculateRequest"),
        //xlib::SelectionClear => println!("SelectionClear"),
        //xlib::SelectionRequest => println!("SelectionRequest"),
        //xlib::SelectionNotify => println!("SelectionNotify"),
        //xlib::ColormapNotify => println!("ColormapNotify"),
        //xlib::MappingNotify => println!("MappingNotify"),
        //xlib::GenericEvent => println!("GenericEvent"),

        _ => {
            //println!("UNKNOWN EVENT: {}", raw_event.get_type() );
        }


    }
}

