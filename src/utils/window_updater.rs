use crate::models::Manager;

/*
 * step over all the windows for each workspace and updates all the things
 * based on the new state of the WM
 */
pub fn update_windows(manager: &mut Manager) {
    let mut strut_windows = vec![];
    for w in &mut manager.windows {
        w.set_visible(w.tags.is_empty());
        if w.strut.is_some() {
            strut_windows.push(w);
        }
    }
    // Update the current tags for static windows, currently only docks
    // This is to allow us to effectively "manage" them when it is necessary
    for ws in &manager.workspaces {
        strut_windows
            .iter_mut()
            .filter(|w| {
                // It should always unwrap as strut is_some()
                let xyhw = w.strut.unwrap_or_default();
                let (x, y) = xyhw.center();
                ws.contains_point(x, y)
            })
            .for_each(|w| {
                w.tags = ws.tags.clone();
            });
    }
    for ws in &mut manager.workspaces {
        ws.update_windows(&mut manager.windows);

        manager
            .windows
            .iter_mut()
            .filter(|w| ws.is_displaying(w) && w.is_fullscreen())
            .for_each(|w| {
                w.set_floating(false);
                w.normal = ws.xyhw;
            });
    }
    manager
        .windows
        .iter()
        .filter(|x| x.debugging)
        .for_each(|w| {
            println!("{:?}", w);
        });
}
