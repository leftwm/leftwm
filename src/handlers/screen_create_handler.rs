use super::{focus_handler, Manager, Screen, Workspace};
use crate::models::Tag;

/// Process a collection of events, and apply them changes to a manager.
/// Returns `true` if changes need to be rendered.
pub fn process(manager: &mut Manager, screen: Screen) -> bool {
    let tag_index = manager.workspaces.len();

    let mut workspace = Workspace::new(
        screen.wsid,
        screen.bbox,
        manager.tags.clone(),
        manager.layouts.clone(),
        screen.max_window_width.or(manager.max_window_width),
    );
    if workspace.id.is_none() {
        workspace.id = Some(
            manager
                .workspaces
                .iter()
                .map(|ws| ws.id.unwrap_or(-1))
                .max()
                .unwrap_or(-1)
                + 1,
        );
    }
    if workspace.id.unwrap_or(0) as usize >= manager.tags.len() {
        dbg!("Workspace ID needs to be less than or equal to the number of tags available.");
    }
    workspace.update_for_theme(&manager.theme_setting);
    //make sure are enough tags for this new screen
    if manager.tags.len() <= tag_index {
        let id = (tag_index + 1).to_string();
        manager.tags.push(Tag::new(&id));
    }
    let next_tag = manager.tags[tag_index].clone();
    focus_handler::focus_workspace(manager, &workspace);
    focus_handler::focus_tag(manager, &next_tag.id);
    workspace.show_tag(&mut manager.tags, &next_tag);
    manager.workspaces.push(workspace.clone());
    manager
        .workspaces
        .sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
    manager.screens.push(screen);
    focus_handler::focus_workspace(manager, &workspace);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_two_screens_should_tag_them_with_first_and_second_tags() {
        let mut manager = Manager::new_test();
        process(&mut manager, Screen::default());
        process(&mut manager, Screen::default());
        assert!(manager.workspaces[0].has_tag("1"));
        assert!(manager.workspaces[1].has_tag("2"));
    }

    #[test]
    fn should_be_able_to_add_screens_with_preexisting_tags() {
        let mut manager = Manager::new_test();
        manager.tags.push(Tag::new("web"));
        manager.tags.push(Tag::new("console"));
        manager.tags.push(Tag::new("code"));
        process(&mut manager, Screen::default());
        process(&mut manager, Screen::default());
        assert!(manager.workspaces[0].has_tag("web"));
        assert!(manager.workspaces[1].has_tag("console"));
    }
}
