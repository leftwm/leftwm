//! Save and restore manager state.

use super::{Command, Config};
use leftwm::errors::Result;
use leftwm::{DisplayAction, Manager};
use std::fs::File;
use std::path::Path;

/// Read old state from a state file.
pub fn load_old_state(path: impl AsRef<Path>) -> Result<Manager<Config, Command>> {
    let file = File::open(path)?;
    let old_manager = serde_json::from_reader(file)?;
    Ok(old_manager)
}

/// Apply saved state to a running manager.
pub fn restore_state(
    manager: &mut Manager<Config, Command>,
    old_manager: &Manager<Config, Command>,
) {
    // restore workspaces
    for workspace in &mut manager.workspaces {
        if let Some(old_workspace) = old_manager.workspaces.iter().find(|w| w.id == workspace.id) {
            workspace.layout = old_workspace.layout.clone();
            workspace.margin_multiplier = old_workspace.margin_multiplier;
        }
    }

    // restore windows
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

    // restore scratchpads
    for (scratchpad, id) in &old_manager.active_scratchpads {
        manager.active_scratchpads.insert(scratchpad.clone(), *id);
    }
}
