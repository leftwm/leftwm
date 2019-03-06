use crate::display_action::DisplayAction;
use crate::models::Screen;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Manager {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>, //list of all known tags
    pub focused_workspace_history: VecDeque<usize>,
    pub focused_window_history: VecDeque<WindowHandle>,
    pub focused_tag_history: VecDeque<String>,
    pub actions: VecDeque<DisplayAction>,
}

impl Default for Manager {
    fn default() -> Manager {
        Manager {
            windows: Vec::new(),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            focused_workspace_history: VecDeque::new(),
            focused_window_history: VecDeque::new(),
            focused_tag_history: VecDeque::new(),
            actions: VecDeque::new(),
        }
    }
}

impl Manager {
    /*
     * return the currently focused workspace
     */
    pub fn focused_workspace(&self) -> Option<&Workspace> {
        if self.focused_workspace_history.is_empty() {
            return None;
        }
        let index = self.focused_workspace_history[0];
        Some(&self.workspaces[index])
    }

    /*
     * return the currently focused workspace
     */
    pub fn focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        if self.focused_workspace_history.is_empty() {
            return None;
        }
        let index = self.focused_workspace_history[0];
        Some(&mut self.workspaces[index])
    }

    /*
     * return the currently focused tag
     */
    pub fn focused_tag(&self) -> Option<String> {
        if self.focused_tag_history.is_empty() {
            return None;
        }
        Some(self.focused_tag_history[0].clone())
    }

    /*
     * return the currently focused window
     */
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

    /*
     * return the currently focused window
     */
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
                if active.contains(t) {
                    let wrap = wraps.pop().unwrap();
                    format!("{}{}{}", wrap.0, t, wrap.1)
                } else {
                    format!(" {} ", t)
                }
            })
            .collect();
        parts.join(" | ")
    }

    pub fn workspaces_display(&self) -> String {
        let mut focused_id = -1;
        if let Some(f) = self.focused_workspace() {
            focused_id = f.id.clone();
        }
        let list: Vec<String> = self
            .workspaces
            .iter()
            .map(|w| {
                let tags = w.tags.join(",");
                if w.id == focused_id {
                    format!("({})", tags)
                } else {
                    format!("[{}]", tags)
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
}
