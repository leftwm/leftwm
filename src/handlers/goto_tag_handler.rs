#![allow(clippy::wildcard_imports)]
use super::*;

pub fn process(manager: &mut Manager, tag_num: usize) -> bool {
    if tag_num > manager.tags.len() || tag_num < 1 {
        return false;
    }

    let tag = manager.tags[tag_num - 1].clone();
    let new_tags = vec![tag.id.clone()];
    //no focus safety check
    if manager.focused_workspace().is_none() {
        return false;
    }
    let old_tags = manager
        .focused_workspace()
        .map(|ws| ws.tags.clone())
        .unwrap_or_default();
    for wp in &mut manager.workspaces {
        if wp.tags == new_tags {
            wp.tags = old_tags.clone();
        }
    }
    let active_workspace = match manager.focused_workspace_mut() {
        Some(x) => x,
        None => return false,
    };

    active_workspace.tags = new_tags;
    focus_handler::focus_tag(manager, &tag.id);

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_to_a_workspace_that_is_already_visible_should_not_duplicate_the_workspace() {
        let mut manager = two_screen_mock_manager();
        process(&mut manager, 1);
        assert_eq!(manager.workspaces[0].tags, ["2".to_owned()]);
        assert_eq!(manager.workspaces[1].tags, ["1".to_owned()]);
    }

    fn two_screen_mock_manager() -> Manager {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        manager
    }
}
