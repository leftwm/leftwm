use crate::{models::TagId, models::WindowHandle, Manager, Window, Workspace};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::MaybeWindowHandle;

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

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct FocusManager {
    pub behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub workspace_history: VecDeque<usize>,
    pub window_history: VecDeque<MaybeWindowHandle>,
    pub tag_history: VecDeque<String>,
    pub tags_last_window: HashMap<TagId, WindowHandle>,
}

impl FocusManager {
    /// Return the currently focused workspace.
    #[must_use]
    pub fn workspace<'a, 'b>(&self, manager: &'a Manager) -> Option<&'b Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.get(0)?;
        manager.workspaces.get(index)
    }

    /// Return the currently focused workspace.
    pub fn workspace_mut<'a, 'b>(
        &self,
        workspaces: &'a mut Vec<Workspace>,
    ) -> Option<&'b mut Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.get(0)?;
        workspaces.get_mut(index)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    pub fn tag(&self, offset: usize) -> Option<String> {
        self.tag_history
            .get(offset)
            .map(std::string::ToString::to_string)
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn window<'a, 'b>(&self, manager: &'a Manager) -> Option<&'b Window>
    where
        'a: 'b,
    {
        let handle = *self.window_history.get(0)?;
        if let Some(handle) = handle {
            return manager.windows.iter().find(|w| w.handle == handle);
        }
        None
    }

    /// Return the currently focused window.
    pub fn window_mut<'a, 'b>(&self, windows: &'a mut Vec<Window>) -> Option<&'b mut Window>
    where
        'a: 'b,
    {
        let handle = *self.window_history.get(0)?;
        if let Some(handle) = handle {
            return windows.iter_mut().find(|w| w.handle == handle);
        }
        None
    }
}
