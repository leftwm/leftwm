#![allow(clippy::wildcard_imports)]
use super::*;
use crate::display_servers::DisplayServer;

impl<C: Config<CMD>, SERVER: DisplayServer<CMD>, CMD> Manager<C, CMD, SERVER> {
    pub fn goto_tag_handler(&mut self, tag_num: usize) -> bool {
        if tag_num > self.state.tags.len() || tag_num < 1 {
            return false;
        }

        let tag = self.state.tags[tag_num - 1].clone();
        let new_tags = vec![tag.id.clone()];
        //no focus safety check
        let old_tags = match self.focused_workspace() {
            Some(ws) => ws.tags.clone(),
            None => return false,
        };
        let handle = self.focused_window().map(|w| w.handle);
        if let Some(handle) = handle {
            let old_handle = self
                .state
                .focus_manager
                .tags_last_window
                .entry(old_tags[0].clone())
                .or_insert(handle);
            *old_handle = handle;
        }
        if let Some(ws) = self
            .state
            .workspaces
            .iter_mut()
            .find(|ws| ws.tags == new_tags)
        {
            ws.tags = old_tags;
        }

        match self.focused_workspace_mut() {
            Some(aws) => aws.tags = new_tags,
            None => return false,
        }
        self.focus_tag(&tag.id);
        self.update_docks();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_to_a_workspace_that_is_already_visible_should_not_duplicate_the_workspace() {
        let mut manager = two_screen_mock_manager();
        manager.goto_tag_handler(1);
        assert_eq!(manager.workspaces[0].tags, ["2".to_owned()]);
        assert_eq!(manager.workspaces[1].tags, ["1".to_owned()]);
    }

    fn two_screen_mock_manager() -> Manager<()> {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager
    }
}
