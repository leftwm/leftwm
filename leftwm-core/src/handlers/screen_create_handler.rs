use super::{Manager, Screen, Workspace};
use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::models::Handle;

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    /// Process a collection of events, and apply the changes to a manager.
    ///
    /// Returns `true` if changes need to be rendered.
    pub fn screen_create_handler(&mut self, screen: Screen<H>) -> bool {
        tracing::trace!("Screen create: {:?}", screen);

        let tag_index = self.state.workspaces.len();
        let tag_len = self.state.tags.len_normal();

        // Only used in tests, where there are multiple screens being created by `Screen::default()`
        // The screen passed to this function should normally already have it's id given in the config serialization.
        let workspace_id = match screen.id {
            None => self.state.workspaces.last().map_or(0, |ws| ws.id) + 1,
            Some(set_id) => set_id,
        };

        let mut new_workspace = Workspace::new(screen.bbox, workspace_id);
        if self.state.workspaces.len() >= tag_len {
            tracing::warn!("The number of workspaces needs to be less than or equal to the number of tags available. No more workspaces will be added.");
        }
        new_workspace.load_config(&self.config);

        // Make sure there are enough tags for this new screen.
        let next_id = if tag_len > tag_index {
            tag_index + 1
        } else {
            // Add a new tag for the workspace.
            self.state.tags.add_new_unlabeled()
        };

        self.state.focus_workspace(&new_workspace);
        self.state.focus_tag(&next_id);
        new_workspace.show_tag(&next_id);
        self.state.workspaces.push(new_workspace.clone());
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
