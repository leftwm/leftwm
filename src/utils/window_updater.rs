use crate::models::Manager;

impl<CMD> Manager<CMD> {
    /*
     * step over all the windows for each workspace and updates all the things
     * based on the new state of the WM
     */
    pub fn update_windows(&mut self) {
        self.windows
            .iter_mut()
            .for_each(|w| w.set_visible(w.tags.is_empty()));

        for ws in &mut self.workspaces {
            ws.update_windows(&mut self.windows, &mut self.tags);

            self.windows
                .iter_mut()
                .filter(|w| ws.is_displaying(w) && w.is_fullscreen())
                .for_each(|w| w.normal = ws.xyhw);
        }
        self.windows.iter().filter(|x| x.debugging).for_each(|w| {
            println!("{:?}", w);
        });
    }
}
