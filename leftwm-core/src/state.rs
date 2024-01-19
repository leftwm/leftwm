//! Save and restore manager state.

use crate::child_process::ChildID;
use crate::config::{Config, InsertBehavior, ScratchPad};
use crate::layouts::LayoutManager;
use crate::models::{
    FocusManager, Mode, ScratchPadName, Screen, Tags, Window, WindowHandle, WindowState,
    WindowType, Workspace, Handle,
};
use crate::DisplayAction;
use leftwm_layouts::Layout;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Serialize, Deserialize, Debug)]
pub struct State<H: Handle> {
    #[serde(bound = "")]
    pub screens: Vec<Screen<H>>,
    #[serde(bound = "")]
    pub windows: Vec<Window<H>>,
    pub workspaces: Vec<Workspace>,
    #[serde(bound = "")]
    pub focus_manager: FocusManager<H>,
    pub layout_manager: LayoutManager,
    #[serde(bound = "")]
    pub mode: Mode<H>,
    pub layout_definitions: Vec<Layout>,
    pub scratchpads: Vec<ScratchPad>,
    pub active_scratchpads: HashMap<ScratchPadName, VecDeque<ChildID>>,
    #[serde(bound = "")]
    pub actions: VecDeque<DisplayAction<H>>,
    pub tags: Tags, // List of all known tags.
    pub mousekey: Vec<String>,
    pub default_width: i32,
    pub default_height: i32,
    pub disable_tile_drag: bool,
    pub reposition_cursor_on_resize: bool,
    pub insert_behavior: InsertBehavior,
    pub single_window_border: bool,
}

impl<H: Handle> State<H> {
    pub(crate) fn new(config: &impl Config) -> Self {
        let mut tags = Tags::new();
        config.create_list_of_tag_labels().iter().for_each(|label| {
            tags.add_new(label.as_str());
        });
        tags.add_new_hidden("NSP");

        Self {
            focus_manager: FocusManager::new(config),
            layout_manager: LayoutManager::new(config),
            scratchpads: config.create_list_of_scratchpads(),
            layout_definitions: config.layout_definitions(),
            screens: Default::default(),
            windows: Default::default(),
            workspaces: Default::default(),
            mode: Default::default(),
            active_scratchpads: Default::default(),
            actions: Default::default(),
            tags,
            mousekey: config.mousekey(),
            default_width: config.default_width(),
            default_height: config.default_height(),
            disable_tile_drag: config.disable_tile_drag(),
            reposition_cursor_on_resize: config.reposition_cursor_on_resize(),
            insert_behavior: config.insert_behavior(),
            single_window_border: config.single_window_border(),
        }
    }

