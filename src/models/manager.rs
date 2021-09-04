use crate::config::{Config, ScratchPad, ThemeSetting};
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::models::FocusManager;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::Tag;
use crate::models::Window;
use crate::models::WindowHandle;
use crate::models::Workspace;
use crate::utils::child_process::Children;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::os::raw::c_ulong;
use std::sync::{atomic::AtomicBool, Arc};

/// Maintains current program state.
#[derive(Serialize, Deserialize, Debug)]
pub struct Manager<CMD> {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub focus_manager: FocusManager,
    pub mode: Mode,
    pub theme_setting: Arc<ThemeSetting>,
    #[serde(skip)]
    pub tags: Vec<Tag>, //list of all known tags
    pub layouts: Vec<Layout>,
    pub scratchpads: Vec<ScratchPad>,
    pub active_scratchpads: HashMap<String, Option<u32>>,
    pub actions: VecDeque<DisplayAction>,

    //this is used to limit framerate when resizing/moving windows
    pub frame_rate_limitor: c_ulong,
    #[serde(skip)]
    pub children: Children,
    #[serde(skip)]
    pub reap_requested: Arc<AtomicBool>,
    #[serde(skip)]
    pub reload_requested: bool,
    #[serde(skip)]
    pub marker: PhantomData<CMD>,
}

impl<CMD> Manager<CMD> {
    pub fn new(config: &impl Config, theme_setting: Arc<ThemeSetting>) -> Self {
        let mut tags: Vec<Tag> = config
            .create_list_of_tags()
            .iter()
            .map(|s| Tag::new(s))
            .collect();
        tags.push(Tag {
            id: "NSP".to_owned(),
            hidden: true,
            ..Tag::default()
        });

        Self {
            theme_setting,
            tags,
            focus_manager: FocusManager::new(config),
            scratchpads: config.create_list_of_scratchpads(),
            layouts: config.layouts(),
            screens: Default::default(),
            windows: Default::default(),
            workspaces: Default::default(),
            mode: Default::default(),
            active_scratchpads: Default::default(),
            actions: Default::default(),
            frame_rate_limitor: Default::default(),
            children: Default::default(),
            reap_requested: Default::default(),
            reload_requested: Default::default(),
            marker: PhantomData,
        }
    }

    /// Return the currently focused workspace.
    #[must_use]
    pub fn focused_workspace(&self) -> Option<&Workspace> {
        self.focus_manager.workspace(self)
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
        self.focus_manager.window(self)
    }

    /// Return the currently focused window.
    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        self.focus_manager.window_mut(&mut self.windows)
    }

    pub fn update_docks(&mut self) {
        let workspaces = self.workspaces.clone();
        self.windows
            .iter_mut()
            .filter(|w| w.strut.is_some())
            .for_each(|w| {
                let (x, y) = w.strut.unwrap_or_default().center();
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
        let mut focused_id = None;
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

    /// Reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }
}

#[cfg(test)]
impl Manager<()> {
    pub fn new_test(tags: Vec<String>) -> Self {
        use crate::config::TestConfig;
        use crate::models::Margins;

        let theme_setting = Arc::new(ThemeSetting {
            border_width: Default::default(),
            margin: Margins::Int(0),
            workspace_margin: None,
            gutter: Default::default(),
            default_border_color: Default::default(),
            floating_border_color: Default::default(),
            focused_border_color: Default::default(),
            on_new_window_cmd: Default::default(),
        });

        Manager::new(&TestConfig { tags }, theme_setting)
    }
}
