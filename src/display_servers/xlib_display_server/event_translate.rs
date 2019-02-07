use x11_dl::xlib;
use super::utils::window::*;
use super::event_queue::EventQueueItem;
use super::XWrap;



pub fn from_xevent(xw: &XWrap, raw_event: xlib::XEvent) -> Option<EventQueueItem>{

    match raw_event.get_type() {

        // new window is created
        xlib::MapRequest => {
            let event = xlib::XMapRequestEvent::from(raw_event);
            let name = xw.get_window_name(event.window);
            let w = Window::new( WindowHandle::XlibHandle(event.window), name );
            Some( EventQueueItem::WindowCreate(w) )
        },

        // window is deleted
        xlib::UnmapNotify => {
            //println!("UnmapNotify");
            let event = xlib::XUnmapEvent::from(raw_event);
            let h = WindowHandle::XlibHandle(event.window);
            Some( EventQueueItem::WindowDelete(h) )
        },

        // window is deleted
        xlib::DestroyNotify => {
            //println!("DestroyNotify");
            let event = xlib::XDestroyWindowEvent::from(raw_event);
            let h = WindowHandle::XlibHandle(event.window);
            Some( EventQueueItem::WindowDelete(h) )
        },
        
        //xlib::ClientMessage => { 
        //    let event = xlib::XClientMessageEvent::from(raw_event);
        //    println!("ClientMessage: {:#?} ", event);
        //}

        //xlib::ButtonPress => { 
        //    let event = xlib::XButtonPressedEvent::from(raw_event);
        //    println!("ButtonPress: {:#?} ", event);
        //    None
        //}
        //xlib::EnterNotify => {
        //    let event = xlib::XEnterWindowEvent::from(raw_event);
        //    println!("EnterNotify: {:#?} ", event);
        //    None
        //},
        //xlib::LeaveNotify => {
        //    let event = xlib::XLeaveWindowEvent::from(raw_event);
        //    println!("LeaveNotify: {:#?} ", event);
        //    None
        //},
        //xlib::PropertyNotify => {
        //    let event = xlib::XPropertyEvent::from(raw_event);
        //    println!("PropertyNotify: {:#?} ", event);
        //    None
        //},

        //xlib::MapNotify => {
        //    let event = xlib::XMappingEvent::from(raw_event);
        //    println!("MapNotify: {:#?} ", event);
        //    None
        //},


        //xlib::KeyPress => {
        //    println!("KeyPress");
        //    None
        //},
        //xlib::KeyRelease => {
        //    println!("KeyRelease");
        //    None
        //},
        //xlib::ButtonRelease => {
        //    println!("ButtonRelease");
        //    None
        //},
        //xlib::MotionNotify => {
        //    {};
        //    None
        //},
        //xlib::FocusIn => {
        //    println!("FocusIn");
        //    None
        //},
        //xlib::FocusOut => {
        //    println!("FocusOut");
        //    None
        //},
        //xlib::KeymapNotify => {
        //    println!("KeymapNotify");
        //    None
        //},
        //xlib::Expose => {
        //    println!("Expose");
        //    None
        //},
        //xlib::GraphicsExpose => {
        //    println!("GraphicsExpose");
        //    None
        //},
        //xlib::NoExpose => {
        //    println!("NoExpose");
        //    None
        //},
        //xlib::VisibilityNotify => {
        //    println!("VisibilityNotify");
        //    None
        //},
        //xlib::CreateNotify => {
        //    println!("CreateNotify");
        //    None
        //},
        //xlib::ReparentNotify => {
        //    println!("ReparentNotify");
        //    None
        //},
        //xlib::ConfigureNotify => {
        //    println!("ConfigureNotify");
        //    None
        //},
        //xlib::ConfigureRequest => {
        //    println!("ConfigureRequest");
        //    None
        //},
        //xlib::GravityNotify => {
        //    println!("GravityNotify");
        //    None
        //},
        //xlib::ResizeRequest => {
        //    println!("ResizeRequest");
        //    None
        //},
        //xlib::CirculateNotify => {
        //    println!("CirculateNotify");
        //    None
        //},
        //xlib::CirculateRequest => {
        //    println!("CirculateRequest");
        //    None
        //},
        //xlib::SelectionClear => {
        //    println!("SelectionClear");
        //    None
        //},
        //xlib::SelectionRequest => {
        //    println!("SelectionRequest");
        //    None
        //},
        //xlib::SelectionNotify => {
        //    println!("SelectionNotify");
        //    None
        //},
        //xlib::ColormapNotify => {
        //    println!("ColormapNotify");
        //    None
        //},
        //xlib::MappingNotify => {
        //    println!("MappingNotify");
        //    None
        //},
        //xlib::GenericEvent => {
        //    println!("GenericEvent");
        //    None
        //},

        _ => {
            None
            //println!("UNKNOWN EVENT: {}", raw_event.get_type() );
        }


    }
}

