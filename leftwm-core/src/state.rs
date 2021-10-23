//! Save and restore manager state.

use crate::config::{Config, ScratchPad};
use crate::layouts::Layout;
use crate::models::Screen;
use crate::models::Size;
use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;
use crate::models::{FocusManager, LayoutManager};
use crate::models::{Mode, WindowHandle};
use crate::DisplayAction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::os::raw::c_ulong;

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub focus_manager: FocusManager,
    pub layout_manager: LayoutManager,
    pub mode: Mode,
    pub layouts: Vec<Layout>,
    pub scratchpads: Vec<ScratchPad>,
    pub active_scratchpads: HashMap<String, Option<u32>>,
    pub actions: VecDeque<DisplayAction>,
    // TODO should this really be saved in the state?
    //this is used to limit framerate when resizing/moving windows
    pub frame_rate_limitor: c_ulong,
    pub tags: Vec<Tag>, //list of all known tags
    pub disable_current_tag_swap: bool,
    pub mousekey: String,
    pub max_window_width: Option<Size>,
}

impl State {
    pub(crate) fn new(config: &impl Config) -> Self {
        let layout_manager = LayoutManager::new(config);
        let mut tags: Vec<Tag> = config
            .create_list_of_tags()
            .iter()
            .map(|s| Tag::new(s, layout_manager.new_layout()))
            .collect();
        tags.push(Tag {
            id: "NSP".to_owned(),
            hidden: true,
            ..Tag::default()
        });

        Self {
            focus_manager: FocusManager::new(config),
            layout_manager,
            scratchpads: config.create_list_of_scratchpads(),
            layouts: config.layouts(),
            screens: Default::default(),
            windows: Default::default(),
            workspaces: Default::default(),
            mode: Default::default(),
            active_scratchpads: Default::default(),
            actions: Default::default(),
            frame_rate_limitor: Default::default(),
            tags,
            disable_current_tag_swap: config.disable_current_tag_swap(),
            max_window_width: config.max_window_width(),
            mousekey: config.mousekey(),
        }
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
        if let Some(f) = self.focus_manager.workspace(&self.workspaces) {
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

    /// Return the index of a given tag.
    #[must_use]
    pub fn tag_index(&self, tag: &str) -> Option<usize> {
        Some(self.tags.iter().position(|t| t.id == tag)).unwrap_or(None)
    }

    pub fn update_static(&mut self) {
        let workspaces = self.workspaces.clone();
        self.windows
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

    pub(crate) fn load_config(&mut self, config: &impl Config) {
        self.mousekey = config.mousekey();
        self.max_window_width = config.max_window_width();
        for win in &mut self.windows {
            win.load_config(config);
        }
        for ws in &mut self.workspaces {
            ws.load_config(config);
        }
    }

    // TODO probably the state should be replace immutably instead of mutated
    /// Apply saved state to a running manager.
    pub fn restore_state(&mut self, state: &State) {
        // restore workspaces
        for workspace in &mut self.workspaces {
            if let Some(old_workspace) = state.workspaces.iter().find(|w| w.id == workspace.id) {
                workspace.layout = old_workspace.layout;
                workspace.margin_multiplier = old_workspace.margin_multiplier;
            }
        }

        // restore tags
        for tag in &mut self.tags {
            if let Some(old_tag) = state.tags.iter().find(|t| t.id == tag.id) {
                tag.hidden = old_tag.hidden;
                tag.layout = old_tag.layout;
                tag.layout_rotation = old_tag.layout_rotation;
                tag.flipped_vertical = old_tag.flipped_vertical;
                tag.flipped_horizontal = old_tag.flipped_horizontal;
                tag.main_width_percentage = old_tag.main_width_percentage;
            }
        }

        // restore windows
        let mut ordered = vec![];
        let mut had_strut = false;

        state.windows.iter().for_each(|old| {
            if let Some((index, window)) = self
                .windows
                .clone()
                .iter_mut()
                .enumerate()
                .find(|w| w.1.handle == old.handle)
            {
                had_strut = old.strut.is_some() || had_strut;

                window.set_floating(old.floating());
                window.set_floating_offsets(old.get_floating_offsets());
                window.apply_margin_multiplier(old.margin_multiplier);
                window.pid = old.pid;
                window.normal = old.normal;
                if self.tags.eq(&state.tags) {
                    window.tags = old.tags.clone();
                } else {
                    old.tags.iter().for_each(|t| {
                        let manager_tags = &self.tags.clone();
                        if let Some(tag_index) = &state.tags.clone().iter().position(|o| &o.id == t)
                        {
                            window.clear_tags();
                            // if the config prior reload had more tags then the current one
                            // we want to move windows of 'lost tags' to the 'first' tag
                            // also we want to ignore the `NSP` tag for length check
                            if tag_index < &(manager_tags.len() - 1) || t == "NSP" {
                                window.tag(&manager_tags[*tag_index].id);
                            } else if let Some(tag) = manager_tags.first() {
                                window.tag(&tag.id);
                            }
                        }
                    });
                }
                window.strut = old.strut;
                window.set_states(old.states());
                ordered.push(window.clone());
                self.windows.remove(index);
            }
        });
        if had_strut {
            self.update_static();
        }
        self.windows.append(&mut ordered);

        // restore scratchpads
        for (scratchpad, id) in &state.active_scratchpads {
            self.active_scratchpads.insert(scratchpad.clone(), *id);
        }
    }
}
