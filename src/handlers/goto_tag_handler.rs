#![allow(clippy::wildcard_imports)]
use super::*;

pub fn process(manager: &mut Manager, tag_num: usize) -> bool {
    if tag_num > manager.tags.len() || tag_num < 1 {
        return false;
    }

    let tag = manager.tags[tag_num - 1].clone();
    let new_tags = vec![tag.id.clone()];
    //no focus safety check
    let old_tags = match manager.focused_workspace() {
        Some(ws) => ws.tags.clone(),
        None => return false,
    };
    if let Some(ws) = manager.workspaces.iter_mut().find(|ws| ws.tags == new_tags) {
        ws.tags = old_tags;
    }

    match manager.focused_workspace_mut() {
        Some(aws) => aws.tags = new_tags,
        None => return false,
    }
    focus_handler::focus_tag(manager, &tag.id);
    manager.update_docks();
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
