use std::ffi::CString;
use x11_dl::xlib;

//#![allow(non_snake_case)]
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct XAtom {
    pub WMProtocols: xlib::Atom,
    pub WMDelete: xlib::Atom,
    pub WMState: xlib::Atom,
    pub WMTakeFocus: xlib::Atom,
    pub NetActiveWindow: xlib::Atom,
    pub NetSupported: xlib::Atom,
    pub NetWMName: xlib::Atom,
    pub NetWMState: xlib::Atom,
    pub NetWMStateSticky: xlib::Atom,
    pub NetWMStateAbove: xlib::Atom,
    pub NetWMStateFullscreen: xlib::Atom,
    pub NetWMWindowType: xlib::Atom,
    pub NetWMWindowTypeDialog: xlib::Atom,
    pub NetWMWindowTypeDock: xlib::Atom,
    pub NetClientList: xlib::Atom,
    pub NetDesktopViewport: xlib::Atom,
    pub NetNumberOfDesktops: xlib::Atom,
    pub NetCurrentDesktop: xlib::Atom,
    pub NetDesktopNames: xlib::Atom,
    pub NetWMStrutPartial: xlib::Atom, //net version - Reserve Screen Space
    pub NetWMStrut: xlib::Atom,        //old version
}

impl XAtom {
    pub fn net_supported(&self) -> Vec<xlib::Atom> {
        vec![
            self.NetActiveWindow,
            self.NetSupported,
            self.NetWMName,
            self.NetWMState,
            self.NetWMStateSticky,
            self.NetWMStateAbove,
            self.NetWMStateFullscreen,
            self.NetWMWindowType,
            self.NetWMWindowTypeDialog,
            self.NetWMWindowTypeDock,
            self.NetClientList,
            self.NetDesktopViewport,
            self.NetNumberOfDesktops,
            self.NetCurrentDesktop,
            self.NetDesktopNames,
            self.NetWMStrutPartial,
            self.NetWMStrut,
        ]
    }

    pub fn get_name(&self, atom: xlib::Atom) -> &str {
        if atom == self.WMProtocols {
            return "WM_PROTOCOLS";
        }
        if atom == self.WMDelete {
            return "WM_DELETE_WINDOW";
        }
        if atom == self.WMState {
            return "WM_STATE";
        }
        if atom == self.WMTakeFocus {
            return "WM_TAKE_FOCUS";
        }
        if atom == self.NetActiveWindow {
            return "_NET_ACTIVE_WINDOW";
        }
        if atom == self.NetSupported {
            return "_NET_SUPPORTED";
        }
        if atom == self.NetWMName {
            return "_NET_WM_NAME";
        }
        if atom == self.NetWMState {
            return "_NET_WM_STATE";
        }
        if atom == self.NetWMStateSticky {
            return "_NET_WM_STATE_STICKY";
        }
        if atom == self.NetWMStateAbove {
            return "_NET_WM_STATE_ABOVE";
        }
        if atom == self.NetWMStateFullscreen {
            return "_NET_WM_STATE_FULLSCREEN";
        }
        if atom == self.NetWMWindowType {
            return "_NET_WM_WINDOW_TYPE";
        }
        if atom == self.NetWMWindowTypeDialog {
            return "_NET_WM_WINDOW_TYPE_DIALOG";
        }
        if atom == self.NetWMWindowTypeDock {
            return "_NET_WM_WINDOW_TYPE_DOCK";
        }
        if atom == self.NetClientList {
            return "_NET_CLIENT_LIST";
        }
        if atom == self.NetDesktopViewport {
            return "_NET_DESKTOP_VIEWPORT";
        }
        if atom == self.NetNumberOfDesktops {
            return "_NET_NUMBER_OF_DESKTOPS";
        }
        if atom == self.NetCurrentDesktop {
            return "_NET_CURRENT_DESKTOP";
        }
        if atom == self.NetDesktopNames {
            return "_NET_DESKTOP_NAMES";
        }
        if atom == self.NetWMStrutPartial {
            return "_NET_WM_STRUT_PARTIAL";
        }
        if atom == self.NetWMStrut {
            return "_NET_WM_STRUT";
        }
        "(UNKNOWN)"
    }

    pub fn new(xlib: &xlib::Xlib, dpy: *mut xlib::Display) -> XAtom {
        XAtom {
            WMProtocols: from(xlib, dpy, "WM_PROTOCOLS"),
            WMDelete: from(xlib, dpy, "WM_DELETE_WINDOW"),
            WMState: from(xlib, dpy, "WM_STATE"),
            WMTakeFocus: from(xlib, dpy, "WM_TAKE_FOCUS"),
            NetActiveWindow: from(xlib, dpy, "_NET_ACTIVE_WINDOW"),
            NetSupported: from(xlib, dpy, "_NET_SUPPORTED"),
            NetWMName: from(xlib, dpy, "_NET_WM_NAME"),
            NetWMState: from(xlib, dpy, "_NET_WM_STATE"),
            NetWMStateSticky: from(xlib, dpy, "_NET_WM_STATE_STICKY"),
            NetWMStateAbove: from(xlib, dpy, "_NET_WM_STATE_ABOVE"),
            NetWMStateFullscreen: from(xlib, dpy, "_NET_WM_STATE_FULLSCREEN"),
            NetWMWindowType: from(xlib, dpy, "_NET_WM_WINDOW_TYPE"),
            NetWMWindowTypeDialog: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_DIALOG"),
            NetWMWindowTypeDock: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_DOCK"),
            NetClientList: from(xlib, dpy, "_NET_CLIENT_LIST"),
            NetDesktopViewport: from(xlib, dpy, "_NET_DESKTOP_VIEWPORT"),
            NetNumberOfDesktops: from(xlib, dpy, "_NET_NUMBER_OF_DESKTOPS"),
            NetCurrentDesktop: from(xlib, dpy, "_NET_CURRENT_DESKTOP"),
            NetDesktopNames: from(xlib, dpy, "_NET_DESKTOP_NAMES"),
            NetWMStrutPartial: from(xlib, dpy, "_NET_WM_STRUT_PARTIAL"),
            NetWMStrut: from(xlib, dpy, "_NET_WM_STRUT"),
        }
    }
}

fn from(xlib: &xlib::Xlib, dpy: *mut xlib::Display, s: &str) -> xlib::Atom {
    unsafe { (xlib.XInternAtom)(dpy, CString::new(s).unwrap().into_raw(), xlib::False) }
}
