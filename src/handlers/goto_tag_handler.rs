use super::*;

pub fn process(manager: &mut Manager, tag: String) -> bool {
    if !manager.tags.contains(&tag) {
        manager.tags.push(tag.clone());
    }
    if let Some(workspace) = manager.focused_workspace() {
        workspace.show_tag(tag.clone());
        focus_handler::focus_tag(manager, &tag);
        return true;
    }
    true
}

fn two_screen_mock_manager() -> Manager {
    let mut manager = Manager::default();
    screen_create_handler::process(&mut manager, Screen::default());
    screen_create_handler::process(&mut manager, Screen::default());
    manager
}

#[test]
fn going_to_a_workspace_that_is_already_visable_should_not_duplicate_the_workspace() {
    let mut manager = two_screen_mock_manager();
    process(&mut manager, "1".to_owned());
    assert_eq!(manager.workspaces[0].tags, ["2".to_owned()]);
    assert_eq!(manager.workspaces[1].tags, ["1".to_owned()]);
}
