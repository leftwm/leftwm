use x11rb::{atom_manager, protocol::xproto};

atom_manager! {
    /// A collection of Atoms.
    pub AtomCollection:

    /// A handle to a response from the X11 server.
    AtomCollectionCookie {
        WMProtocols: b"WM_PROTOCOLS" as &[u8],
        WMDelete: b"WM_DELETE_WINDOW",
        WMState: b"WM_STATE",
        WMClass: b"WM_CLASS",
        WMTakeFocus: b"WM_TAKE_FOCUS",
        NetActiveWindow: b"_NET_ACTIVE_WINDOW",
        NetSupported: b"_NET_SUPPORTED",
        NetWMName: b"_NET_WM_NAME",
        NetWMPid: b"_NET_WM_PID",

        NetWMState: b"_NET_WM_STATE",
        NetWMStateModal: b"_NET_WM_STATE_MODAL",
        NetWMStateSticky: b"_NET_WM_STATE_STICKY",
        NetWMStateMaximizedVert: b"_NET_WM_STATE_MAXIMIZED_VERT",
        NetWMStateMaximizedHorz: b"_NET_WM_STATE_MAXIMIZED_HORZ",
        NetWMStateShaded: b"_NET_WM_STATE_SHADED",
        NetWMStateSkipTaskbar: b"_NET_WM_STATE_SKIP_TASKBAR",
        NetWMStateSkipPager: b"_NET_WM_STATE_SKIP_PAGER",
        NetWMStateHidden: b"_NET_WM_STATE_HIDDEN",
        NetWMStateFullscreen: b"_NET_WM_STATE_FULLSCREEN",
        NetWMStateAbove: b"_NET_WM_STATE_ABOVE",
        NetWMStateBelow: b"_NET_WM_STATE_BELOW",
        NetWMStateDemandsAttention: b"_NET_WM_STATE_DEMANDS_ATTENTION",

        NetWMAction: b"_NET_WM_ALLOWED_ACTIONS",
        NetWMActionMove: b"_NET_WM_ACTION_MOVE",
        NetWMActionResize: b"_NET_WM_ACTION_RESIZE",
        NetWMActionMinimize: b"_NET_WM_ACTION_MINIMIZE",
        NetWMActionShade: b"_NET_WM_ACTION_SHADE",
        NetWMActionStick: b"_NET_WM_ACTION_STICK",
        NetWMActionMaximizeHorz: b"_NET_WM_ACTION_MAXIMIZE_HORZ",
        NetWMActionMaximizeVert: b"_NET_WM_ACTION_MAXIMIZE_VERT",
        NetWMActionFullscreen: b"_NET_WM_ACTION_FULLSCREEN",
        NetWMActionChangeDesktop: b"_NET_WM_ACTION_CHANGE_DESKTOP",
        NetWMActionClose: b"_NET_WM_ACTION_CLOSE",

        NetWMWindowType: b"_NET_WM_WINDOW_TYPE",
        NetWMWindowTypeDesktop: b"_NET_WM_WINDOW_TYPE_DESKTOP",
        NetWMWindowTypeDock: b"_NET_WM_WINDOW_TYPE_DOCK",
        NetWMWindowTypeToolbar: b"_NET_WM_WINDOW_TYPE_TOOLBAR",
        NetWMWindowTypeMenu: b"_NET_WM_WINDOW_TYPE_MENU",
        NetWMWindowTypeUtility: b"_NET_WM_WINDOW_TYPE_UTILITY",
        NetWMWindowTypeSplash: b"_NET_WM_WINDOW_TYPE_SPLASH",
        NetWMWindowTypeDialog: b"_NET_WM_WINDOW_TYPE_DIALOG",
        NetSupportingWmCheck: b"_NET_SUPPORTING_WM_CHECK",

        NetClientList: b"_NET_CLIENT_LIST",
        NetDesktopViewport: b"_NET_DESKTOP_VIEWPORT",
        NetNumberOfDesktops: b"_NET_NUMBER_OF_DESKTOPS",
        NetCurrentDesktop: b"_NET_CURRENT_DESKTOP",
        NetDesktopNames: b"_NET_DESKTOP_NAMES",
        NetWMDesktop: b"_NET_WM_DESKTOP",
        NetWMStrutPartial: b"_NET_WM_STRUT_PARTIAL",
        NetWMStrut: b"_NET_WM_STRUT",

        UTF8String: b"UTF8_STRING",

        WMNormalHints: b"WM_NORMAL_HINTS",
        WMSizeHints: b"WM_SIZE_HINTS",
    }
}

