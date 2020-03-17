use crate::models::Manager;

/// Step over all the windows for each workspace and updates all the
/// things based on the new state of the WM.
pub fn update_windows(manager: &mut Manager) {
    // show windows without tags as well as fullscreen ones.
    for window in &mut manager.windows {
        window.set_visible(window.tags.is_empty() || window.is_fullscreen())
    }

    let focused_window = manager.focused_window().cloned();
    for workspace in &manager.workspaces {
        workspace.update_windows(&mut manager.windows, &focused_window);

        // Handle fullscreen windows
        for window in &mut manager.windows {
            if window.is_fullscreen() && workspace.is_displaying(window) {
                window.set_floating(false);
                window.normal = workspace.xyhw;
            }
        }
    }

    for window in &manager.windows {
        if window.debugging {
            log::debug!("{:?}", window);
        }
    }
}
