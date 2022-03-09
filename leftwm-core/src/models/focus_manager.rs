use crate::config::Config;
use crate::{models::TagId, models::WindowHandle, Window, Workspace};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::MaybeWindowHandle;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FocusBehaviour {
    Sloppy,
    ClickTo,
    Driven,
}

impl Default for FocusBehaviour {
    fn default() -> Self {
        Self::Sloppy
    }
}

impl FocusBehaviour {
    pub fn is_sloppy(self) -> bool {
        self == FocusBehaviour::Sloppy
    }

    pub fn is_clickto(self) -> bool {
        self == FocusBehaviour::ClickTo
    }

    pub fn is_driven(self) -> bool {
        self == FocusBehaviour::Driven
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FocusManager {
    pub behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub workspace_history: VecDeque<usize>,
    pub window_history: VecDeque<MaybeWindowHandle>,
    pub tag_history: VecDeque<TagId>,
    pub tags_last_window: HashMap<TagId, WindowHandle>,
}

impl FocusManager {
    pub fn new(config: &impl Config) -> Self {
        Self {
            behaviour: config.focus_behaviour(),
            focus_new_windows: config.focus_new_windows(),
            workspace_history: Default::default(),
            window_history: Default::default(),
            tag_history: Default::default(),
            tags_last_window: Default::default(),
        }
    }

    /// Return the currently focused workspace.
    #[must_use]
    pub fn workspace<'a, 'b>(&self, workspaces: &'a [Workspace]) -> Option<&'b Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.get(0)?;
        workspaces.get(index)
    }

    /// Return the currently focused workspace.
    pub fn workspace_mut<'a, 'b>(
        &self,
        workspaces: &'a mut [Workspace],
    ) -> Option<&'b mut Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.get(0)?;
        workspaces.get_mut(index)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    pub fn tag(&self, offset: usize) -> Option<TagId> {
        self.tag_history.get(offset).copied()
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn window<'a, 'b>(&self, windows: &'a [Window]) -> Option<&'b Window>
    where
        'a: 'b,
    {
        let handle = *self.window_history.get(0)?;
        if let Some(handle) = handle {
            return windows.iter().find(|w| w.handle == handle);
        }
        None
    }

    /// Return the currently focused window.
    pub fn window_mut<'a, 'b>(&self, windows: &'a mut [Window]) -> Option<&'b mut Window>
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
