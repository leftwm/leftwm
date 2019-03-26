use super::utils;
use super::xatom::XAtom;
use super::xcursor::XCursor;
use super::Config;
use super::Screen;
use super::Window;
use super::WindowHandle;
use crate::models::DockArea;
use crate::models::WindowChange;
use crate::models::WindowType;
use crate::models::XYHW;
use crate::utils::xkeysym_lookup::ModMask;
use crate::DisplayEvent;
use crate::DisplayServerMode;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};
use std::ptr;
use std::slice;
use x11_dl::xlib;

//type WindowStateConst = u8;
//const WITHDRAWN_STATE: WindowStateConst = 0;
//const NORMAL_STATE: WindowStateConst = 1;
//const ICONIC_STATE: WindowStateConst = 2;
const MAX_PROPERTY_VALUE_LEN: i64 = 4096;

const BUTTONMASK: i64 = xlib::ButtonPressMask | xlib::ButtonReleaseMask;
const MOUSEMASK: i64 = BUTTONMASK | xlib::PointerMotionMask;

pub struct Colors {
    normal: c_ulong,
    active: c_ulong,
    //urgent: c_ulong,
}

pub struct XWrap {
    pub xlib: xlib::Xlib,
    pub display: *mut xlib::Display,
    root: xlib::Window,
    pub atoms: XAtom,
    cursors: XCursor,
    colors: Colors,
    pub tags: Vec<String>,
    pub mode: DisplayServerMode,
    pub mod_key_mask: ModMask,
    pub mode_origin: (i32, i32),
}

