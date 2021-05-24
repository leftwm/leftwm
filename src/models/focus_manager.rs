use crate::{models::WindowHandle, Manager, Window, Workspace};

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FocusBehaviour {
    Sloppy,
    ClickTo,
    Driven,
}

impl Default for FocusBehaviour {
    fn default() -> Self {
        FocusBehaviour::Sloppy
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct FocusManager {
    pub behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub focused_workspace_history: VecDeque<usize>,
    pub focused_window_history: VecDeque<WindowHandle>,
    pub focused_tag_history: VecDeque<String>,
}

impl FocusManager {
    /// Return the currently focused workspace.
    #[must_use]
    pub fn focused_workspace<'a, 'b>(&self, manager: &'a Manager) -> Option<&'b Workspace>
    where
        'a: 'b,
    {
        let index = *self.focused_workspace_history.get(0)?;
        manager.workspaces.get(index)
    }

    /// Return the currently focused workspace.
    pub fn focused_workspace_mut<'a, 'b>(
        &self,
        manager: &'a mut Manager,
    ) -> Option<&'b mut Workspace>
    where
        'a: 'b,
    {
        let index = *self.focused_workspace_history.get(0)?;
        manager.workspaces.get_mut(index)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    pub fn focused_tag(&self, offset: usize) -> Option<String> {
        self.focused_tag_history
            .get(offset)
            .map(std::string::ToString::to_string)
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn focused_window<'a, 'b>(&self, manager: &'a Manager) -> Option<&'b Window>
    where
        'a: 'b,
    {
        let handle = *self.focused_window_history.get(0)?;
        manager.windows.iter().find(|w| w.handle == handle)
    }

    /// Return the currently focused window.
    pub fn focused_window_mut<'a, 'b>(&self, manager: &'a mut Manager) -> Option<&'b mut Window>
    where
        'a: 'b,
    {
        let handle = *self.focused_window_history.get(0)?;
        manager.windows.iter_mut().find(|w| w.handle == handle)
    }
}
