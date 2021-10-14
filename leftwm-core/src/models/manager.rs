use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::state::State;
use crate::utils::child_process::Children;
use std::sync::{atomic::AtomicBool, Arc};

/// Maintains current program state.
#[derive(Debug)]
pub struct Manager<C, SERVER> {
    pub state: State<C>,

    pub(crate) children: Children,
    pub(crate) reap_requested: Arc<AtomicBool>,
    pub(crate) reload_requested: bool,
    pub display_server: SERVER,
}

impl<C, SERVER> Manager<C, SERVER>
where
    C: Config,
    SERVER: DisplayServer,
{
    pub fn new(config: C) -> Self {
        let display_server = SERVER::new(&config);

        Self {
            state: State::new(config),
            children: Default::default(),
            reap_requested: Default::default(),
            reload_requested: false,
            display_server,
        }
    }

    pub fn register_child_hook(&self) {
        crate::child_process::register_child_hook(self.reap_requested.clone());
    }

    /// Return the currently focused workspace.
    #[must_use]
    pub fn focused_workspace(&self) -> Option<&Workspace> {
        self.state.focus_manager.workspace(self)
    }

    /// Return the currently focused workspace.
    pub fn focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.state
            .focus_manager
            .workspace_mut(&mut self.state.workspaces)
    }

    /// Return the currently focused tag if the offset is 0.
    /// Offset is used to reach further down the history.
    #[must_use]
    pub fn focused_tag(&self, offset: usize) -> Option<String> {
        self.state.focus_manager.tag(offset)
    }

    /// Return the index of a given tag.
    #[must_use]
    pub fn tag_index(&self, tag: &str) -> Option<usize> {
        Some(self.state.tags.iter().position(|t| t.id == tag)).unwrap_or(None)
    }

    /// Return the currently focused window.
    #[must_use]
    pub fn focused_window(&self) -> Option<&Window> {
        self.state.focus_manager.window(self)
    }

    /// Return the currently focused window.
    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        self.state.focus_manager.window_mut(&mut self.state.windows)
    }
    
    pub fn update_static(&mut self) {
        let workspaces = self.state.workspaces.clone();
        self.state
            .windows
            .iter_mut()
            .filter(|w| w.strut.is_some() || w.is_sticky())
            .for_each(|w| {
                let (x, y) = match w.strut {
                    Some(strut) => strut.center(),
                    None => w.calculated_xyhw().center(),
                };
                if let Some(ws) = workspaces.iter().find(|ws| ws.contains_point(x, y)) {
                    w.tags = ws.tags.clone();
                }
            });
    }

    //sorts the windows and puts them in order of importance
    //keeps the order for each importance level
    pub fn sort_windows(&mut self) {
        use crate::models::WindowType;
        //first dialogs and modals
        let (level1, other): (Vec<&Window>, Vec<&Window>) =
            self.state.windows.iter().partition(|w| {
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
        self.state.windows = windows;
        let order: Vec<_> = self.state.windows.iter().map(|w| w.handle).collect();
        let act = DisplayAction::SetWindowOrder(order);
        self.state.actions.push_back(act);
    }

    pub fn move_to_top(&mut self, handle: &WindowHandle) -> Option<()> {
        let index = self
            .state
            .windows
            .iter()
            .position(|w| &w.handle == handle)?;
        let window = self.state.windows.remove(index);
        self.state.windows.insert(0, window);
        self.sort_windows();
        Some(())
    }

    #[must_use]
    pub fn workspaces_display(&self) -> String {
        let mut focused_id = None;
        if let Some(f) = self.focused_workspace() {
            focused_id = f.id;
        }
        let list: Vec<String> = self
            .state
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
            .state
            .windows
            .iter()
            .map(|w| {
                let tags = w.tags.join(",");
                format!("[{:?}:{}]", w.handle, tags)
            })
            .collect();
        list.join(" ")
    }

    /// Soft reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }

    pub fn update_for_theme(&mut self) -> bool {
        for win in &mut self.state.windows {
            win.update_for_theme(&self.state.config);
        }
        for ws in &mut self.state.workspaces {
            ws.update_for_theme(&self.state.config);
        }
        true
    }
}

#[cfg(test)]
impl Manager<crate::config::TestConfig, crate::display_servers::MockDisplayServer> {
    pub fn new_test(tags: Vec<String>) -> Self {
        Self::new(crate::config::TestConfig { tags })
    }
}
