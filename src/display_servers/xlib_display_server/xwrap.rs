use super::event_queue;
use super::utils;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_uint};
use std::ptr;
use std::slice;
use utils::window::WindowHandle;
use x11_dl::keysym;
use x11_dl::xlib;
//type ErrorCallback = extern "C" fn (*mut xlib::Display, *mut xlib::XErrorEvent) -> c_int;

pub struct XWrap {
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
    q: event_queue::EventQueue,
}

impl XWrap {
    pub fn new() -> XWrap {
        let xlib = xlib::Xlib::open().unwrap();
        let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
        assert!(!display.is_null(), "Null pointer in display");
        let q = event_queue::new();
        let xw = XWrap { xlib, display, q };

        ////On xlib error
        //let error_q = xw.q.clone();
        //let lam :ErrorCallback =  |_: *mut xlib::Display, er: *mut xlib::XErrorEvent|  {
        //    let _ = error_q.lock();
        //    0
        //};

        extern "C" fn on_error_from_xlib(
            _: *mut xlib::Display,
            er: *mut xlib::XErrorEvent,
        ) -> c_int {
            let err = unsafe { *er };
            //ignore bad window errors
            if err.error_code == xlib::BadWindow {
                return 0;
            }
            1
        }

        unsafe {
            (xw.xlib.XSetErrorHandler)(Some(on_error_from_xlib));
        };
        xw
    }

    //returns all the screens the display
    pub fn get_screens(&self) -> Vec<xlib::Screen> {
        let mut screens = Vec::new();
        let screen_count = unsafe { (self.xlib.XScreenCount)(self.display) };
        for screen_num in 0..(screen_count) {
            let screen = unsafe { *(self.xlib.XScreenOfDisplay)(self.display, screen_num) };
            screens.push(screen);
        }
        screens
    }

    //returns all the roots the display
    pub fn get_roots(&self) -> Vec<xlib::Window> {
        self.get_screens()
            .into_iter()
            .map(|mut s| unsafe { (self.xlib.XRootWindowOfScreen)(&mut s) })
            .collect()
    }

