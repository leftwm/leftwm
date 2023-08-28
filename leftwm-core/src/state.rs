//! Save and restore manager state.

use crate::child_process::ChildID;
use crate::config::{Config, InsertBehavior, ScratchPad};
use crate::layouts::LayoutManager;
use crate::models::{
    FocusManager, Mode, ScratchPadName, Screen, TagId, Tags, Window, WindowHandle, WindowType,
    Workspace,
};
use crate::DisplayAction;
use leftwm_layouts::Layout;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub window_history: HashMap<TagId, Vec<WindowHandle>>,
    pub workspaces: Vec<Workspace>,
    pub focus_manager: FocusManager,
    pub layout_manager: LayoutManager,
    pub mode: Mode,
    pub layout_definitions: Vec<Layout>,
    pub scratchpads: Vec<ScratchPad>,
    pub active_scratchpads: HashMap<ScratchPadName, VecDeque<ChildID>>,
    pub actions: VecDeque<DisplayAction>,
    pub tags: Tags, // List of all known tags.
    pub mousekey: Vec<String>,
    pub default_width: i32,
    pub default_height: i32,
    pub disable_tile_drag: bool,
    pub reposition_cursor_on_resize: bool,
    pub insert_behavior: InsertBehavior,
    pub single_window_border: bool,
}

impl State {
    pub(crate) fn new(config: &impl Config) -> Self {
        let mut tags = Tags::new();
        config.create_list_of_tag_labels().iter().for_each(|label| {
            tags.add_new(label.as_str());
        });
        tags.add_new_hidden("NSP");

        Self {
            window_history: HashMap::new(),
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
        // The windows we are managing should be behind unmanaged windows. Unless they are
        // fullscreen, or their children.
        // Fullscreen windows.
        let (level2, fullscreen_windows, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(self.windows.iter(), Window::is_fullscreen);

        // Fullscreen windows children.
        let (level1, fullscreen_children, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(other.iter(), |w| {
                level2.contains(&w.transient.unwrap_or_else(|| 0.into()))
            });

        // Left over managed windows.
        // Dialogs and modals.
        let (level3, dialogs, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(other.iter(), |w| {
                w.r#type == WindowType::Dialog
                    || w.r#type == WindowType::Splash
                    || w.r#type == WindowType::Utility
                    || w.r#type == WindowType::Menu
            });

        // Floating windows.
        let (level4, floating, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(other.iter(), |w| {
                w.r#type == WindowType::Normal && w.floating()
            });

        // Maximized windows.
        let (level5, maximized, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(other.iter(), |w| {
                w.r#type == WindowType::Normal && w.is_maximized()
            });

        // Tiled windows.
        let (level6, tiled, other): (Vec<WindowHandle>, Vec<Window>, Vec<Window>) =
            partition_windows(other.iter(), |w| w.r#type == WindowType::Normal);

        // Last docks.
        let level7: Vec<WindowHandle> = other.iter().map(|w| w.handle).collect();

        self.windows = [
            fullscreen_children,
            fullscreen_windows,
            dialogs,
            floating,
            maximized,
            tiled,
            other,
        ]
        .concat();

        let fullscreen: Vec<WindowHandle> = [level1, level2].concat();
        let handles: Vec<WindowHandle> = [level3, level4, level5, level6, level7].concat();
        let act = DisplayAction::SetWindowOrder(fullscreen, handles);
        self.actions.push_back(act);
    }

    pub fn handle_single_border(&mut self, border_width: i32) {
        if self.single_window_border {
            return;
        }

        for tag in self.tags.normal() {
            let mut windows_on_tag: Vec<&mut Window> = self
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
                new_window.set_states(old_window.states());
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

fn partition_windows<'a, I, F>(windows: I, f: F) -> (Vec<WindowHandle>, Vec<Window>, Vec<Window>)
where
    I: Iterator<Item = &'a Window>,
    F: FnMut(&Window) -> bool + 'a,
{
    #[inline]
    fn extend<'a>(
        mut f: impl FnMut(&Window) -> bool + 'a,
        handles: &'a mut Vec<WindowHandle>,
        left: &'a mut Vec<Window>,
        right: &'a mut Vec<Window>,
    ) -> impl FnMut((), &Window) + 'a {
        move |(), x| {
            if f(x) {
                handles.push(x.handle);
                left.push(x.clone());
            } else {
                right.push(x.clone());
            }
        }
    }

    let mut handles: Vec<WindowHandle> = Default::default();
    let mut left: Vec<Window> = Default::default();
    let mut right: Vec<Window> = Default::default();
    windows.fold((), extend(f, &mut handles, &mut left, &mut right));
    (handles, left, right)
}
