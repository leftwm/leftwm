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