    //returns all the windows under a root windows
    pub fn get_windows_for_root<'w>(
        &self,
        root: xlib::Window,
    ) -> Result<&'w [xlib::Window], String> {
        unsafe {
            let mut root_return: xlib::Window = std::mem::zeroed();
            let mut parent_return: xlib::Window = std::mem::zeroed();
            let mut array: *mut xlib::Window = std::mem::zeroed();
            let mut length: c_uint = std::mem::zeroed();
            let status: xlib::Status = (self.xlib.XQueryTree)(
                self.display,
                root,
                &mut root_return,
                &mut parent_return,
                &mut array,
                &mut length,
            );
            let windows: &[xlib::Window] = slice::from_raw_parts(array, length as usize);
            match status {
                0 /* XcmsFailure */ => { Err("Could not load list of windows".to_string() ) }
                1 /* XcmsSuccess */ => { Ok(windows) }
                2 /* XcmsSuccessWithCompression */ => { Ok(windows) }
                _ => { Err("Unknown return status".to_string() ) }
            }
        }
    }

    pub fn get_all_windows(&self) -> Result<Vec<xlib::Window>, String> {
        let mut all = Vec::new();
        for root in self.get_roots() {
            match self.get_windows_for_root(root) {
                Ok(some_windows) => {
                    for w in some_windows {
                        all.push(*w);
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Ok(all)
    }

    pub fn get_window_attrs(&self, window: xlib::Window) -> Result<xlib::XWindowAttributes, ()> {
        let mut attrs: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        let status = unsafe { (self.xlib.XGetWindowAttributes)(self.display, window, &mut attrs) };
        if status == 0 {
            return Err(());
        }
        Ok(attrs)
    }

    pub fn get_parent_window(&self, window: xlib::Window) -> Option<xlib::Window> {
        unsafe {
            let mut root_return: xlib::Window = std::mem::zeroed();
            let mut parent_return: xlib::Window = std::mem::zeroed();
            let mut array: *mut xlib::Window = std::mem::zeroed();
            let mut length: c_uint = std::mem::zeroed();
            let status: xlib::Status = (self.xlib.XQueryTree)(
                self.display,
                window,
                &mut root_return,
                &mut parent_return,
                &mut array,
                &mut length,
            );
            //take ownership of the array
            let _: &[xlib::Window] = slice::from_raw_parts(array, length as usize);
            match status {
                0 /* XcmsFailure */ => { None }
                1 /* XcmsSuccess */ => { Some(parent_return) }
                2 /* XcmsSuccessWithCompression */ => { Some(parent_return) }
                _ => { None }
            }
        }
    }

    pub fn update_window(&self, window: &utils::window::Window) {
        let mut changes = xlib::XWindowChanges {
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            border_width: window.border,
            sibling: 0,    //not unlocked
            stack_mode: 0, //not unlocked
        };
        if let WindowHandle::XlibHandle(h) = window.handle {
            println!("in xwrap update window: {:#?} ", changes);
            let unlock =
                xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth;
            unsafe {
                (self.xlib.XConfigureWindow)(self.display, h, u32::from(unlock), &mut changes);
                (self.xlib.XSync)(self.display, 0);
                let rw: u32 = window.width as u32;
                let rh: u32 = window.height as u32;
                (self.xlib.XMoveResizeWindow)(self.display, h, window.x, window.y, rw, rh);
            }
            self.send_config(window);
            self.update_visable(window);
        }
    }

    pub fn update_visable(&self, window: &utils::window::Window) {
        use utils::window::WindowHandle;
        if let WindowHandle::XlibHandle(handle) = window.handle {
            unsafe {
                if window.visable {
                    (self.xlib.XMapWindow)(self.display, handle);
                } else {
                    (self.xlib.XUnmapWindow)(self.display, handle);
                }
            }
        }
    }

    /**
     * used to send and XConfigureEvent for a changed window to the xserver
     */
    pub fn send_config(&self, window: &utils::window::Window) {
        use utils::window::WindowHandle;
        if let WindowHandle::XlibHandle(handle) = window.handle {
            let config = xlib::XConfigureEvent {
                type_: xlib::ConfigureNotify,
                serial: 0, //not used
                send_event: 0,
                display: self.display,
                event: handle,
                window: handle,
                x: window.x,
                y: window.y,
                width: window.width,
                height: window.height,
                border_width: window.border,
                above: 0,
                override_redirect: 0,
            };
            unsafe {
                let mut event: xlib::XEvent = xlib::XConfigureEvent::into(config);
                (self.xlib.XSendEvent)(
                    self.display,
                    handle,
                    0,
                    xlib::StructureNotifyMask,
                    &mut event,
                );
            }
        }

        //ce.type = ConfigureNotify;
        //ce.display = dpy;
        //ce.event = c->win;
        //ce.window = c->win;
        //ce.x = c->x;
        //ce.y = c->y;
        //ce.width = c->w;
        //ce.height = c->h;
        //ce.border_width = c->bw;
        //ce.above = None;
        //ce.override_redirect = False;
        //XSendEvent(dpy, c->win, False, StructureNotifyMask, (XEvent *)&ce);
    }

    pub fn get_transient_for(&self, window: xlib::Window) -> Option<xlib::Window> {
        unsafe {
            let mut transient: xlib::Window = std::mem::zeroed();
            let status: c_int =
                (self.xlib.XGetTransientForHint)(self.display, window, &mut transient);
            if status > 0 {
                Some(transient)
            } else {
                None
            }
        }
    }

    pub fn get_window_name(&self, window: xlib::Window) -> Option<String> {
        let c_string = unsafe {
            let mut ptr: *mut c_char = std::mem::zeroed();
            let status: c_int = (self.xlib.XFetchName)(self.display, window, &mut ptr);
            if status == 0 {
                return None;
            }
            CString::from_raw(ptr)
        };
        match c_string.into_string() {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    ////get the WMName of a window
    //pub fn get_window_title(&self, window: xlib::Window) -> Result<String, ()> {
    //    unsafe{
    //        let mut ptr : *mut *mut c_char = std::mem::zeroed();
    //        let mut ptr_len: c_int = 0;
    //        let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
    //        let status :c_int = (self.xlib.XGetTextProperty)(
    //            self.display,
    //            window,
    //            &mut text_prop,
    //            2);
    //        if status == 0 { return Err( () ) }
    //        (self.xlib.XTextPropertyToStringList)(
    //            &mut text_prop,
    //            &mut ptr,
    //            &mut ptr_len );
    //        let raw: &[*mut c_char] = slice::from_raw_parts(ptr, ptr_len as usize);
    //        for i in 0..ptr_len {
    //            if let Ok(s) = CString::from_raw(*ptr).into_string() {
    //                return Ok(s)
    //            }
    //        }
    //    };
    //    return Err(())
    //}

    ////get the WMName of a window
    //pub fn get_wmname(&self, window: xlib::Window) -> Result<String, ()> {
    //    unsafe{
    //        let mut ptr : *mut *mut c_char = std::mem::zeroed();
    //        let mut ptr_len: c_int = 0;
    //        let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
    //        let status :c_int = (self.xlib.XGetWMName)(
    //            self.display,
    //            window,
    //            &mut text_prop );
    //        if status == 0 { return Err( () ) }
    //        (self.xlib.XTextPropertyToStringList)(
    //            &mut text_prop,
    //            &mut ptr,
    //            &mut ptr_len );
    //        let raw: &[*mut c_char] = slice::from_raw_parts(ptr, ptr_len as usize);
    //        for i in 0..ptr_len {
    //            if let Ok(s) = CString::from_raw(*ptr).into_string() {
    //                return Ok(s)
    //            }
    //        }
    //    };
    //    return Err(())
    //}

    pub fn subscribe_to_event(&self, window: xlib::Window, mask: c_long) {
        unsafe {
            (self.xlib.XSelectInput)(self.display, window, mask);
        }
    }

    pub fn subscribe_to_window_events(&self, window: &utils::window::Window) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            let mask = xlib::EnterWindowMask
                | xlib::FocusChangeMask
                | xlib::PropertyChangeMask
                | xlib::StructureNotifyMask;
            self.subscribe_to_event(handle, mask);
            //might want to grab buttons here???
        }
    }

    pub fn grab_keys(&self, root: xlib::Window, keysym: u32, modifiers: u32) {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, u64::from(keysym)) };
        //grb the keys with and without numlock (Mod2)
        let mods: Vec<u32> = vec![modifiers, modifiers | xlib::Mod2Mask];
        for m in mods {
            unsafe {
                (self.xlib.XGrabKey)(
                    self.display,
                    //xlib::AnyKey,
                    i32::from(code),
                    //xlib::AnyModifier,
                    m,
                    root,
                    1,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
            }
        }
    }

    pub fn init(&self) {
        let root_event_mask: c_long = xlib::ButtonPressMask
            | xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::PointerMotionMask
            | xlib::EnterWindowMask
            | xlib::LeaveWindowMask
            | xlib::StructureNotifyMask
            | xlib::PropertyChangeMask;
        for root in self.get_roots() {
            self.subscribe_to_event(root, root_event_mask);
            unsafe {
                (self.xlib.XUngrabKey)(self.display, xlib::AnyKey, xlib::AnyModifier, root);
                self.grab_keys(root, keysym::XK_KP_1, xlib::Mod1Mask);
                self.grab_keys(root, keysym::XK_KP_2, xlib::Mod1Mask);
                //self.grab_keys(root, keysym::XK_KP_1, xlib::Mod1Mask);
                //self.grab_keys(root, keysym::XK_KP_2, xlib::Mod2Mask);
            }
        }
        unsafe {
            (self.xlib.XSync)(self.display, 0);
        }
    }

    pub fn get_next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe { std::mem::uninitialized() };
        unsafe {
            (self.xlib.XNextEvent)(self.display, &mut event);
        };
        event
    }
}
