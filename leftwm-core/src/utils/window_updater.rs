use crate::config::{Config, FocusBehaviour};
use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
use crate::models::{Manager, WindowHandle};

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

        let mut main_monocle_to_focus: Option<WindowHandle> = None;

        for ws in &self.state.workspaces {
            let tag = self
                .state
                .tags
                .iter_mut()
                .find(|t| ws.has_tag(&t.id))
                .expect("Workspace has no tag.");
            tag.update_windows(&mut self.state.windows, ws);

            self.state
                .windows
                .iter_mut()
                .filter(|w| ws.is_displaying(w) && w.is_fullscreen())
                .for_each(|w| w.normal = ws.xyhw);

            // When switching to Monocle layout while in Driven and ClickTo
            // focus mode, we give focus to the main window, which will be
            // the window which will apear when switching.
            let focused_window = self.state.focus_manager.window_history.get(0);
            let is_focused_floating = match self
                .state
                .windows
                .iter()
                .find(|w| Some(&Some(w.handle)) == focused_window)
            {
                Some(w) => w.floating(),
                None => false,
            };
            if ws.layout == Layout::Monocle
                && self.state.focus_manager.behaviour != FocusBehaviour::Sloppy
                && !is_focused_floating
            {
                let window = self
                    .state
                    .windows
                    .iter()
                    .find(|w| w.has_tag(tag.id.as_str()) && !w.is_unmanaged() && !w.floating());
                if let Some(w) = window {
                    main_monocle_to_focus = Some(w.handle);
                }
            }
        }
        if let Some(h) = main_monocle_to_focus {
            self.state.focus_window(&h);
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
