//! Save and restore manager state.

use leftwm::errors::Result;
use leftwm::{DisplayAction, Manager};
use std::fs::File;
use std::path::Path;

// TODO: make configurable
/// Path to file where state will be dumper upon soft reload.
const STATE_FILE: &str = "/tmp/leftwm.state";

pub struct State;

impl leftwm::State for State {
    fn save<CMD>(&self, manager: &Manager<CMD>) -> Result<()> {
        let state_file = File::create(STATE_FILE)?;
        serde_json::to_writer(state_file, &manager)?;
        Ok(())
    }

    fn load<CMD>(&self, manager: &mut Manager<CMD>) {
        if Path::new(STATE_FILE).exists() {
            match load_old_state() {
                Ok(old_manager) => restore_state(manager, &old_manager),
                Err(err) => log::error!("Cannot load old state: {}", err),
            }
            // Clean old state.
            if let Err(err) = std::fs::remove_file(STATE_FILE) {
                log::error!("Cannot remove old state file: {}", err);
            }
        }
    }
}

/// Read old state from a state file.
fn load_old_state<CMD>() -> Result<Manager<CMD>> {
    let file = File::open(STATE_FILE)?;
    let old_manager = serde_json::from_reader(file)?;
    Ok(old_manager)
}

/// Apply saved state to a running manager.
fn restore_state<CMD>(manager: &mut Manager<CMD>, old_manager: &Manager<CMD>) {
    restore_workspaces(manager, old_manager);
    restore_windows(manager, old_manager);
    restore_scratchpads(manager, old_manager);
}

/// Restore workspaces layout.
fn restore_workspaces<CMD>(manager: &mut Manager<CMD>, old_manager: &Manager<CMD>) {
    for workspace in &mut manager.workspaces {
        if let Some(old_workspace) = old_manager.workspaces.iter().find(|w| w.id == workspace.id) {
            workspace.layout = old_workspace.layout.clone();
            workspace.margin_multiplier = old_workspace.margin_multiplier;
        }
    }
}

/// Copy windows state.
fn restore_windows<CMD>(manager: &mut Manager<CMD>, old_manager: &Manager<CMD>) {
    let mut ordered = vec![];
    let mut had_strut = false;

    old_manager.windows.iter().for_each(|old| {
        if let Some((index, window)) = manager
            .windows
            .iter_mut()
            .enumerate()
            .find(|w| w.1.handle == old.handle)
        {
            had_strut = old.strut.is_some() || had_strut;
            if let Some(tag) = old.tags.first() {
                let act = DisplayAction::SetWindowTags(window.handle, tag.clone());
                manager.actions.push_back(act);
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
            manager.windows.remove(index);
        }
    });
    if had_strut {
        manager.update_docks();
    }
    manager.windows.append(&mut ordered);
}

fn restore_scratchpads<CMD>(manager: &mut Manager<CMD>, old_manager: &Manager<CMD>) {
    for (scratchpad, id) in &old_manager.active_scratchpads {
        manager.active_scratchpads.insert(scratchpad.clone(), *id);
    }
}
