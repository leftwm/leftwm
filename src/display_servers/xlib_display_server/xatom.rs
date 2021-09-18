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
    pub NetWMPid: xlib::Atom,

    //pub NetWMStateSticky: xlib::Atom,
    //pub NetWMStateAbove: xlib::Atom,
    //pub NetWMStateFullscreen: xlib::Atom,
    pub NetWMStateModal: xlib::Atom,
    pub NetWMStateSticky: xlib::Atom,
    pub NetWMStateMaximizedVert: xlib::Atom,
    pub NetWMStateMaximizedHorz: xlib::Atom,
    pub NetWMStateShaded: xlib::Atom,
    pub NetWMStateSkipTaskbar: xlib::Atom,
    pub NetWMStateSkipPager: xlib::Atom,
    pub NetWMStateHidden: xlib::Atom,
    pub NetWMStateFullscreen: xlib::Atom,
    pub NetWMStateAbove: xlib::Atom,
    pub NetWMStateBelow: xlib::Atom,
    pub NetWMStateDemandsAttention: xlib::Atom,

    pub NetWMWindowType: xlib::Atom,
    pub NetWMWindowTypeDesktop: xlib::Atom,
    pub NetWMWindowTypeDock: xlib::Atom,
    pub NetWMWindowTypeToolbar: xlib::Atom,
    pub NetWMWindowTypeMenu: xlib::Atom,
    pub NetWMWindowTypeUtility: xlib::Atom,
    pub NetWMWindowTypeSplash: xlib::Atom,
    pub NetWMWindowTypeDialog: xlib::Atom,

    pub NetSupportingWmCheck: xlib::Atom,
    pub NetClientList: xlib::Atom,
    pub NetDesktopViewport: xlib::Atom,
    pub NetNumberOfDesktops: xlib::Atom,
    pub NetCurrentDesktop: xlib::Atom,
    pub NetDesktopNames: xlib::Atom,
    pub NetWMDesktop: xlib::Atom,
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
            self.NetWMPid,
            self.NetWMStateModal,
            self.NetWMStateSticky,
            self.NetWMStateMaximizedVert,
            self.NetWMStateMaximizedHorz,
            self.NetWMStateShaded,
            self.NetWMStateSkipTaskbar,
            self.NetWMStateSkipPager,
            self.NetWMStateHidden,
            self.NetWMStateFullscreen,
            self.NetWMStateAbove,
            self.NetWMStateBelow,
            self.NetWMStateDemandsAttention,
            self.NetWMWindowType,
            self.NetWMWindowTypeDesktop,
            self.NetWMWindowTypeDock,
            self.NetWMWindowTypeToolbar,
            self.NetWMWindowTypeMenu,
            self.NetWMWindowTypeUtility,
            self.NetWMWindowTypeSplash,
            self.NetWMWindowTypeDialog,
            self.NetSupportingWmCheck,
            self.NetClientList,
            self.NetDesktopViewport,
            self.NetNumberOfDesktops,
            self.NetCurrentDesktop,
            self.NetDesktopNames,
            self.NetWMDesktop,
            self.NetWMStrutPartial,
            self.NetWMStrut,
        ]
    }

    pub const fn get_name(&self, atom: xlib::Atom) -> &str {
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
        if atom == self.NetWMPid {
            return "_NET_WM_PID";
        }

        if atom == self.NetWMStateModal {
            return "NetWMStateModal";
        }
        if atom == self.NetWMStateSticky {
            return "NetWMStateSticky";
        }
        if atom == self.NetWMStateMaximizedVert {
            return "NetWMStateMaximizedVert";
        }
        if atom == self.NetWMStateMaximizedHorz {
            return "NetWMStateMaximizedHorz";
        }
        if atom == self.NetWMStateShaded {
            return "NetWMStateShaded";
        }
        if atom == self.NetWMStateSkipTaskbar {
            return "NetWMStateSkipTaskbar";
        }
        if atom == self.NetWMStateSkipPager {
            return "NetWMStateSkipPager";
        }
        if atom == self.NetWMStateHidden {
            return "NetWMStateHidden";
        }
        if atom == self.NetWMStateFullscreen {
            return "NetWMStateFullscreen";
        }
        if atom == self.NetWMStateAbove {
            return "NetWMStateAbove";
        }
        if atom == self.NetWMStateBelow {
            return "NetWMStateBelow";
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
        if atom == self.NetWMDesktop {
            return "_NET_WM_DESKTOP";
        }
        if atom == self.NetWMStrutPartial {
            return "_NET_WM_STRUT_PARTIAL";
        }
        if atom == self.NetWMStrut {
            return "_NET_WM_STRUT";
        }
        "(UNKNOWN)"
    }

    pub fn new(xlib: &xlib::Xlib, dpy: *mut xlib::Display) -> Self {
        Self {
            WMProtocols: from(xlib, dpy, "WM_PROTOCOLS"),
            WMDelete: from(xlib, dpy, "WM_DELETE_WINDOW"),
            WMState: from(xlib, dpy, "WM_STATE"),
            WMTakeFocus: from(xlib, dpy, "WM_TAKE_FOCUS"),
            NetActiveWindow: from(xlib, dpy, "_NET_ACTIVE_WINDOW"),
            NetSupported: from(xlib, dpy, "_NET_SUPPORTED"),
            NetWMName: from(xlib, dpy, "_NET_WM_NAME"),
            NetWMPid: from(xlib, dpy, "_NET_WM_PID"),

            NetWMState: from(xlib, dpy, "_NET_WM_STATE"),
            NetWMStateModal: from(xlib, dpy, "_NET_WM_STATE_MODAL"),
            NetWMStateSticky: from(xlib, dpy, "_NET_WM_STATE_STICKY"),
            NetWMStateMaximizedVert: from(xlib, dpy, "_NET_WM_STATE_MAXIMIZED_VERT"),
            NetWMStateMaximizedHorz: from(xlib, dpy, "_NET_WM_STATE_MAXIMIZED_HORZ"),
            NetWMStateShaded: from(xlib, dpy, "_NET_WM_STATE_SHADED"),
            NetWMStateSkipTaskbar: from(xlib, dpy, "_NET_WM_STATE_SKIP_TASKBAR"),
            NetWMStateSkipPager: from(xlib, dpy, "_NET_WM_STATE_SKIP_PAGER"),
            NetWMStateHidden: from(xlib, dpy, "_NET_WM_STATE_HIDDEN"),
            NetWMStateFullscreen: from(xlib, dpy, "_NET_WM_STATE_FULLSCREEN"),
            NetWMStateAbove: from(xlib, dpy, "_NET_WM_STATE_ABOVE"),
            NetWMStateBelow: from(xlib, dpy, "_NET_WM_STATE_BELOW"),
            NetWMStateDemandsAttention: from(xlib, dpy, "_NET_WM_STATE_DEMANDS_ATTENTION"),

            NetWMWindowType: from(xlib, dpy, "_NET_WM_WINDOW_TYPE"),
            NetWMWindowTypeDesktop: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_DESKTOP"),
            NetWMWindowTypeDock: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_DOCK"),
            NetWMWindowTypeToolbar: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_TOOLBAR"),
            NetWMWindowTypeMenu: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_MENU"),
            NetWMWindowTypeUtility: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_UTILITY"),
            NetWMWindowTypeSplash: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_SPLASH"),
            NetWMWindowTypeDialog: from(xlib, dpy, "_NET_WM_WINDOW_TYPE_DIALOG"),
            NetSupportingWmCheck: from(xlib, dpy, "_NET_SUPPORTING_WM_CHECK"),

            NetClientList: from(xlib, dpy, "_NET_CLIENT_LIST"),
            NetDesktopViewport: from(xlib, dpy, "_NET_DESKTOP_VIEWPORT"),
            NetNumberOfDesktops: from(xlib, dpy, "_NET_NUMBER_OF_DESKTOPS"),
            NetCurrentDesktop: from(xlib, dpy, "_NET_CURRENT_DESKTOP"),
            NetDesktopNames: from(xlib, dpy, "_NET_DESKTOP_NAMES"),
            NetWMDesktop: from(xlib, dpy, "_NET_WM_DESKTOP"),
            NetWMStrutPartial: from(xlib, dpy, "_NET_WM_STRUT_PARTIAL"),
            NetWMStrut: from(xlib, dpy, "_NET_WM_STRUT"),
        }
    }
}

fn from(xlib: &xlib::Xlib, dpy: *mut xlib::Display, s: &str) -> xlib::Atom {
    unsafe {
        (xlib.XInternAtom)(
            dpy,
            CString::new(s).unwrap_or_default().into_raw(),
            xlib::False,
        )
    }
}