impl XWrap {
    pub fn new() -> XWrap {
        let xlib = xlib::Xlib::open().unwrap();
        let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
        assert!(!display.is_null(), "Null pointer in display");

        let atoms = XAtom::new(&xlib, display);
        let cursors = XCursor::new(&xlib, display);
        let root = unsafe { (xlib.XDefaultRootWindow)(display) };
        let colors = Colors {
            normal: 0,
            active: 0,
        };

        let xw = XWrap {
            xlib,
            display,
            root,
            atoms,
            cursors,
            colors,
            tags: vec![],
            mode: DisplayServerMode::NormalMode,
            mod_key_mask: 0,
            mode_origin: (0, 0),
        };

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
    pub fn get_screens(&self) -> Vec<Screen> {
        use x11_dl::xinerama::XineramaScreenInfo;
        use x11_dl::xinerama::Xlib;
        let xlib = Xlib::open().unwrap();
        let xinerama = unsafe { (xlib.XineramaIsActive)(self.display) } > 0;
        if xinerama {
            let root = self.get_default_root_handle();
            let mut screen_count = 0;
            let info_array_raw =
                unsafe { (xlib.XineramaQueryScreens)(self.display, &mut screen_count) };
            //take ownership of the array
            let xinerama_infos: &[XineramaScreenInfo] =
                unsafe { slice::from_raw_parts(info_array_raw, screen_count as usize) };
            xinerama_infos
                .iter()
                .map(|i| {
                    let mut s = Screen::from(i);
                    s.root = root.clone();
                    s
                })
                .collect()
        } else {
            //NON-XINERAMA
            let roots: Vec<xlib::XWindowAttributes> = self
                .get_roots()
                .iter()
                .map(|w| self.get_window_attrs(*w).unwrap())
                .collect();
            roots.iter().map(|w| Screen::from(w)).collect()
        }
    }

    //returns all the screens the display
    pub fn get_xscreens(&self) -> Vec<xlib::Screen> {
        let mut screens = Vec::new();
        let screen_count = unsafe { (self.xlib.XScreenCount)(self.display) };
        for screen_num in 0..(screen_count) {
            let screen = unsafe { *(self.xlib.XScreenOfDisplay)(self.display, screen_num) };
            screens.push(screen);
        }
        screens
    }

    //returns all the screens the display
    pub fn get_default_root_handle(&self) -> WindowHandle {
        WindowHandle::XlibHandle(self.get_default_root())
    }

    pub fn get_default_root(&self) -> xlib::Window {
        self.root
    }

    //returns all the roots the display
    pub fn get_roots(&self) -> Vec<xlib::Window> {
        self.get_xscreens()
            .into_iter()
            .map(|mut s| unsafe { (self.xlib.XRootWindowOfScreen)(&mut s) })
            .collect()
    }

    pub fn keycode_to_keysym(&self, keycode: u32) -> utils::xkeysym_lookup::XKeysym {
        let sym = unsafe { (self.xlib.XKeycodeToKeysym)(self.display, keycode as u8, 0) };
        sym as u32
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

    pub fn get_atom_prop_value(
        &self,
        window: xlib::Window,
        prop: xlib::Atom,
    ) -> Option<xlib::Atom> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::uninitialized() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                prop,
                0,
                MAX_PROPERTY_VALUE_LEN / 4,
                xlib::False,
                xlib::XA_ATOM,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut nitems_return,
                &mut prop_return,
            );
            if status == xlib::Success as i32 && !prop_return.is_null() {
                let atom = *(prop_return as *const xlib::Atom);
                return Some(atom);
            }
            None
        }
    }

    pub fn get_window_type(&self, window: xlib::Window) -> WindowType {
        if let Some(value) = self.get_atom_prop_value(window, self.atoms.NetWMWindowType) {
            if value == self.atoms.NetWMWindowTypeDock {
                return WindowType::Dock;
            }
            if value == self.atoms.NetWMWindowTypeDialog {
                return WindowType::Dialog;
            }
        }
        WindowType::Normal
    }

    /* EWMH support used for bars such as polybar */
    pub fn init_destops_hints(&self, tags: Vec<String>) {
        let tag_length = tags.len();
        //set the number of desktop
        let data = vec![tag_length as u32];
        self.set_desktop_prop(&data, self.atoms.NetNumberOfDesktops);
        //set a current desktop
        let data = vec![0 as u32, xlib::CurrentTime as u32];
        self.set_desktop_prop(&data, self.atoms.NetCurrentDesktop);
        //set desktop names
        let mut text: xlib::XTextProperty = unsafe { std::mem::uninitialized() };
        unsafe {
            let mut clist_tags: Vec<*mut c_char> = tags
                .iter()
                .map(|x| CString::new(x.clone()).unwrap().into_raw())
                .collect();
            let ptr = clist_tags.as_mut_ptr();
            (self.xlib.Xutf8TextListToTextProperty)(
                self.display,
                ptr,
                clist_tags.len() as i32,
                xlib::XUTF8StringStyle,
                &mut text,
            );
            std::mem::forget(clist_tags);
            (self.xlib.XSetTextProperty)(
                self.display,
                self.get_default_root(),
                &mut text,
                self.atoms.NetDesktopNames,
            );
        }
        //set a viewport
        let data = vec![0 as u32, 0 as u32];
        self.set_desktop_prop(&data, self.atoms.NetDesktopViewport);
    }

    fn set_desktop_prop(&self, data: &Vec<u32>, atom: c_ulong) {
        let xdata = data.clone();
        unsafe {
            (self.xlib.XChangeProperty)(
                self.display,
                self.get_default_root(),
                atom,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                xdata.as_ptr() as *const u8,
                data.len() as i32,
            );
            std::mem::forget(xdata);
        }
    }

    pub fn set_current_viewport(&self, tags: Vec<&String>) {
        let mut indexes: Vec<u32> = vec![];
        for tag in tags {
            for (i, mytag) in self.tags.iter().enumerate() {
                if tag.contains(mytag) {
                    indexes.push(i as u32);
                }
            }
        }
        if indexes.is_empty() {
            indexes.push(0)
        }
        self.set_desktop_prop(&indexes, self.atoms.NetDesktopViewport);
        //self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    pub fn set_current_desktop(&self, current_tags: &String) {
        let mut indexes: Vec<u32> = vec![];
        for (i, tag) in self.tags.iter().enumerate() {
            if current_tags.contains(tag) {
                indexes.push(i as u32);
            }
        }
        if indexes.is_empty() {
            indexes.push(0)
        }
        self.set_desktop_prop(&indexes, self.atoms.NetCurrentDesktop);
    }

    //pub fn get_parent_window(&self, window: xlib::Window) -> Option<xlib::Window> {
    //    unsafe {
    //        let mut root_return: xlib::Window = std::mem::zeroed();
    //        let mut parent_return: xlib::Window = std::mem::zeroed();
    //        let mut array: *mut xlib::Window = std::mem::zeroed();
    //        let mut length: c_uint = std::mem::zeroed();
    //        let status: xlib::Status = (self.xlib.XQueryTree)(
    //            self.display,
    //            window,
    //            &mut root_return,
    //            &mut parent_return,
    //            &mut array,
    //            &mut length,
    //        );
    //        //take ownership of the array
    //        let _: &[xlib::Window] = slice::from_raw_parts(array, length as usize);
    //        match status {
    //            0 /* XcmsFailure */ => { None }
    //            1 /* XcmsSuccess */ => { Some(parent_return) }
    //            2 /* XcmsSuccessWithCompression */ => { Some(parent_return) }
    //            _ => { None }
    //        }
    //    }
    //}

    pub fn update_window(&self, window: &Window, is_focused: bool) {
        if let WindowHandle::XlibHandle(h) = window.handle {
            if window.visable() {
                let mut changes = xlib::XWindowChanges {
                    x: window.x(),
                    y: window.y(),
                    width: window.width(),
                    height: window.height(),
                    border_width: window.border,
                    sibling: 0,    //not unlocked
                    stack_mode: 0, //not unlocked
                };
                let unlock =
                    xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth;
                unsafe {
                    (self.xlib.XConfigureWindow)(self.display, h, u32::from(unlock), &mut changes);
                    (self.xlib.XSync)(self.display, 0);
                    let rw: u32 = window.width() as u32;
                    let rh: u32 = window.height() as u32;
                    (self.xlib.XMoveResizeWindow)(self.display, h, window.x(), window.y(), rw, rh);

                    let color: c_ulong = match is_focused {
                        true => self.colors.active,
                        false => self.colors.normal,
                    };
                    (self.xlib.XSetWindowBorder)(self.display, h, color);
                }
                self.send_config(window);
            } else {
                unsafe {
                    //if not visable x is <---- way over there <----
                    (self.xlib.XMoveWindow)(self.display, h, window.width() * -2, window.y());
                }
            }
        }
    }

    //this code is ran one time when a window is added to the managers list of windows
    pub fn setup_managed_window(&self, h: WindowHandle) -> Option<DisplayEvent> {
        self.subscribe_to_window_events(&h);
        if let WindowHandle::XlibHandle(handle) = h {
            //make sure the window is mapped
            unsafe {
                (self.xlib.XMapWindow)(self.display, handle);
            }

            unsafe {
                //let Xlib know we are managing this window
                let list = vec![handle];
                (self.xlib.XChangeProperty)(
                    self.display,
                    handle,
                    self.atoms.NetClientList,
                    xlib::XA_WINDOW,
                    32,
                    xlib::PropModeAppend,
                    list.as_ptr() as *const u8,
                    1,
                );
                std::mem::forget(list);
            }

            unsafe {
                (self.xlib.XSync)(self.display, 0);
            }

            if self.get_window_type(handle) == WindowType::Dock {
                if let Some(dock_area) = self.get_window_strut_array(handle) {
                    let dems = self.screens_area_dimensions();
                    if let Some(xywh) = dock_area.as_xyhw(dems.0, dems.1) {
                        let mut change = WindowChange::new(h);
                        change.floating = Some(xywh.clone());
                        change.type_ = Some(WindowType::Dock);
                        return Some(DisplayEvent::WindowChange(change));
                    }
                }
            }
        }
        None
    }

    pub fn screens_area_dimensions(&self) -> (i32, i32) {
        let mut height = 0;
        let mut width = 0;
        for s in self.get_screens() {
            height = std::cmp::max(height, s.height + s.y);
            width = std::cmp::max(width, s.width + s.x);
        }
        (height, width)
    }

    pub fn get_window_strut_array(&self, window: xlib::Window) -> Option<DockArea> {
        let mut format_return: i32 = 0;
        let mut nitems_return: c_ulong = 0;
        let mut type_return: xlib::Atom = 0;
        let mut bytes_after_return: xlib::Atom = 0;
        let mut prop_return: *mut c_uchar = unsafe { std::mem::uninitialized() };
        unsafe {
            let status = (self.xlib.XGetWindowProperty)(
                self.display,
                window,
                self.atoms.NetWMStrutPartial,
                0,
                MAX_PROPERTY_VALUE_LEN,
                xlib::False,
                xlib::XA_CARDINAL,
                &mut type_return,
                &mut format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return,
            );
            if status == xlib::Success as i32 {
                let array_ptr = prop_return as *const i64;
                let slice = slice::from_raw_parts(array_ptr, nitems_return as usize);
                if slice.len() == 12 {
                    return Some(DockArea::from(slice));
                }
                None
            } else {
                None
            }
        }
    }

    //this code is ran one when a window is destoryed
    pub fn teardown_managed_window(&self, h: WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            unsafe {
                (self.xlib.XGrabServer)(self.display);
                //(self.xlib.XSetErrorHandler)(xerrordummy);
                (self.xlib.XUngrabButton)(
                    self.display,
                    xlib::AnyButton as u32,
                    xlib::AnyModifier,
                    handle,
                );
                //self.set_window_state(&h, WITHDRAWN_STATE);
                (self.xlib.XSync)(self.display, 0);
                //(self.xlib.XSetErrorHandler)(xerror);
                (self.xlib.XUngrabServer)(self.display);
            }
            //self.set_window_state(&h, NORMAL_STATE);
        }
    }

    //fn set_window_state(&self, handle: &WindowHandle, state: WindowState) {
    //    if let WindowHandle::XlibHandle(handle) = handle {
    //        unsafe {
    //            let list = vec![state as u32, 0 as u32];
    //            (self.xlib.XChangeProperty)(
    //                self.display,
    //                *handle,
    //                self.atoms.WMState,
    //                self.atoms.WMState,
    //                32,
    //                xlib::PropModeReplace,
    //                list.as_ptr() as *const u8,
    //                2,
    //            );
    //            std::mem::forget(list);
    //        }
    //    }
    //}

    /**
     * used to send and XConfigureEvent for a changed window to the xserver
     */
    pub fn send_config(&self, window: &Window) {
        if let WindowHandle::XlibHandle(handle) = window.handle {
            let config = xlib::XConfigureEvent {
                type_: xlib::ConfigureNotify,
                serial: 0, //not used
                send_event: 0,
                display: self.display,
                event: handle,
                window: handle,
                x: window.x(),
                y: window.y(),
                width: window.width(),
                height: window.height(),
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
    }

    fn send_xevent_atom(&self, window: xlib::Window, atom: xlib::Atom) -> bool {
        if self.can_send_xevent_atom(window, atom) {
            let mut msg: xlib::XClientMessageEvent = unsafe { std::mem::uninitialized() };
            msg.type_ = xlib::ClientMessage;
            msg.window = window;
            msg.message_type = self.atoms.WMProtocols;
            msg.format = 32;
            msg.data.set_long(0, atom as i64);
            msg.data.set_long(1, xlib::CurrentTime as i64);
            let mut ev: xlib::XEvent = msg.into();
            unsafe { (self.xlib.XSendEvent)(self.display, window, 0, xlib::NoEventMask, &mut ev) };
            return true;
        }
        false
    }

    //return true if the underlying window exsepts this type of atom:protocal
    fn can_send_xevent_atom(&self, window: xlib::Window, atom: xlib::Atom) -> bool {
        unsafe {
            let mut array: *mut xlib::Atom = std::mem::zeroed();
            let mut length: c_int = std::mem::zeroed();
            let status: xlib::Status =
                (self.xlib.XGetWMProtocols)(self.display, window, &mut array, &mut length);
            let protocols: &[xlib::Atom] = slice::from_raw_parts(array, length as usize);
            status > 0 && protocols.contains(&atom)
        }
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

    pub fn move_to_top(&self, handle: WindowHandle) {
        if let WindowHandle::XlibHandle(window) = handle {
            unsafe {
                (self.xlib.XRaiseWindow)(self.display, window);
            }
        }
    }

    fn is_window_under_cursor(&self, window: xlib::Window) -> bool {
        if let Some(mouse) = self.get_pointer_location() {
            if let Ok(xyhw) = self.get_window_geometry(window) {
                return xyhw.contains_point(mouse.0, mouse.1);
            }
        }
        false
    }

    fn get_window_geometry(&self, window: xlib::Window) -> Result<XYHW, ()> {
        let mut root_return: xlib::Window = 0;
        let mut x_return: c_int = 0;
        let mut y_return: c_int = 0;
        let mut width_return: c_uint = 0;
        let mut height_return: c_uint = 0;
        let mut border_width_return: c_uint = 0;
        let mut depth_return: c_uint = 0;
        unsafe {
            let status = (self.xlib.XGetGeometry)(
                self.display,
                window,
                &mut root_return,
                &mut x_return,
                &mut y_return,
                &mut width_return,
                &mut height_return,
                &mut border_width_return,
                &mut depth_return,
            );
            if status == 0 {
                return Err(());
            }
        }
        Ok(XYHW {
            x: x_return,
            y: y_return,
            w: width_return as i32,
            h: height_return as i32,
        })
    }

    pub fn window_take_focus(&self, h: WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            if !self.is_window_under_cursor(handle) {
                return;
            }

            //tell the window to take focus
            self.send_xevent_atom(handle, self.atoms.WMTakeFocus);

            //mark this window as the NetActiveWindow
            unsafe {
                (self.xlib.XSetInputFocus)(
                    self.display,
                    handle,
                    xlib::RevertToPointerRoot,
                    xlib::CurrentTime,
                );
                let list = vec![handle];
                (self.xlib.XChangeProperty)(
                    self.display,
                    self.get_default_root(),
                    self.atoms.NetActiveWindow,
                    xlib::XA_WINDOW,
                    32,
                    xlib::PropModeReplace,
                    list.as_ptr() as *const c_uchar,
                    1,
                );
                std::mem::forget(list);
            }
            unsafe {
                //cleanup all old watches
                (self.xlib.XUngrabButton)(
                    self.display,
                    xlib::AnyButton as u32,
                    xlib::AnyModifier,
                    handle,
                ); //cleanup

                //just watchout for these mouse combos se we can act on them
                self.grab_buttons(handle, xlib::Button1, self.mod_key_mask);
                self.grab_buttons(handle, xlib::Button1, self.mod_key_mask | xlib::ShiftMask);
                self.grab_buttons(handle, xlib::Button3, self.mod_key_mask);
                self.grab_buttons(handle, xlib::Button3, self.mod_key_mask | xlib::ShiftMask);
            }
        }
    }

    pub fn kill_window(&self, h: WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = h {
            self.send_xevent_atom(handle, self.atoms.WMDelete);
            //cleanup
            unsafe {
                (self.xlib.XGrabServer)(self.display);
                (self.xlib.XSetCloseDownMode)(self.display, xlib::DestroyAll);
                (self.xlib.XKillClient)(self.display, handle);
                (self.xlib.XSync)(self.display, xlib::False);
                (self.xlib.XUngrabServer)(self.display);
            }
        }
    }

    pub fn subscribe_to_event(&self, window: xlib::Window, mask: c_long) {
        unsafe {
            (self.xlib.XSelectInput)(self.display, window, mask);
        }
    }

    pub fn subscribe_to_window_events(&self, handle: &WindowHandle) {
        if let WindowHandle::XlibHandle(handle) = handle {
            let mask = xlib::EnterWindowMask
                | xlib::FocusChangeMask
                | xlib::PropertyChangeMask
                | xlib::StructureNotifyMask;
            self.subscribe_to_event(handle.clone(), mask);
        }
    }

    pub fn get_pointer_location(&self) -> Option<(i32, i32)> {
        let mut root: xlib::Window = 0;
        let mut window: xlib::Window = 0;
        let mut root_x: c_int = 0;
        let mut root_y: c_int = 0;
        let mut win_x: c_int = 0;
        let mut win_y: c_int = 0;
        let mut state: c_uint = 0;
        unsafe {
            let success = (self.xlib.XQueryPointer)(
                self.display,
                self.root,
                &mut root,
                &mut window,
                &mut root_x,
                &mut root_y,
                &mut win_x,
                &mut win_y,
                &mut state,
            );
            if success > 0 {
                return Some((root_x, root_y));
            }
        }
        None
    }

    pub fn get_hint_sizing(&self, window: xlib::Window) -> Option<xlib::XSizeHints> {
        let mut xsize: xlib::XSizeHints = unsafe { std::mem::uninitialized() };
        let mut msize: c_long = xlib::PSize;
        let status =
            unsafe { (self.xlib.XGetWMNormalHints)(self.display, window, &mut xsize, &mut msize) };
        match status {
            0 => None,
            _ => Some(xsize),
        }
    }

    pub fn get_hint_sizing_as_xyhw(&self, window: xlib::Window) -> Option<XYHW> {
        if let Some(size) = self.get_hint_sizing(window) {
            let mut xyhw = XYHW {
                x: size.x,
                y: size.y,
                w: size.width,
                h: size.height,
            };

            if xyhw.w == 0 {
                xyhw.w = size.base_width
            }
            if xyhw.h == 0 {
                xyhw.h = size.base_height
            }
            if xyhw.w == 0 || xyhw.h == 0 {
                return None;
            }
            return Some(xyhw);
        }
        None
    }

    pub fn grab_buttons(&self, window: xlib::Window, button: u32, modifiers: u32) {
        //grab the keys with and without numlock (Mod2)
        let mods: Vec<u32> = vec![
            modifiers,
            modifiers | xlib::Mod2Mask,
            modifiers | xlib::LockMask,
        ];
        for m in mods {
            unsafe {
                (self.xlib.XGrabButton)(
                    self.display,
                    button,
                    m,
                    window,
                    0,
                    BUTTONMASK as u32,
                    xlib::GrabModeAsync,
                    xlib::GrabModeSync,
                    0,
                    0,
                );
            }
        }
    }

    pub fn grab_keys(&self, root: xlib::Window, keysym: u32, modifiers: u32) {
        let code = unsafe { (self.xlib.XKeysymToKeycode)(self.display, u64::from(keysym)) };
        //grab the keys with and without numlock (Mod2)
        let mods: Vec<u32> = vec![
            modifiers,
            modifiers | xlib::Mod2Mask,
            modifiers | xlib::LockMask,
        ];
        for m in mods {
            unsafe {
                (self.xlib.XGrabKey)(
                    self.display,
                    i32::from(code),
                    m,
                    root,
                    1,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
            }
        }
    }

    fn load_colors(&self, _config: &Config) -> Colors {
        Colors {
            normal: self.get_color("#000000"),
            active: self.get_color("#FF0000"),
        }
    }

    fn get_color(&self, color: &str) -> c_ulong {
        let screen = unsafe { (self.xlib.XDefaultScreen)(self.display) };
        let cmap: xlib::Colormap = unsafe { (self.xlib.XDefaultColormap)(self.display, screen) };
        let color_cstr = CString::new(color).unwrap().into_raw();
        let mut color: xlib::XColor = unsafe { std::mem::uninitialized() };
        unsafe {
            (self.xlib.XAllocNamedColor)(self.display, cmap, color_cstr, &mut color, &mut color);
        }
        color.pixel
    }

    pub fn init(&mut self, config: &Config) {
        let root_event_mask: c_long = xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::ButtonPressMask
            | xlib::PointerMotionMask
            | xlib::EnterWindowMask
            | xlib::LeaveWindowMask
            | xlib::StructureNotifyMask
            | xlib::PropertyChangeMask;

        let root = self.get_default_root();
        self.colors = self.load_colors(config);

        let mut attrs: xlib::XSetWindowAttributes = unsafe { std::mem::uninitialized() };
        attrs.cursor = self.cursors.normal;
        attrs.event_mask = root_event_mask;

        unsafe {
            (self.xlib.XChangeWindowAttributes)(
                self.display,
                self.get_default_root(),
                xlib::CWEventMask | xlib::CWCursor,
                &mut attrs,
            );
        }

        self.subscribe_to_event(root, root_event_mask);

        //EWMH junk
        unsafe {
            let supported = self.atoms.net_supported();
            let supported_ptr: *const xlib::Atom = supported.as_ptr();
            let size = supported.len() as i32;
            (self.xlib.XChangeProperty)(
                self.display,
                root,
                self.atoms.NetSupported,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                supported_ptr as *const u8,
                size,
            );
            std::mem::forget(supported);
            //cleanup the client list
            (self.xlib.XDeleteProperty)(self.display, root, self.atoms.NetClientList);
        }

        //EWMH stuff for desktops
        let tags = config.get_list_of_tags();
        self.tags = tags.clone();
        self.init_destops_hints(tags);

        //cleanup grabs
        unsafe {
            (self.xlib.XUngrabKey)(self.display, xlib::AnyKey, xlib::AnyModifier, root);
        }

        //grab all the key combos from the config file
        for kb in config.mapped_bindings() {
            if let Some(keysym) = utils::xkeysym_lookup::into_keysym(&kb.key) {
                let modmask = utils::xkeysym_lookup::into_modmask(&kb.modifier);
                self.grab_keys(root, keysym, modmask);
            }
        }

        unsafe {
            (self.xlib.XSync)(self.display, 0);
        }
    }

    pub fn set_mode(&mut self, mode: DisplayServerMode) {
        //prevent resizing and moveing or root
        match &mode {
            DisplayServerMode::MovingWindow(h) | DisplayServerMode::ResizingWindow(h) => {
                if h == &self.get_default_root_handle() {
                    return;
                }
            }
            _ => {}
        }
        if self.mode == DisplayServerMode::NormalMode && mode != DisplayServerMode::NormalMode {
            self.mode = mode.clone();
            //safe this point as the start of the move/resize
            if let Some(loc) = self.get_pointer_location() {
                self.mode_origin = loc;
            }
            unsafe {
                let cursor = match mode {
                    DisplayServerMode::ResizingWindow(_) => self.cursors.resize,
                    DisplayServerMode::MovingWindow(_) => self.cursors.move_,
                    DisplayServerMode::NormalMode => self.cursors.normal,
                };
                //grab the mouse
                (self.xlib.XGrabPointer)(
                    self.display,
                    self.root,
                    0,
                    MOUSEMASK as u32,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                    0,
                    cursor,
                    xlib::CurrentTime,
                );
            }
        }
        if mode == DisplayServerMode::NormalMode {
            //release the mouse grab
            unsafe {
                (self.xlib.XUngrabPointer)(self.display, xlib::CurrentTime);
            }
            self.mode = mode;
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
