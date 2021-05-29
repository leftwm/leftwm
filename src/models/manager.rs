use crate::config::ScratchPad;
use crate::display_action::DisplayAction;
use crate::models::FocusManager;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::Tag;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::state;
use crate::utils::child_process::Children;
use crate::{config::ThemeSetting, layouts::Layout};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{atomic::AtomicBool, Arc};

/// Maintains current program state.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Manager {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub focus_manager: FocusManager,
    pub mode: Mode,
    pub theme_setting: ThemeSetting,
    #[serde(skip)]
    pub tags: Vec<Tag>, //list of all known tags
    pub layouts: Vec<Layout>,
    #[serde(skip)]
    pub scratchpads: HashMap<ScratchPad, Option<u32>>,
    pub actions: VecDeque<DisplayAction>,

    //this is used to limit framerate when resizing/moving windows
    pub frame_rate_limitor: u64,
    #[serde(skip)]
    pub children: Children,
    #[serde(skip)]
    pub reap_requested: Arc<AtomicBool>,
    #[serde(skip)]
    pub reload_requested: bool,
}

impl Manager {
    /// Return the currently focused workspace.
    #[must_use]
    pub fn focused_workspace(&self) -> Option<&Workspace> {
        self.focus_manager.workspace(&self)
    }

    /// Return the currently focused workspace.
    pub fn focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.focus_manager.workspace_mut(&mut self.workspaces)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    #[must_use]
    pub fn focused_tag(&self, offset: usize) -> Option<String> {
        self.focus_manager.tag(offset)
    }

    /// Return the index of a given tag.
    #[must_use]
    pub fn tag_index(&self, tag: &str) -> Option<usize> {
        Some(self.tags.iter().position(|t| t.id == tag)).unwrap_or(None)
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn focused_window(&self) -> Option<&Window> {
        self.focus_manager.window(&self)
    }

    /// Return the currently focused window.
    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        self.focus_manager.window_mut(&mut self.windows)
    }

    //sorts the windows and puts them in order of importance
    //keeps the order for each importance level
    pub fn sort_windows(&mut self) {
        use crate::models::WindowType;
        //first dialogs and modals
        let (level1, other): (Vec<&Window>, Vec<&Window>) = self.windows.iter().partition(|w| {
            w.type_ == WindowType::Dialog
                || w.type_ == WindowType::Splash
                || w.type_ == WindowType::Utility
                || w.type_ == WindowType::Menu
        });

        //next floating
        let (level2, other): (Vec<&Window>, Vec<&Window>) = other
            .iter()
            .partition(|w| w.type_ == WindowType::Normal && w.floating());

        //then normal windows
        let (level3, other): (Vec<&Window>, Vec<&Window>) =
            other.iter().partition(|w| w.type_ == WindowType::Normal);

        //last docks
        //other is all the reset

        //build the updated window list
        let windows: Vec<Window> = level1
            .iter()
            .chain(level2.iter())
            .chain(level3.iter())
            .chain(other.iter())
            .map(|&w| w.clone())
            .collect();
        self.windows = windows;
        let order: Vec<_> = self.windows.iter().map(|w| w.handle).collect();
        let act = DisplayAction::SetWindowOrder(order);
        self.actions.push_back(act);
    }

    pub fn move_to_top(&mut self, handle: &WindowHandle) -> Option<()> {
        let index = self.windows.iter().position(|w| &w.handle == handle)?;
        let window = self.windows.remove(index);
        self.windows.insert(0, window);
        self.sort_windows();
        Some(())
    }

    #[must_use]
    pub fn workspaces_display(&self) -> String {
        let mut focused_id = -1;
        if let Some(f) = self.focused_workspace() {
            focused_id = f.id;
        }
        let list: Vec<String> = self
            .workspaces
            .iter()
            .map(|w| {
                let tags = w.tags.join(",");
                if w.id == focused_id {
                    format!("({})", tags)
                } else {
                    format!(" {} ", tags)
                }
            })
            .collect();
        list.join(" ")
    }

    #[must_use]
    pub fn windows_display(&self) -> String {
        let list: Vec<String> = self
            .windows
            .iter()
            .map(|w| {
                let tags = w.tags.join(",");
                format!("[{:?}:{}]", w.handle, tags)
            })
            .collect();
        list.join(" ")
    }

    /// Soft reload the worker.
    ///
    /// First write current state to a file and then exit current process.
    pub fn soft_reload(&mut self) {
        if let Err(err) = state::save(self) {
            log::error!("Cannot save state: {}", err);
        }
        self.hard_reload();
    }

    /// Reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }
}
