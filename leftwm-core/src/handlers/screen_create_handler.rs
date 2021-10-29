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
        if self.tags.len() <= tag_index {
            let id = (tag_index + 1).to_string();
            self.tags.add_new(id.as_str(), self.layout_manager.new_layout());
        }
        let next_tag = self.tags[tag_index].clone();
        self.focus_workspace(&workspace);
        self.focus_tag(&next_tag.label);
        workspace.show_tag(&next_tag);
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
        assert!(state.workspaces[0].has_tag("1"));
        assert!(state.workspaces[1].has_tag("2"));
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
        assert!(state.workspaces[0].has_tag("web"));
        assert!(state.workspaces[1].has_tag("console"));
    }
}
