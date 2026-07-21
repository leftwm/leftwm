use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    Dnd,
    Normal,
}

impl WindowType {
    #[must_use]
    pub fn is_dialog_like(&self) -> bool {
        matches!(
            self,
            Self::Dialog
                | Self::Splash
                | Self::Utility
                | Self::Menu
                | Self::DropdownMenu
                | Self::PopupMenu
                | Self::Tooltip
                | Self::Notification
                | Self::Combo
                | Self::Dnd
        )
    }
}