    // Sorts the windows and puts them in order of importance.
    pub fn sort_windows(&mut self) {
        let mut sorter = WindowSorter::new(self.windows.iter().collect());

        // Windows explicitly marked as on top
        sorter.sort(|w| w.states.contains(&WindowState::Above) && w.floating());

        // Transient windows should be above a fullscreen/maximized parent
        sorter.sort(|w| {
            w.transient.is_some_and(|trans| {
                self.windows
                    .iter()
                    .any(|w| w.handle == trans && (w.is_fullscreen() || w.is_maximized()))
            })
        });

        // Fullscreen windows
        sorter.sort(Window::is_fullscreen);

        // Dialogs and modals.
        sorter.sort(|w| {
            w.r#type == WindowType::Dialog
                || w.r#type == WindowType::Splash
                || w.r#type == WindowType::Utility
                || w.r#type == WindowType::Menu
        });

        // Floating windows.
        sorter.sort(|w| w.r#type == WindowType::Normal && w.floating());

        // Maximized windows.
        sorter.sort(|w| w.r#type == WindowType::Normal && w.is_maximized());

        // Tiled windows.
        sorter.sort(|w| w.r#type == WindowType::Normal);

        // Last docks.
        sorter.sort(|w| w.r#type == WindowType::Dock);

        // Finish and put all unsorted at the end.
        let windows = sorter.finish();
        let handles = windows.iter().map(|w| w.handle).collect();

        let act = DisplayAction::SetWindowOrder(handles);
        self.actions.push_back(act);
    }

    pub fn handle_single_border(&mut self, border_width: i32) {
        if self.single_window_border {
            return;
        }

        for tag in self.tags.normal() {
            let mut windows_on_tag: Vec<&mut Window<H>> = self
                .windows
                .iter_mut()
                .filter(|w| w.tag.unwrap_or(0) == tag.id && w.r#type == WindowType::Normal)
                .collect();

            let wsid = self
                .workspaces
                .iter()
                .find(|ws| ws.has_tag(&tag.id))
                .map(|w| w.id);
            let layout = self.layout_manager.layout(wsid.unwrap_or(1), tag.id);
            if layout.is_monocle() {
                windows_on_tag.iter_mut().for_each(|w| w.border = 0);
                continue;
            }

            if windows_on_tag.len() == 1 {
                if let Some(w) = windows_on_tag.first_mut() {
                    w.border = 0;
                }
                continue;
            }

            windows_on_tag
                .iter_mut()
                .for_each(|w| w.border = border_width);
        }
    }

    pub fn move_to_top(&mut self, handle: &WindowHandle<H>) -> Option<()> {
        let index = self.windows.iter().position(|w| &w.handle == handle)?;
        let window = self.windows.remove(index);
        self.windows.insert(0, window);
        self.sort_windows();
        Some(())
    }

    pub fn update_static(&mut self) {
        self.windows
            .iter_mut()
            .filter(|w| w.strut.is_some() || w.is_sticky())
            .for_each(|w| {
                let (x, y) = match w.strut {
                    Some(strut) => strut.center(),
                    None => w.calculated_xyhw().center(),
                };
                if let Some(ws) = self.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
                    w.tag = ws.tag;
                }
            });
    }

    pub(crate) fn load_config(&mut self, config: &impl Config) {
        self.mousekey = config.mousekey();
        for win in &mut self.windows {
            config.load_window(win);
        }
        for ws in &mut self.workspaces {
            ws.load_config(config);
        }
    }

    /// Apply saved state to a running manager.
    pub fn restore_state(&mut self, old_state: &Self) {
        tracing::debug!("Restoring old state");

        // Restore tags.
        for old_tag in old_state.tags.all() {
            if let Some(tag) = self.tags.get_mut(old_tag.id) {
                tag.hidden = old_tag.hidden;
            }
        }

        let are_tags_equal = self.tags.all().eq(&old_state.tags.all());

        // Restore windows.
        let mut ordered = vec![];
        let mut had_strut = false;
        old_state.windows.iter().for_each(|old_window| {
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
                    new_window.tag = old_window.tag;
                } else {
                    let mut new_tag = old_window.tag;
                    // Only retain the tag if it still exists, otherwise default to tag 1
                    match new_tag {
                        Some(tag) if self.tags.get(tag).is_some() => {}
                        _ => new_tag = Some(1),
                    }
                    new_window.untag();
                    new_tag.iter().for_each(|&tag_id| new_window.tag(&tag_id));
                }
                new_window.strut = old_window.strut;
                new_window.states = old_window.states.clone();
                ordered.push(new_window.clone());
                self.windows.remove(index);

                // Make the x server aware of any tag changes for the window.
                let act = DisplayAction::SetWindowTag(new_window.handle, new_window.tag);
                self.actions.push_back(act);
            }
        });
        if had_strut {
            self.update_static();
        }
        self.windows.append(&mut ordered);

        // This is needed due to mutable/immutable borrows.
        let all_tags = &self.tags;

        // Restore workspaces.
        for workspace in &mut self.workspaces {
            if let Some(old_workspace) = old_state.workspaces.iter().find(|w| w.id == workspace.id)
            {
                workspace.margin_multiplier = old_workspace.margin_multiplier;
                if are_tags_equal {
                    workspace.tag = old_workspace.tag;
                } else {
                    let mut new_tag = old_workspace.tag;
                    // Only retain the tag if it still exists, otherwise default to tag 1
                    match new_tag {
                        Some(tag) if all_tags.get(tag).is_some() => {}
                        _ => new_tag = Some(1),
                    }
                    new_tag
                        .iter()
                        .for_each(|&tag_id| workspace.tag = Some(tag_id));
                }
            }
        }

        // Restore scratchpads.
        for (scratchpad, id) in &old_state.active_scratchpads {
            self.active_scratchpads
                .insert(scratchpad.clone(), id.clone());
        }

        // Restore focus.
        self.focus_manager.tags_last_window = old_state.focus_manager.tags_last_window.clone();
        self.focus_manager
            .tags_last_window
            .retain(|&id, _| all_tags.get(id).is_some());
        let tag_id = match old_state.focus_manager.tag(0) {
            // If the tag still exists it should be displayed on a workspace.
            Some(tag_id) if self.tags.get(tag_id).is_some() => tag_id,
            // If the tag doesn't exist, tag 1 should be displayed on a workspace.
            Some(_) => 1,
            // If we don't have any tag history (We should), focus the tag on workspace 1.
            None => match self.workspaces.first() {
                Some(ws) => ws.tag.unwrap_or(1),
                // This should never happen.
                _ => 1,
            },
        };
        self.focus_tag(&tag_id);

        // Restore layout manager
        self.layout_manager.restore(&old_state.layout_manager);
    }
}

struct WindowSorter<'a, H: Handle> {
    stack: Vec<&'a Window<H>>,
    unsorted: Vec<&'a Window<H>>,
}

impl<'a, H: Handle> WindowSorter<'a, H> {
    pub fn new(windows: Vec<&'a Window<H>>) -> Self {
        Self {
            stack: Vec::with_capacity(windows.len()),
            unsorted: windows,
        }
    }

    pub fn sort<F: Fn(&Window<H>) -> bool>(&mut self, filter: F) {
        self.unsorted.retain(|window| {
            if filter(window) {
                self.stack.push(window);
                false
            } else {
                true
            }
        });
    }

    pub fn finish(mut self) -> Vec<&'a Window<H>> {
        self.stack.append(&mut self.unsorted);
        self.stack
    }
}
