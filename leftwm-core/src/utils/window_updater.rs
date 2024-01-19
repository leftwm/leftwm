use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::{Manager, Handle};

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    /*
     * step over all the windows for each workspace and updates all the things
     * based on the new state of the WM
     */
    pub fn update_windows(&mut self) {
        // set all tagged windows as visible
        self.state
            .windows
            .iter_mut()
            .for_each(|w| w.set_visible(w.tag.is_none()));

        for ws in &self.state.workspaces {
            let windows = &mut self.state.windows;
            let all_tags = &self.state.tags;
            if let Some(Some(tag)) = ws.tag.map(|tag_id| all_tags.get(tag_id)) {
                tag.update_windows(windows, ws, &mut self.state.layout_manager);
            }
        }
    }
}
