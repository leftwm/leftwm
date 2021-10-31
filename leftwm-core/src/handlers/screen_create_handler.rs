use super::{Screen, Workspace};
use crate::config::Config;
use crate::models::Tag;
use crate::state::State;

impl<C: Config> State<C> {
    /// Process a collection of events, and apply them changes to a manager.
    ///
    /// Returns `true` if changes need to be rendered.
    pub fn screen_create_handler(&mut self, screen: Screen) -> bool {
        let tag_index = self.workspaces.len();

        let mut workspace = Workspace::new(
            screen.wsid,
            screen.bbox,
            self.layout_manager.new_layout(),
            screen
                .max_window_width
                .or_else(|| self.config.max_window_width()),
        );
        if workspace.id.is_none() {
            workspace.id = Some(
                self.workspaces
                    .iter()
                    .map(|ws| ws.id.unwrap_or(-1))
                    .max()
                    .unwrap_or(-1)
                    + 1,
            );
        }
        if workspace.id.unwrap_or(0) as usize >= self.tags.len() {
            dbg!("Workspace ID needs to be less than or equal to the number of tags available.");
        }
        workspace.update_for_theme(&self.config);

        //make sure are enough tags for this new screen
        let next_id = tag_index + 1;
        if self.tags.len() < next_id {
            let id = next_id.to_string();
            self.tags.add_new(id.as_str(), self.layout_manager.new_layout());
        }
        let next_tag = self.tags.get(next_id).unwrap().clone();
        self.focus_workspace(&workspace);
        self.focus_tag(&next_tag.id);
        workspace.show_tag(&next_tag.id);
        
        self.workspaces.push(workspace.clone());
        self.workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        self.screens.push(screen);
        self.focus_workspace(&workspace);
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
        let state = &mut manager.state;
        state.screen_create_handler(Screen::default());
        state.screen_create_handler(Screen::default());
        assert!(state.workspaces[0].has_tag(&1));
        assert!(state.workspaces[1].has_tag(&2));
    }

    #[test]
    fn should_be_able_to_add_screens_with_preexisting_tags() {
        let mut manager = Manager::new_test(vec![
            "web".to_string(),
            "console".to_string(),
            "code".to_string(),
        ]);
        let state = &mut manager.state;
        state.screen_create_handler(Screen::default());
        state.screen_create_handler(Screen::default());
        assert!(state.workspaces[0].has_tag(&1));
        assert!(state.workspaces[1].has_tag(&2));
    }

    #[test]
    fn creating_more_screens_than_tags_should_automatically_create_new_tags() {
        let mut manager = Manager::new_test(vec![
            "web".to_string(),
            "console".to_string(),
        ]);
        let state = &mut manager.state;

        // there should be 2 tags in the beginning
        assert_eq!(state.tags.len(), 2);

        state.screen_create_handler(Screen::default());
        state.screen_create_handler(Screen::default());
        state.screen_create_handler(Screen::default());
        state.screen_create_handler(Screen::default());

        // there should be 4 workspaces
        assert_eq!(state.workspaces.len(), 4);

        // there should be 4 tags after creating 4 screens/workspaces
        assert_eq!(state.tags.len(), 4);

        // workspaces should have tags in order
        assert!(state.workspaces[0].has_tag(&1));
        assert!(state.workspaces[1].has_tag(&2));
        assert!(state.workspaces[2].has_tag(&3));
        assert!(state.workspaces[3].has_tag(&4));
    }
}
