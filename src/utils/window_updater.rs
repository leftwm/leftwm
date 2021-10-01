use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::Manager;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /*
     * step over all the windows for each workspace and updates all the things
     * based on the new state of the WM
     */
    pub fn update_windows(&mut self) {
        self.state
            .windows
            .iter_mut()
            .for_each(|w| w.set_visible(w.tags.is_empty()));

        for ws in &self.state.workspaces {
            let tag = ws
                .current_tags(&mut self.state.tags)
                .get(0)
                .expect("Workspace has no tags.");
            tag.update_windows(&mut self.state.windows, ws);

            self.state
                .windows
                .iter_mut()
                .filter(|w| ws.is_displaying(w) && w.is_fullscreen())
                .for_each(|w| w.normal = ws.xyhw);
        }
        self.state
            .windows
            .iter()
            .filter(|x| x.debugging)
            .for_each(|w| {
                println!("{:?}", w);
            });
    }
}
