//! Save and restore manager state.

use crate::config::{Config, ScratchPad};
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
    pub tags: Tags, //list of all known tags
    pub disable_current_tag_swap: bool,
    pub mousekey: String,
    pub max_window_width: Option<Size>,
    pub default_width: i32,
    pub default_height: i32,
}

impl State {
    pub(crate) fn new(config: &impl Config) -> Self {
        let layout_manager = LayoutManager::new(config);
        let mut tags = Tags::new();
        config.create_list_of_tag_labels().iter().for_each(|label| {
            tags.add_new(label.as_str(), layout_manager.new_layout());
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
            frame_rate_limitor: Default::default(),
            tags,
            disable_current_tag_swap: config.disable_current_tag_swap(),
            max_window_width: config.max_window_width(),
            mousekey: config.mousekey(),
            default_width: config.default_width(),
            default_height: config.default_height(),
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

    /// Apply saved state to a running manager.
    pub fn restore_state(&mut self, state: &State) {
        // restore workspaces
        for workspace in &mut self.workspaces {
            if let Some(old_workspace) = state.workspaces.iter().find(|w| w.id == workspace.id) {
                workspace.layout = old_workspace.layout;
                workspace.main_width_percentage = old_workspace.main_width_percentage;
                workspace.margin_multiplier = old_workspace.margin_multiplier;
            }
        }

        for tag in &mut self.tags.all() {
            if let Some(old_tag) = state.tags.get(tag.id) {
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
                if self.tags.all().eq(&state.tags.all()) {
                    new_window.tags = old_window.tags.clone();
                } else {
                    let mut new_tags = old_window.tags.clone();
                    // only retain the tags, that still exist
                    new_tags.retain(|&tag_id| self.tags.get(tag_id).is_some());
                    // if there are no tags, add tag '1', so the window will not be lost
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
