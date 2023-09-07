use crate::{models::TagId, state::State};

impl State {
    pub fn goto_tag_handler(&mut self, tag_id: TagId) -> Option<bool> {
        if tag_id > self.tags.len_normal() || tag_id < 1 {
            return Some(false);
        }

        let new_tag = Some(tag_id);
        // No focus safety check.
        let old_tag = self.focus_manager.workspace(&self.workspaces)?.tag;
        if let Some(old_tag) = old_tag {
            if let Some(handle) = self.focus_manager.window(&self.windows).map(|w| w.handle) {
                let old_handle = self
                    .focus_manager
                    .tags_last_window
                    .entry(old_tag)
                    .or_insert(handle);
                *old_handle = handle;
            }

            let old_tag_ws = self
                .workspaces
                .iter()
                .any(|ws| ws.tag == Some(old_tag) && ws.pinned_tags.contains(&old_tag));

            if let Some(ws) = self
                .workspaces
                .iter_mut()
                .find(|ws| ws.tag == new_tag && !ws.pinned_tags.contains(&old_tag))
            {
                if old_tag_ws {
                    ws.tag = None;
                } else {
                    ws.tag = Some(old_tag);
                }
            }
        }

        if let Some(workspace) = self
            .workspaces
            .clone()
            .into_iter()
            .find(|w| w.pinned_tags.contains(&tag_id))
        {
            tracing::debug!("tag is pinned to: {workspace:#?}");
            self.focus_workspace(&workspace);
            // self.focus_tag(&tag_id);
        }

        self.focus_manager.workspace_mut(&mut self.workspaces)?.tag = new_tag;

        self.focus_tag(&tag_id);
        self.update_static();

        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Screen;
    use crate::Manager;

    #[test]
    fn going_to_a_workspace_that_is_already_visible_should_not_duplicate_the_workspace() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default(), None);
        manager.screen_create_handler(Screen::default(), None);
        manager.state.goto_tag_handler(1);
        assert_eq!(manager.state.workspaces[0].tag, Some(2));
        assert_eq!(manager.state.workspaces[1].tag, Some(1));
    }
}
