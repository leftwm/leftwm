use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::{Manager, Tag};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /*
     * step over all the windows for each workspace and updates all the things
     * based on the new state of the WM
     */
    pub fn update_windows(&mut self) {
        
        // set all tagged windows as visible
        self.state
            .windows
            .iter_mut()
            .for_each(|w| w.set_visible(w.tags.is_empty()));

        for ws in &self.state.workspaces {
            let windows = &mut self.state.windows;
            let all_tags = &self.state.tags;
            let tags: Vec<&Tag> = ws.tags.iter()
                .flat_map(|tag_id| all_tags.get(tag_id.clone()))
                .collect();
            for tag in &tags {
                tag.update_windows(windows, ws)
            }

            // resize all windows marked as fullscreen to the workspace size
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
