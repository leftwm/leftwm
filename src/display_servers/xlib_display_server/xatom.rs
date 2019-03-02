use std::ffi::CString;
use std::os::raw::c_ulong;
use x11_dl::xlib;

pub struct XAtom {
    pub WMProtocols: c_ulong,
    pub WMDelete: c_ulong,
    pub WMState: c_ulong,
    pub WMTakeFocus: c_ulong,
    pub NetActiveWindow: c_ulong,
    pub NetSupported: c_ulong,
    pub NetWMName: c_ulong,
    pub NetWMState: c_ulong,
    pub NetWMFullscreen: c_ulong,
    pub NetWMWindowType: c_ulong,
    pub NetWMWindowTypeDialog: c_ulong,
    pub NetClientList: c_ulong,
}

impl XAtom {
    pub fn new(xlib: &xlib::Xlib, dpy: *mut xlib::Display) -> XAtom {
        unsafe {
            XAtom {
                WMProtocols: (xlib.XInternAtom)(
                    dpy,
                    CString::new("WM_PROTOCOLS").unwrap().into_raw(),
                    0,
                ),
                WMDelete: (xlib.XInternAtom)(
                    dpy,
                    CString::new("WM_DELETE_WINDOW").unwrap().into_raw(),
                    0,
                ),
                WMState: (xlib.XInternAtom)(dpy, CString::new("WM_STATE").unwrap().into_raw(), 0),
                WMTakeFocus: (xlib.XInternAtom)(
                    dpy,
                    CString::new("WM_TAKE_FOCUS").unwrap().into_raw(),
                    0,
                ),
                NetActiveWindow: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_ACTIVE_WINDOW").unwrap().into_raw(),
                    0,
                ),
                NetSupported: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_SUPPORTED").unwrap().into_raw(),
                    0,
                ),
                NetWMName: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_WM_NAME").unwrap().into_raw(),
                    0,
                ),
                NetWMState: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_WM_STATE").unwrap().into_raw(),
                    0,
                ),
                NetWMFullscreen: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().into_raw(),
                    0,
                ),
                NetWMWindowType: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_WM_WINDOW_TYPE").unwrap().into_raw(),
                    0,
                ),
                NetWMWindowTypeDialog: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_WM_WINDOW_TYPE_DIALOG")
                        .unwrap()
                        .into_raw(),
                    0,
                ),
                NetClientList: (xlib.XInternAtom)(
                    dpy,
                    CString::new("_NET_CLIENT_LIST").unwrap().into_raw(),
                    0,
                ),
            }
        }
    }
}
