//! Save and restore manager state.

use crate::errors::Result;
use crate::Manager;
use std::fs::File;
use std::path::Path;

// TODO: make configurable
/// Path to file where state will be dumper upon soft reload.
const STATE_FILE: &str = "/tmp/leftwm.state";

/// Write current state to a file.
/// It will be used to restore the state after soft reload.
/// # Errors
///
/// Will return error if unable to create `state_file` or
/// if unable to serialize the text.
/// May be caused by inadequate permissions, not enough
/// space on drive, or other typical filesystem issues.
pub fn save(manager: &Manager) -> Result<()> {
    let state_file = File::create(STATE_FILE)?;
    serde_json::to_writer(state_file, &manager)?;
    Ok(())
}

/// Load saved state if it exists.
pub fn load(manager: &mut Manager) {
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

/// Read old state from a state file.
fn load_old_state() -> Result<Manager> {
    let file = File::open(STATE_FILE)?;
    let old_manager = serde_json::from_reader(file)?;
    Ok(old_manager)
}

/// Apply saved state to a running manager.
fn restore_state(manager: &mut Manager, old_manager: &Manager) {
    restore_workspaces(manager, old_manager);
    restore_windows(manager, old_manager);
}

/// Restore workspaces layout.
fn restore_workspaces(manager: &mut Manager, old_manager: &Manager) {
    for workspace in &mut manager.workspaces {
        if let Some(old_workspace) = old_manager.workspaces.iter().find(|w| w.id == workspace.id) {
            workspace.layout = old_workspace.layout.clone();
            workspace.margin_multiplier = old_workspace.margin_multiplier;
        }
    }
}

/// Copy windows state.
fn restore_windows(manager: &mut Manager, old_manager: &Manager) {
    let mut ordered = vec![];

    old_manager.windows.iter().for_each(|old| {
        if let Some((index, window)) = manager
            .windows
            .iter_mut()
            .enumerate()
            .find(|w| w.1.handle == old.handle)
        {
            let mut tags = old.tags.clone();
            if let Some(xyhw) = old.strut {
                let (x, y) = xyhw.center();
                if let Some(ws) = manager.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
                    tags = ws.tags.clone()
                }
            }

            window.set_floating(old.floating());
            window.set_floating_offsets(old.get_floating_offsets());
            window.apply_margin_multiplier(old.margin_multiplier);
            window.normal = old.normal;
            window.tags = tags;
            window.strut = old.strut;
            ordered.push(window.clone());
            manager.windows.remove(index);
        }
    });
    // manager.windows.clear();
    manager.windows.append(&mut ordered);
}
