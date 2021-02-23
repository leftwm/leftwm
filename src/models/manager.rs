use crate::config::ThemeSetting;
use crate::display_action::DisplayAction;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::Tag;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::state;
use crate::utils::child_process::Children;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Manager {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub mode: Mode,
    pub theme_setting: ThemeSetting,
    #[serde(skip)]
    pub tags: Vec<Tag>, //list of all known tags
    pub focused_workspace_history: VecDeque<usize>,
    pub focused_window_history: VecDeque<WindowHandle>,
    pub focused_tag_history: VecDeque<String>,
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
    pub fn focused_workspace(&self) -> Option<&Workspace> {
        if self.focused_workspace_history.is_empty() {
            return None;
        }
        let index = self.focused_workspace_history[0];
        Some(&self.workspaces[index])
    }

    /// Return the currently focused workspace.
    pub fn focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        if self.focused_workspace_history.is_empty() {
            return None;
        }
        let index = self.focused_workspace_history[0];
        Some(&mut self.workspaces[index])
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    pub fn focused_tag(&self, offset: usize) -> Option<String> {
        self.focused_tag_history.get(offset).map(|t| t.to_string())
    }

    /// Return the index of a given tag.
    pub fn tag_index(&self, tag: String) -> Option<usize> {
        Some(self.tags.iter().position(|t| t.id == tag)).unwrap_or(None)
    }

    /// Return the currently focused window.
    pub fn focused_window(&self) -> Option<&Window> {
        if self.focused_window_history.is_empty() {
            return None;
        }
        let handle = self.focused_window_history[0].clone();
        for w in &self.windows {
            if handle == w.handle {
                return Some(w);
            }
        }
        None
    }

    /// Return the currently focused window.
    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        if self.focused_window_history.is_empty() {
            return None;
        }
        let handle = self.focused_window_history[0].clone();
        for w in &mut self.windows {
            if handle == w.handle {
                return Some(w);
            }
        }
        None
    }

    pub fn tags_display(&self) -> String {
        let mut active: Vec<String> = vec![];
        for w in &self.workspaces {
            active.extend(w.tags.clone())
        }
        let mut wraps = vec![('<', '>'), ('(', ')'), ('{', '}'), ('[', ']')];
        let parts: Vec<String> = self
            .tags
            .iter()
            .map(|t| {
                if active.contains(&t.id) {
                    let wrap = wraps.pop().unwrap();
                    format!("{}{}{}", wrap.0, &t.id, wrap.1)
                } else {
                    format!(" {} ", &t.id)
                }
            })
            .collect();
        parts.join(" | ")
    }

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
