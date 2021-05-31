#![allow(clippy::wildcard_imports)]
use crate::utils::helpers;

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
    let mut changed = vec![];
    if let Some(ws) = manager.workspaces.iter_mut().find(|ws| ws.tags == new_tags) {
        update_dock_tags(&mut manager.windows, ws, &old_tags, &mut changed);
        ws.tags = old_tags;
    }

    match manager.focus_manager.workspace_mut(&mut manager.workspaces) {
        Some(ws) => {
            update_dock_tags(&mut manager.windows, ws, &new_tags, &mut changed);
            ws.tags = new_tags
        }
        None => return false,
    }
    focus_handler::focus_tag(manager, &tag.id);
    true
}

fn update_dock_tags(
    windows: &mut Vec<Window>,
    ws: &Workspace,
    tags: &[String],
    changed: &mut Vec<WindowHandle>,
) {
    let clone = changed.clone();
    //Update the tags of the docks
    windows
        .iter_mut()
        .filter(|w| {
            helpers::intersect(&ws.tags, &w.tags) && w.strut.is_some() && !clone.contains(&w.handle)
        })
        .for_each(|w| {
            w.tags = tags.to_vec();
            changed.push(w.handle)
        });
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
