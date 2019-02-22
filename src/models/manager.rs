use super::config;
use super::Screen;
use super::Window;
use super::WindowHandle;
use super::Workspace;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Manager {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>, //list of all known tags
    pub config: config::Config,
    focused_workspace_history: VecDeque<usize>,
    focused_window_history: VecDeque<WindowHandle>,
    focused_tag_history: VecDeque<String>,
}

impl Default for Manager {
    fn default() -> Manager {
        let config = config::load();
        let mut h = VecDeque::new();
        h.push_front(0);
        Manager {
            windows: Vec::new(),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            focused_workspace_history: h,
            focused_window_history: VecDeque::new(),
            focused_tag_history: VecDeque::new(),
            config,
        }
    }
}

impl Manager {

    /*
     * return the currently focused workspace
     */
    pub fn focused_workspace(&mut self) -> Option<&mut Workspace> {
        if self.focused_workspace_history.len() == 0 { return None }
        let index = self.focused_workspace_history[0];
        Some( &mut self.workspaces[index] )
    }

    /*
     * return the currently focused tag
     */
    pub fn focused_tag(&mut self) -> Option<String> {
        if self.focused_tag_history.len() == 0 { return None }
        Some( self.focused_tag_history[0].clone() )
    }

    /*
     * return the currently focused window
     */
    pub fn focused_window(&mut self) -> Option<&mut Window> {
        if self.focused_window_history.len() == 0 { return None }
        let handle = self.focused_window_history[0].clone();
        for w in &mut self.windows {
            if handle == w.handle {
                return Some(w);
            }
        }
        None
    }

}
