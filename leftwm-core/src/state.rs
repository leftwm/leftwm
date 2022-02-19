//! Save and restore manager state.

use crate::config::{Config, InsertBehavior, ScratchPad};
use crate::layouts::Layout;
use crate::models::Screen;
use crate::models::Size;
use crate::models::Tags;
use crate::models::Window;
use crate::models::Workspace;
use crate::models::{FocusManager, LayoutManager};
use crate::models::{Mode, WindowHandle};
use crate::DisplayAction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

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
    pub tags: Tags, // List of all known tags.
    pub mousekey: Vec<String>,
    pub max_window_width: Option<Size>,
    pub default_width: i32,
    pub default_height: i32,
    pub disable_tile_drag: bool,
    pub insert_behavior: InsertBehavior,
}

impl State {
    pub(crate) fn new(config: &impl Config) -> Self {
        let layout_manager = LayoutManager::new(config);
        let mut tags = Tags::new();
        config.create_list_of_tag_labels().iter().for_each(|label| {
            tags.add_new(label.as_str(), layout_manager.new_layout(None));
        });
        tags.add_new_hidden("NSP");

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
            tags,
            max_window_width: config.max_window_width(),
            mousekey: config.mousekey(),
            default_width: config.default_width(),
            default_height: config.default_height(),
            disable_tile_drag: config.disable_tile_drag(),
            insert_behavior: config.insert_behavior(),
        }
    }

    //sorts the windows and puts them in order of importance
    //keeps the order for each importance level
    pub fn sort_windows(&mut self) {
        use crate::models::WindowType;
        //first dialogs and modals
        let (level1, other): (Vec<&Window>, Vec<&Window>) = self.windows.iter().partition(|w| {
            w.r#type == WindowType::Dialog
                || w.r#type == WindowType::Splash
                || w.r#type == WindowType::Utility
                || w.r#type == WindowType::Menu
        });

        //next floating
        let (level2, other): (Vec<&Window>, Vec<&Window>) = other
            .iter()
            .partition(|w| w.r#type == WindowType::Normal && w.floating());

        //then normal windows
        let (level3, other): (Vec<&Window>, Vec<&Window>) =
            other.iter().partition(|w| w.r#type == WindowType::Normal);

        //last docks
        //other is all the reset

        //build the updated window list
        self.windows = level1
            .iter()
            .chain(level2.iter())
            .chain(level3.iter())
            .chain(other.iter())
            .map(|&w| w.clone())
            .collect();
        let act = DisplayAction::SetWindowOrder(self.windows.clone());
        self.actions.push_back(act);
    }

    pub fn move_to_top(&mut self, handle: &WindowHandle) -> Option<()> {
        let index = self.windows.iter().position(|w| &w.handle == handle)?;
        let window = self.windows.remove(index);
        self.windows.insert(0, window);
        self.sort_windows();
        Some(())
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
            config.load_window(win);
        }
        for ws in &mut self.workspaces {
            ws.load_config(config);
        }
    }

    /// Apply saved state to a running manager.
    pub fn restore_state(&mut self, state: &Self) {
        // Restore tags.
        for old_tag in state.tags.all() {
            if let Some(tag) = self.tags.get_mut(old_tag.id) {
                tag.hidden = old_tag.hidden;
                tag.layout = old_tag.layout;
                tag.layout_rotation = old_tag.layout_rotation;
                tag.flipped_vertical = old_tag.flipped_vertical;
                tag.flipped_horizontal = old_tag.flipped_horizontal;
                tag.main_width_percentage = old_tag.main_width_percentage;
            }
        }

        let are_tags_equal = self.tags.all().eq(&state.tags.all());

        // Restore windows.
        let mut ordered = vec![];
        let mut had_strut = false;
        state.windows.iter().for_each(|old_window| {
            if let Some((index, new_window)) = self
                .windows
                .clone()
                .iter_mut()
                .enumerate()
                .find(|w| w.1.handle == old_window.handle)
            {
                had_strut = old_window.strut.is_some() || had_strut;

                new_window.set_floating(old_window.floating());
                new_window.set_floating_offsets(old_window.get_floating_offsets());
                new_window.apply_margin_multiplier(old_window.margin_multiplier);
                new_window.pid = old_window.pid;
                new_window.normal = old_window.normal;
                if are_tags_equal {
                    new_window.tags = old_window.tags.clone();
                } else {
                    let mut new_tags = old_window.tags.clone();
                    // Only retain the tags, that still exist.
                    new_tags.retain(|&tag_id| self.tags.get(tag_id).is_some());
                    // If there are no tags, add tag '1', so the window will not be lost.
                    if new_tags.is_empty() {
                        new_tags.push(1);
                    }
                    new_window.clear_tags();
                    new_tags.iter().for_each(|&tag_id| new_window.tag(&tag_id));
                }
                new_window.strut = old_window.strut;
                new_window.set_states(old_window.states());
                ordered.push(new_window.clone());
                self.windows.remove(index);

                // Make the x server aware of any tag changes for the window.
                let act = DisplayAction::SetWindowTags(new_window.handle, new_window.tags.clone());
                self.actions.push_back(act);
            }
        });
        if had_strut {
            self.update_static();
        }
        self.windows.append(&mut ordered);

        // This is needed due to mutable/immutable borrows.
        let tags = &self.tags;

        // Restore workspaces.
        for workspace in &mut self.workspaces {
            if let Some(old_workspace) = state.workspaces.iter().find(|w| w.id == workspace.id) {
                workspace.layout = old_workspace.layout;
                workspace.main_width_percentage = old_workspace.main_width_percentage;
                workspace.margin_multiplier = old_workspace.margin_multiplier;
                if are_tags_equal {
                    workspace.tags = old_workspace.tags.clone();
                } else {
                    let mut new_tags = old_workspace.tags.clone();
                    // Only retain the tags, that still exist.
                    new_tags.retain(|&tag_id| tags.get(tag_id).is_some());
                    // If there are no tags, add tag '1', so the workspace has a tag.
                    if new_tags.is_empty() {
                        new_tags.push(1);
                    }
                    new_tags
                        .iter()
                        .for_each(|&tag_id| workspace.tags = vec![tag_id]);
                }
            }
        }

        // Restore scratchpads.
        for (scratchpad, id) in &state.active_scratchpads {
            self.active_scratchpads.insert(scratchpad.clone(), *id);
        }

        // Restore focus.
        self.focus_manager.tags_last_window = state.focus_manager.tags_last_window.clone();
        self.focus_manager
            .tags_last_window
            .retain(|&id, _| tags.get(id).is_some());
        let tag_id = match state.focus_manager.tag(0) {
            // If the tag still exists it should be displayed on a workspace.
            Some(tag_id) if self.tags.get(tag_id).is_some() => tag_id,
            // If the tag doesn't exist, tag 1 should be displayed on a workspace.
            Some(_) => 1,
            // If we don't have any tag history (We should), focus the tag on workspace 1.
            None => match self.workspaces.first() {
                Some(ws) => ws.tags[0],
                // This should never happen.
                None => 1,
            },
        };
        self.focus_tag(&tag_id);
    }
}
