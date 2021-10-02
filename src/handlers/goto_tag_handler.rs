#![allow(clippy::wildcard_imports)]
use super::*;
use crate::{display_servers::DisplayServer, models::LayoutMode};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    pub fn goto_tag_handler(&mut self, tag_num: usize) -> bool {
        if tag_num > self.state.tags.len() || tag_num < 1 {
            return false;
        }

        let tag_id = self.state.tags[tag_num - 1].id.clone();
        let new_tags = vec![tag_id.clone()];
        //no focus safety check
        let (old_tags, ws_layout) = match self.focused_workspace() {
            Some(ws) => (ws.tags.clone(), ws.layout),
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
            Some(aws) => aws.tags = new_tags.clone(),
            None => return false,
        }
        if self.state.layout_manager.mode == LayoutMode::Workspace {
            if let Some(tag) = self.state.tags.iter_mut().find(|t| t.id == new_tags[0]) {
                tag.layout = ws_layout;
            }
        }
        self.focus_tag(&tag_id);
        self.update_docks();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_to_a_workspace_that_is_already_visible_should_not_duplicate_the_workspace() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.goto_tag_handler(1);
        assert_eq!(manager.state.workspaces[0].tags, ["2".to_owned()]);
        assert_eq!(manager.state.workspaces[1].tags, ["1".to_owned()]);
    }
}
