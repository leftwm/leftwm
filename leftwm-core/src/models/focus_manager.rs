use crate::config::Config;
use crate::{models::TagId, models::WindowHandle, Window, Workspace};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::MaybeWindowHandle;
use super::window::Handle;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct FocusManager<H: Handle> {
    pub behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub workspace_history: VecDeque<usize>,
    #[serde(bound = "")]
    pub window_history: VecDeque<MaybeWindowHandle<H>>,
    pub tag_history: VecDeque<TagId>,
    #[serde(bound = "")]
    pub tags_last_window: HashMap<TagId, WindowHandle<H>>,
    pub sloppy_mouse_follows_focus: bool,
    pub create_follows_cursor: bool,
    pub last_mouse_position: Option<(i32, i32)>,
}

impl<H: Handle> FocusManager<H> {
    pub fn new(config: &impl Config) -> Self {
        Self {
            behaviour: config.focus_behaviour(),
            focus_new_windows: config.focus_new_windows(),
            workspace_history: Default::default(),
            window_history: Default::default(),
            tag_history: Default::default(),
            tags_last_window: Default::default(),
            sloppy_mouse_follows_focus: config.sloppy_mouse_follows_focus(),
            create_follows_cursor: config.create_follows_cursor(),
            last_mouse_position: None,
        }
    }

    /// Return the currently focused workspace.
    #[must_use]
    pub fn workspace<'a, 'b>(&self, workspaces: &'a [Workspace]) -> Option<&'b Workspace>
    where
        'a: 'b,
    {
        let index = *self.workspace_history.front()?;
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
        let index = *self.workspace_history.front()?;
        workspaces.get_mut(index)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    pub fn tag(&self, offset: usize) -> Option<TagId> {
        self.tag_history.get(offset).copied()
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn window<'a, 'b>(&self, windows: &'a [Window<H>]) -> Option<&'b Window<H>>
    where
        'a: 'b,
    {
        let handle = *self.window_history.front()?;
        if let Some(handle) = handle {
            return windows.iter().find(|w| w.handle == handle);
        }
        None
    }

    /// Return the currently focused window.
    pub fn window_mut<'a, 'b>(&self, windows: &'a mut [Window<H>]) -> Option<&'b mut Window<H>>
    where
        'a: 'b,
    {
        let handle = *self.window_history.front()?;
        if let Some(handle) = handle {
            return windows.iter_mut().find(|w| w.handle == handle);
        }
        None
    }

    pub fn create_follows_cursor(&self) -> bool {
        self.create_follows_cursor
    }
}
