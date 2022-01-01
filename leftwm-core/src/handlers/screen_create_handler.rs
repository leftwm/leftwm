use super::{Manager, Screen, Workspace};
use crate::config::Config;
use crate::display_servers::DisplayServer;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Process a collection of events, and apply them changes to a manager.
    ///
    /// Returns `true` if changes need to be rendered.
    pub fn screen_create_handler(&mut self, screen: Screen) -> bool {
        let tag_index = self.state.workspaces.len();
        let tag_len = self.state.tags.len_normal();
        let workspace_id = screen.wsid.unwrap_or_else(|| {
            self.state
                .workspaces
                .iter()
                .map(|ws| ws.id.unwrap_or(-1))
                .max()
                .unwrap_or(-1)
                + 1
        });
        let mut new_workspace = Workspace::new(
            Some(workspace_id),
            screen.bbox,
            self.state.layout_manager.new_layout(Some(workspace_id)),
            screen.max_window_width.or(self.state.max_window_width),
        );
        if new_workspace.id.unwrap_or(0) as usize >= tag_len {
            dbg!("Workspace ID needs to be less than or equal to the number of tags available.");
        }
        new_workspace.load_config(&self.config);

        //make sure are enough tags for this new screen
        let next_id = if tag_len > tag_index {
            tag_index + 1
        } else {
            // add a new tag for the workspace
            self.state
                .tags
                .add_new_unlabeled(self.state.layout_manager.new_layout(Some(workspace_id)))
        };

        if let Some(tag) = self.state.tags.get_mut(next_id) {
            tag.layout = new_workspace.layout;
        }

        self.state.focus_workspace(&new_workspace);
        self.state.focus_tag(&next_id);
        new_workspace.show_tag(&next_id);
        self.state.workspaces.push(new_workspace.clone());
        self.state.workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        self.state.screens.push(screen);
        self.state.focus_workspace(&new_workspace);
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
}
