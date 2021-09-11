//! Save and restore manager state.

use leftwm::errors::Result;
use leftwm::Manager;
use std::fs::File;
use std::path::Path;

// TODO: make configurable
/// Path to file where state will be dumped upon soft reload.
const STATE_FILE: &str = "/tmp/leftwm.state";

pub struct State;

impl leftwm::State for State {
    fn save(&self, manager: &Manager) -> Result<()> {
        let state_file = File::create(STATE_FILE)?;
        serde_json::to_writer(state_file, &manager)?;
        Ok(())
    }

    fn load(&self, manager: &mut Manager) {
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
fn load_old_state() -> Result<Manager> {
    let file = File::open(STATE_FILE)?;
    let old_manager = serde_json::from_reader(file)?;
    Ok(old_manager)
}

/// Apply saved state to a running manager.
fn restore_state(manager: &mut Manager, old_manager: &Manager) {
    restore_workspaces(manager, old_manager);
    restore_windows(manager, old_manager);
    restore_scratchpads(manager, old_manager);
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
    let mut had_strut = false;

    old_manager.windows.iter().for_each(|old| {
        if let Some((index, window)) = manager
            .windows
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
            if manager.tags.eq(&old_manager.tags) {
                window.tags = old.tags.clone();
            } else {
                log::info!("Tag config changed, mapping tags based on index.");
                for t in &old.tags {
                log::info!("Window before: {:?}", &window.tags);
                    let manager_tags = &manager.tags.clone();
                    let old_tags = old_manager.tags.clone();
                    let tag_index = &old_tags.iter().position(|o| &o.id == t);
                    window.clear_tags();
                    // if the config prior reload had more tags then the new one
                    // we want to move windows of lost tags to the 'first' tag
                    if tag_index.unwrap() <= manager_tags.len() {
                        log::info!("Index: {:?} old tag: {:?}", &tag_index.unwrap(), t);
                        let designated_id = &manager_tags[tag_index.clone().unwrap()].id;
                        log::info!("Assigning tag ID: {:?}", &designated_id);
                        window.tag(&designated_id);
                    } else {
                        log::info!("Default ID: {:?}", &manager_tags[0].id);
                        window.tag(&manager_tags.first().unwrap().id);
                    }
                log::info!("Window after: {:?}", &window.tags);
                }
            }
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

fn restore_scratchpads(manager: &mut Manager, old_manager: &Manager) {
    for (scratchpad, id) in &old_manager.active_scratchpads {
        manager.active_scratchpads.insert(scratchpad.clone(), *id);
    }
}
