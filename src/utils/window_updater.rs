use crate::models::Manager;
use crate::models::Window;

/*
 * step over all the windows for each workspace and updates all the things
 * based on the new state of the WM
 */
pub fn update_windows(manager: &mut Manager) {
    manager
        .windows
        .iter_mut()
        .for_each(|w| w.set_visible(w.tags.is_empty() || w.floating() || w.is_fullscreen()));
    let all_windows = &mut manager.windows;
    manager.workspaces.iter_mut().for_each(|ws| {
        let mut windows: Vec<&mut Window> = all_windows.iter_mut().collect();
        ws.update_windows(&mut windows);

        windows
            .iter_mut()
            .filter(|w| ws.is_displaying(w) && w.is_fullscreen())
            .for_each(|w| w.floating = Some(ws.xyhw));
    });
}
