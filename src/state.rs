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
        old_manager.workspaces
            .iter()
            .find(|w| w.id == workspace.id)
            .map(|old_workspace| {
                workspace.layout = old_workspace.layout.clone();
            });
    }
}

/// Copy windows state.
fn restore_windows(manager: &mut Manager, old_manager: &Manager) {
    for window in &mut manager.windows {
        if let Some(old) = old_manager
            .windows
            .iter()
            .find(|w| w.handle == window.handle)
        {
            window.set_floating(old.floating());
            window.set_floating_offsets(old.get_floating_offsets());
            window.normal = old.normal;
            window.tags = old.tags.clone();
        }
    }
}
