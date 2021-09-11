//! Save and restore manager state.

use crate::config::{Config, ScratchPad};
use crate::layouts::Layout;
use crate::models::FocusManager;
use crate::models::Mode;
use crate::models::Screen;
use crate::models::Window;
use crate::models::Workspace;
use crate::{DisplayAction, DisplayServer, Manager};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::os::raw::c_ulong;

#[derive(Serialize, Deserialize, Debug)]
pub struct State<C, CMD> {
    pub screens: Vec<Screen>,
    pub windows: Vec<Window>,
    pub workspaces: Vec<Workspace>,
    pub focus_manager: FocusManager,
    pub mode: Mode,
    // TODO should this really be saved in the state?
    pub config: C,
    pub layouts: Vec<Layout>,
    pub scratchpads: Vec<ScratchPad>,
    pub active_scratchpads: HashMap<String, Option<u32>>,
    pub actions: VecDeque<DisplayAction>,
    // TODO should this really be saved in the state?
    //this is used to limit framerate when resizing/moving windows
    pub frame_rate_limitor: c_ulong,
    marker: PhantomData<CMD>,
}

impl<C, CMD> State<C, CMD>
where
    C: Config<CMD>,
{
    pub(crate) fn new(config: C) -> Self {
        Self {
            focus_manager: FocusManager::new(&config),
            scratchpads: config.create_list_of_scratchpads(),
            layouts: config.layouts(),
            screens: Default::default(),
            windows: Default::default(),
            workspaces: Default::default(),
            mode: Default::default(),
            active_scratchpads: Default::default(),
            actions: Default::default(),
            frame_rate_limitor: Default::default(),
            config,
            marker: PhantomData,
        }
    }
}

impl<C, CMD, SERVER> Manager<C, CMD, SERVER>
where
    C: Config<CMD>,
    SERVER: DisplayServer<CMD>,
{
    /// Apply saved state to a running manager.
    pub fn restore_state(&mut self, state: &State<C, CMD>) {
        // restore workspaces
        for workspace in &mut self.state.workspaces {
            if let Some(old_workspace) = state.workspaces.iter().find(|w| w.id == workspace.id) {
                workspace.layout = old_workspace.layout.clone();
                workspace.margin_multiplier = old_workspace.margin_multiplier;
            }
        }

        // restore windows
        let mut ordered = vec![];
        let mut had_strut = false;

        state.windows.iter().for_each(|old| {
            if let Some((index, window)) = self
                .state
                .windows
                .iter_mut()
                .enumerate()
                .find(|w| w.1.handle == old.handle)
            {
                had_strut = old.strut.is_some() || had_strut;
                if let Some(tag) = old.tags.first() {
                    let act = DisplayAction::SetWindowTags(window.handle, tag.clone());
                    self.state.actions.push_back(act);
                }

                window.set_floating(old.floating());
                window.set_floating_offsets(old.get_floating_offsets());
                window.apply_margin_multiplier(old.margin_multiplier);
                window.pid = old.pid;
                window.normal = old.normal;
                window.tags = old.tags.clone();
                window.strut = old.strut;
                window.set_states(old.states());
                ordered.push(window.clone());
                self.state.windows.remove(index);
            }
        });
        if had_strut {
            self.update_docks();
        }
        self.state.windows.append(&mut ordered);

        // restore scratchpads
        for (scratchpad, id) in &state.active_scratchpads {
            self.state
                .active_scratchpads
                .insert(scratchpad.clone(), *id);
        }
    }
}
