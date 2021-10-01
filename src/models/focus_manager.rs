use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::{models::TagId, models::WindowHandle, Manager, Window, Workspace};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FocusManager {
    pub behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub workspace_history: VecDeque<usize>,
    pub window_history: VecDeque<MaybeWindowHandle>,
    pub tag_history: VecDeque<String>,
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
    pub fn workspace<'a, 'b, C: Config, SERVER: DisplayServer>(
        &self,
        manager: &'a Manager<C, SERVER>,
    ) -> Option<&'b Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.get(0)?;
        manager.state.workspaces.get(index)
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
    pub fn window<'a, 'b, C: Config, SERVER: DisplayServer>(
        &self,
        manager: &'a Manager<C, SERVER>,
    ) -> Option<&'b Window>
    where
        'a: 'b,
    {
        let handle = *self.window_history.get(0)?;
        if let Some(handle) = handle {
            return manager.state.windows.iter().find(|w| w.handle == handle);
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
