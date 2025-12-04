use serde::{Deserialize, Serialize};

//NOTE: Any wayland window will be assigned the normal window type, any wayland wlr_surface is
//assigned WlrSurface type, and while not being a window is treated by leftwm as such for the
//purpose of focus
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    WlrSurface,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    Dnd,
    Normal,
}
