use std::ffi::CString;
use x11_dl::xlib;

// Specifications can be found here:
// https://specifications.freedesktop.org/wm-spec/1.3/ar01s03.html

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct XAtom {
    pub WMProtocols: xlib::Atom,
    pub WMDelete: xlib::Atom,
    pub WMState: xlib::Atom,
    pub WMClass: xlib::Atom,
    pub WMTakeFocus: xlib::Atom,
    pub NetActiveWindow: xlib::Atom,
    pub NetSupported: xlib::Atom,
    pub NetWMName: xlib::Atom,
    pub NetWMState: xlib::Atom,
    pub NetWMAction: xlib::Atom,
    pub NetWMPid: xlib::Atom,

    pub NetWMActionMove: xlib::Atom,
    pub NetWMActionResize: xlib::Atom,
    pub NetWMActionMinimize: xlib::Atom,
    pub NetWMActionShade: xlib::Atom,
    pub NetWMActionStick: xlib::Atom,
    pub NetWMActionMaximizeHorz: xlib::Atom,
    pub NetWMActionMaximizeVert: xlib::Atom,
    pub NetWMActionFullscreen: xlib::Atom,
    pub NetWMActionChangeDesktop: xlib::Atom,
    pub NetWMActionClose: xlib::Atom,

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

    pub UTF8String: xlib::Atom,
}

impl XAtom {
    pub fn net_supported(&self) -> Vec<xlib::Atom> {
        vec![
            self.NetActiveWindow,
            self.NetSupported,
            self.NetWMName,
            self.NetWMState,
            self.NetWMAction,
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
            self.NetWMActionMove,
            self.NetWMActionResize,
            self.NetWMActionMinimize,
            self.NetWMActionShade,
            self.NetWMActionStick,
            self.NetWMActionMaximizeHorz,
            self.NetWMActionMaximizeVert,
            self.NetWMActionFullscreen,
            self.NetWMActionChangeDesktop,
            self.NetWMActionClose,
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
        match atom {
            a if a == self.WMProtocols => "WM_PROTOCOLS",
            a if a == self.WMDelete => "WM_DELETE_WINDOW",
            a if a == self.WMState => "WM_STATE",
            a if a == self.WMClass => "WM_CLASS",
            a if a == self.WMTakeFocus => "WM_TAKE_FOCUS",
            a if a == self.NetActiveWindow => "_NET_ACTIVE_WINDOW",
            a if a == self.NetSupported => "_NET_SUPPORTED",
            a if a == self.NetWMName => "_NET_WM_NAME",
            a if a == self.NetWMState => "_NET_WM_STATE",
            a if a == self.NetWMAction => "_NET_WM_ALLOWED_ACTIONS",
            a if a == self.NetWMPid => "_NET_WM_PID",

            a if a == self.NetWMStateModal => "NetWMStateModal",
            a if a == self.NetWMStateSticky => "NetWMStateSticky",
            a if a == self.NetWMStateMaximizedVert => "NetWMStateMaximizedVert",
            a if a == self.NetWMStateMaximizedHorz => "NetWMStateMaximizedHorz",
            a if a == self.NetWMStateShaded => "NetWMStateShaded",
            a if a == self.NetWMStateSkipTaskbar => "NetWMStateSkipTaskbar",
            a if a == self.NetWMStateSkipPager => "NetWMStateSkipPager",
            a if a == self.NetWMStateHidden => "NetWMStateHidden",
            a if a == self.NetWMStateFullscreen => "NetWMStateFullscreen",
            a if a == self.NetWMStateAbove => "NetWMStateAbove",
            a if a == self.NetWMStateBelow => "NetWMStateBelow",

            a if a == self.NetWMActionMove => "_NET_WM_ACTION_MOVE",
            a if a == self.NetWMActionResize => "_NET_WM_ACTION_RESIZE",
            a if a == self.NetWMActionMinimize => "_NET_WM_ACTION_MINIMIZE",
            a if a == self.NetWMActionShade => "_NET_WM_ACTION_SHADE",
            a if a == self.NetWMActionStick => "_NET_WM_ACTION_STICK",
            a if a == self.NetWMActionMaximizeHorz => "_NET_WM_ACTION_MAXIMIZE_HORZ",
            a if a == self.NetWMActionMaximizeVert => "_NET_WM_ACTION_MAXIMIZE_VERT",
            a if a == self.NetWMActionFullscreen => "_NET_WM_ACTION_FULLSCREEN",
            a if a == self.NetWMActionChangeDesktop => "_NET_WM_ACTION_CHANGE_DESKTOP",
            a if a == self.NetWMActionClose => "_NET_WM_ACTION_CLOSE",

            a if a == self.NetWMWindowType => "_NET_WM_WINDOW_TYPE",
            a if a == self.NetWMWindowTypeDialog => "_NET_WM_WINDOW_TYPE_DIALOG",
            a if a == self.NetWMWindowTypeDock => "_NET_WM_WINDOW_TYPE_DOCK",
            a if a == self.NetClientList => "_NET_CLIENT_LIST",
            a if a == self.NetDesktopViewport => "_NET_DESKTOP_VIEWPORT",
            a if a == self.NetNumberOfDesktops => "_NET_NUMBER_OF_DESKTOPS",
            a if a == self.NetCurrentDesktop => "_NET_CURRENT_DESKTOP",
            a if a == self.NetDesktopNames => "_NET_DESKTOP_NAMES",
            a if a == self.NetWMDesktop => "_NET_WM_DESKTOP",
            a if a == self.NetWMStrutPartial => "_NET_WM_STRUT_PARTIAL",
            a if a == self.NetWMStrut => "_NET_WM_STRUT",

            a if a == self.UTF8String => "UTF8_STRING",
            _ => "(UNKNOWN)",
        }
    }

    pub fn new(xlib: &xlib::Xlib, dpy: *mut xlib::Display) -> Self {
        Self {
            WMProtocols: from(xlib, dpy, "WM_PROTOCOLS"),
            WMDelete: from(xlib, dpy, "WM_DELETE_WINDOW"),
            WMState: from(xlib, dpy, "WM_STATE"),
            WMClass: from(xlib, dpy, "WM_CLASS"),
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

            NetWMAction: from(xlib, dpy, "_NET_WM_ALLOWED_ACTIONS"),
            NetWMActionMove: from(xlib, dpy, "_NET_WM_ACTION_MOVE"),
            NetWMActionResize: from(xlib, dpy, "_NET_WM_ACTION_RESIZE"),
            NetWMActionMinimize: from(xlib, dpy, "_NET_WM_ACTION_MINIMIZE"),
            NetWMActionShade: from(xlib, dpy, "_NET_WM_ACTION_SHADE"),
            NetWMActionStick: from(xlib, dpy, "_NET_WM_ACTION_STICK"),
            NetWMActionMaximizeHorz: from(xlib, dpy, "_NET_WM_ACTION_MAXIMIZE_HORZ"),
            NetWMActionMaximizeVert: from(xlib, dpy, "_NET_WM_ACTION_MAXIMIZE_VERT"),
            NetWMActionFullscreen: from(xlib, dpy, "_NET_WM_ACTION_FULLSCREEN"),
            NetWMActionChangeDesktop: from(xlib, dpy, "_NET_WM_ACTION_CHANGE_DESKTOP"),
            NetWMActionClose: from(xlib, dpy, "_NET_WM_ACTION_CLOSE"),

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

            UTF8String: from(xlib, dpy, "UTF8_STRING"),
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
