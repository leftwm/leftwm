use super::{Manager, Screen, Workspace};
use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::BBox;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// `screen_create_handler` is called when the display server sends a
    /// `DisplayEvent::ScreenCreate(screen)` event. This happens at initialization.
    ///
    /// Returns `true` if changes need to be rendered.
    pub fn screen_create_handler(&mut self, screen: Screen) -> bool {
        tracing::info!("Screen create handler on {}", screen.output);
        let workspaces = self
            .config
            .workspaces()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .filter(|(_, wsc)| wsc.output == screen.output)
            .collect::<Vec<_>>();

        if workspaces.is_empty() && self.config.auto_derive_workspaces() {
            // If there is no workspace for this screen in the config and `auto_derive_workspaces` is set, create a workspace based on the screen.
            self.create_workspace(screen);
        } else {
            for (i, wsc) in workspaces {
                let mut new_screen = Screen::from(&wsc);
                if wsc.relative == Some(true) {
                    new_screen.bbox.add(screen.bbox);
                }
                new_screen.root = screen.root;
                new_screen.id = Some(i + 1);
                self.create_workspace(new_screen);
            }
        }

        false
    }

    fn create_workspace(&mut self, mut screen: Screen) {
        tracing::warn!("Creating Workspace on screen: {:?}", screen);

        let tag_index = self.state.workspaces.len();
        let tag_len = self.state.tags.len_normal();

        let screen_id = screen.id.unwrap_or_else(|| {
            // Used in tests, where there are multiple screens being created by `Screen::default()`
            // and for workspaces created by `auto_derive_worspaces`
            // Selects the next auto-generated id, being minimally one higher than those already defined in the config.
            let min_id_current = self.state.workspaces.iter().fold(0, |i, wsc| i.max(wsc.id));
            let min_id_config = self.config.workspaces().map_or(0, |w| w.len());
            min_id_current.max(min_id_config) + 1
        });
        screen.id = Some(screen_id);
        // TODO Currently unused
        //screen.max_window_width = screen.max_window_width.or(self.config.max_window_width);

        let mut new_workspace = Workspace::new(screen);

        if self.state.workspaces.len() >= tag_len {
            tracing::warn!("The number of workspaces needs to be less than or equal to the number of tags available. Unlabled tags will be automatically created.");
        }

        // Make sure there are enough tags for this new screen.
        let next_id = if tag_len > tag_index {
            tag_index + 1
        } else {
            // Add a new tag for the workspace.
            self.state.tags.add_new(screen_id.to_string().as_str())
        };

        self.state.focus_workspace(&new_workspace); // focus_workspace is called.
        self.state.focus_tag(&next_id);
        new_workspace.show_tag(&next_id);
        self.state.workspaces.push(new_workspace.clone());
        self.state.focus_workspace(&new_workspace); // focus_workspace is called again.
    }

    pub fn screen_update_handler(&mut self, screen: Screen) -> bool {
        tracing::debug!("Screen update handler on {}", screen.output);
        // Also recieves new screens, needs to ckeck that -> update or create.
        let affected = self
            .state
            .workspaces
            .iter_mut()
            .filter(|ws| ws.output == screen.output)
            .collect::<Vec<_>>();
        if affected.is_empty() {
            self.screen_create_handler(screen);
        } else {
            tracing::info!("Screen update for {}", screen.output);
            for wsc in affected {
                // Apply config changes (if existing)
                let bbox = if let Some(config_ws) = self
                    .config
                    .workspaces()
                    .and_then(|wss| wss.get(wsc.id).cloned())
                {
                    let mut bbox = BBox {
                        x: config_ws.x,
                        y: config_ws.y,
                        width: config_ws.width,
                        height: config_ws.height,
                    };
                    if config_ws.relative == Some(true) {
                        bbox.add(screen.bbox);
                    }
                    bbox
                } else {
                    screen.bbox
                };

                wsc.update_bbox(bbox);
            }
        }

        // Call up scripts (reload theme)
        // TODO add reload hook
        self.call_up_scripts();

        false
    }

    pub fn screen_delete_handler(&mut self, output: &str) -> bool {
        tracing::info!("Screen delete handler on {}", output);
        self.state.workspaces.retain(|wsc| wsc.output != output);

        tracing::warn!("Screen delete handler on {}", output);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manager;

    #[test]
    fn creating_two_screens_should_tag_them_with_first_and_second_tags() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        assert!(manager.state.workspaces[0].has_tag(&1));
        assert!(manager.state.workspaces[1].has_tag(&2));
    }

    #[test]
    fn should_be_able_to_add_screens_with_preexisting_tags() {
        let mut manager = Manager::new_test(vec![
            "web".to_string(),
            "console".to_string(),
            "code".to_string(),
        ]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        assert!(manager.state.workspaces[0].has_tag(&1));
        assert!(manager.state.workspaces[1].has_tag(&2));
    }

    #[test]
    fn creating_more_screens_than_tags_should_automatically_create_new_tags() {
        let mut manager = Manager::new_test(vec!["web".to_string(), "console".to_string()]);

        // there should be 2 tags in the beginning
        assert_eq!(manager.state.tags.len_normal(), 2);

        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());

        // there should be 4 workspaces
        assert_eq!(manager.state.workspaces.len(), 4);

        // there should be 4 tags after creating 4 screens/workspaces
        assert_eq!(manager.state.tags.len_normal(), 4);

        // workspaces should have tags in order
        assert!(manager.state.workspaces[0].has_tag(&1));
        assert!(manager.state.workspaces[1].has_tag(&2));
        assert!(manager.state.workspaces[2].has_tag(&3));
        assert!(manager.state.workspaces[3].has_tag(&4));
    }

    #[test]
    fn creating_workspaces_should_set_ids_correctly() {
        let mut manager = Manager::new_test_with_outputs(vec![
            "NOT-USED-1".to_string(),
            "USED-1".to_string(),
            "NOT-USED-2".to_string(),
        ]);

        // there should be no workspaces in the begining
        assert_eq!(manager.state.workspaces.len(), 0);

        manager.screen_create_handler(Screen {
            output: "UNDEFINED-1".to_string(),
            ..Default::default()
        });
        manager.screen_create_handler(Screen {
            output: "USED-1".to_string(),
            ..Default::default()
        });
        manager.screen_create_handler(Screen {
            output: "UNDEFINED-2".to_string(),
            ..Default::default()
        });

        // there should be 2 workspaces
        assert_eq!(manager.state.workspaces.len(), 3);

        // undefined workspace was added after last config ws
        assert_eq!(manager.state.workspaces[0].id, 4);

        // used workspace was the second in config
        assert_eq!(manager.state.workspaces[1].id, 2);

        // second undefined workspace increments count
        assert_eq!(manager.state.workspaces[2].id, 5);
    }
}
