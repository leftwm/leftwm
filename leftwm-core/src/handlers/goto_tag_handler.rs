#![allow(clippy::wildcard_imports)]
use super::*;
use crate::state::State;

impl<C: Config> State<C> {
    pub fn goto_tag_handler(&mut self, tag_num: usize) -> Option<bool> {
        if tag_num > self.tags.len() || tag_num < 1 {
            return Some(false);
        }

        let tag_id = self.tags[tag_num - 1].id.clone();
        let new_tags = vec![tag_id.clone()];
        //no focus safety check
        let old_tags = self.focus_manager.workspace(&self.workspaces)?.tags.clone();
        if let Some(handle) = self.focus_manager.window(&self.windows).map(|w| w.handle) {
            let old_handle = self
                .focus_manager
                .tags_last_window
                .entry(old_tags[0].clone())
                .or_insert(handle);
            *old_handle = handle;
        }
        if let Some(ws) = self.workspaces.iter_mut().find(|ws| ws.tags == new_tags) {
            ws.tags = old_tags;
        }

        self.focus_manager.workspace_mut(&mut self.workspaces)?.tags = new_tags;
        self.focus_tag(&tag_id);
        self.update_static();
        self.layout_manager
            .update_layouts(&mut self.workspaces, &mut self.tags);
        Some(true)
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