impl AtomCollection {
    pub fn net_supported(&self) -> Vec<xproto::Atom> {
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
}

impl AtomCollection {
    pub fn get_name(&self, atom: xproto::Atom) -> &'static str {
        match atom {
            x if x == self.WMProtocols => "WM_PROTOCOLS",
            x if x == self.WMDelete => "WM_DELETE_WINDOW",
            x if x == self.WMState => "WM_STATE",
            x if x == self.WMClass => "WM_CLASS",
            x if x == self.WMTakeFocus => "WM_TAKE_FOCUS",
            x if x == self.NetActiveWindow => "_NET_ACTIVE_WINDOW",
            x if x == self.NetSupported => "_NET_SUPPORTED",
            x if x == self.NetWMName => "_NET_WM_NAME",
            x if x == self.NetWMPid => "_NET_WM_PID",
            x if x == self.NetWMState => "_NET_WM_STATE",
            x if x == self.NetWMStateModal => "_NET_WM_STATE_MODAL",
            x if x == self.NetWMStateSticky => "_NET_WM_STATE_STICKY",
            x if x == self.NetWMStateMaximizedVert => "_NET_WM_STATE_MAXIMIZED_VERT",
            x if x == self.NetWMStateMaximizedHorz => "_NET_WM_STATE_MAXIMIZED_HORZ",
            x if x == self.NetWMStateShaded => "_NET_WM_STATE_SHADED",
            x if x == self.NetWMStateSkipTaskbar => "_NET_WM_STATE_SKIP_TASKBAR",
            x if x == self.NetWMStateSkipPager => "_NET_WM_STATE_SKIP_PAGER",
            x if x == self.NetWMStateHidden => "_NET_WM_STATE_HIDDEN",
            x if x == self.NetWMStateFullscreen => "_NET_WM_STATE_FULLSCREEN",
            x if x == self.NetWMStateAbove => "_NET_WM_STATE_ABOVE",
            x if x == self.NetWMStateBelow => "_NET_WM_STATE_BELOW",
            x if x == self.NetWMStateDemandsAttention => "_NET_WM_STATE_DEMANDS_ATTENTION",
            x if x == self.NetWMAction => "_NET_WM_ALLOWED_ACTIONS",
            x if x == self.NetWMActionMove => "_NET_WM_ACTION_MOVE",
            x if x == self.NetWMActionResize => "_NET_WM_ACTION_RESIZE",
            x if x == self.NetWMActionMinimize => "_NET_WM_ACTION_MINIMIZE",
            x if x == self.NetWMActionShade => "_NET_WM_ACTION_SHADE",
            x if x == self.NetWMActionStick => "_NET_WM_ACTION_STICK",
            x if x == self.NetWMActionMaximizeHorz => "_NET_WM_ACTION_MAXIMIZE_HORZ",
            x if x == self.NetWMActionMaximizeVert => "_NET_WM_ACTION_MAXIMIZE_VERT",
            x if x == self.NetWMActionFullscreen => "_NET_WM_ACTION_FULLSCREEN",
            x if x == self.NetWMActionChangeDesktop => "_NET_WM_ACTION_CHANGE_DESKTOP",
            x if x == self.NetWMActionClose => "_NET_WM_ACTION_CLOSE",
            x if x == self.NetWMWindowType => "_NET_WM_WINDOW_TYPE",
            x if x == self.NetWMWindowTypeDesktop => "_NET_WM_WINDOW_TYPE_DESKTOP",
            x if x == self.NetWMWindowTypeDock => "_NET_WM_WINDOW_TYPE_DOCK",
            x if x == self.NetWMWindowTypeToolbar => "_NET_WM_WINDOW_TYPE_TOOLBAR",
            x if x == self.NetWMWindowTypeMenu => "_NET_WM_WINDOW_TYPE_MENU",
            x if x == self.NetWMWindowTypeUtility => "_NET_WM_WINDOW_TYPE_UTILITY",
            x if x == self.NetWMWindowTypeSplash => "_NET_WM_WINDOW_TYPE_SPLASH",
            x if x == self.NetWMWindowTypeDialog => "_NET_WM_WINDOW_TYPE_DIALOG",
            x if x == self.NetSupportingWmCheck => "_NET_SUPPORTING_WM_CHECK",
            x if x == self.NetClientList => "_NET_CLIENT_LIST",
            x if x == self.NetDesktopViewport => "_NET_DESKTOP_VIEWPORT",
            x if x == self.NetNumberOfDesktops => "_NET_NUMBER_OF_DESKTOPS",
            x if x == self.NetCurrentDesktop => "_NET_CURRENT_DESKTOP",
            x if x == self.NetDesktopNames => "_NET_DESKTOP_NAMES",
            x if x == self.NetWMDesktop => "_NET_WM_DESKTOP",
            x if x == self.NetWMStrutPartial => "_NET_WM_STRUT_PARTIAL",
            x if x == self.NetWMStrut => "_NET_WM_STRUT",
            x if x == self.WMNormalHints => "WM_NORMAL_HINTS",
            x if x == self.WMSizeHints => "WM_SIZE_HINTS",
            x if x == self.UTF8String => "UTF8_STRING",
            _ => "(UNKNOWN)",
        }
    }
}

/// Possible values of the `state` field of `WM_STATE`
#[derive(Debug, PartialEq)]
pub enum WMStateWindowState {
    Withdrawn,
    Normal,
    Iconic,
}

pub struct InvalidWindowState;

impl TryFrom<&u32> for WMStateWindowState {
    type Error = InvalidWindowState;

    fn try_from(value: &u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Withdrawn),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Iconic),
            _ => Err(InvalidWindowState),
        }
    }
}

impl From<WMStateWindowState> for u32 {
    fn from(value: WMStateWindowState) -> Self {
        match value {
            WMStateWindowState::Withdrawn => 0,
            WMStateWindowState::Normal => 1,
            WMStateWindowState::Iconic => 2,
        }
    }
}
