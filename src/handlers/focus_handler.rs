#![allow(clippy::wildcard_imports)]
use super::*;
use crate::{display_action::DisplayAction, utils::helpers};

/// Marks a workspace as the focused workspace.
//NOTE: should only be called externally from this file
pub fn focus_workspace(manager: &mut Manager, workspace: &Workspace) -> bool {
    if focus_workspace_work(manager, workspace.id).is_some() {
        //make sure this workspaces tag is focused
        workspace.tags.iter().for_each(|t| {
            focus_tag_work(manager, t);
        });
        // create an action to inform the DM
        update_current_tags(manager);
        return true;
    }
    false
}

fn focus_workspace_work(manager: &mut Manager, workspace_id: i32) -> Option<()> {
    //no new history if no change
    if let Some(fws) = manager.focused_workspace() {
        if fws.id == workspace_id {
            return None;
        }
    }
    //clean old ones
    manager.focus_manager.workspace_history.truncate(10);
    //add this focus to the history
    let index = manager
        .workspaces
        .iter()
        .position(|x| x.id == workspace_id)?;
    manager.focus_manager.workspace_history.push_front(index);
    Some(())
}

/// Create a `DisplayAction` to cause this window to become focused
pub fn focus_window(manager: &mut Manager, handle: &WindowHandle) -> bool {
    let window = match focus_window_by_handle_work(manager, handle) {
        Some(w) => w,
        None => return false,
    };

    //make sure the focused window's workspace is focused
    let (tags, workspace_id) = match manager
        .workspaces
        .iter()
        .find(|ws| ws.is_displaying(&window))
    {
        Some(ws) => (ws.tags.clone(), Some(ws.id)),
        None => (vec![], None),
    };
    if let Some(workspace_id) = workspace_id {
        let _ = focus_workspace_work(manager, workspace_id);
    }

    //make sure the focused window's tag is focused
    if let Some(tag) = tags.iter().find(|t| window.has_tag(&t)) {
        let _ = focus_tag_work(manager, tag);
    }
    true
}

fn focus_window_by_handle_work(manager: &mut Manager, handle: &WindowHandle) -> Option<Window> {
    //Docks don't want to get focus. If they do weird things happen. They don't get events...
    //Do the focus, Add the action to the list of action
    let found: &Window = manager.windows.iter().find(|w| &w.handle == handle)?;
    if found.type_ == WindowType::Dock {
        return None;
    }
    //NOTE: we are intentionally creating the focus event even if we think this window
    //is already in focus. This is to force the DM to update its knowledge of the focused window
    let act = DisplayAction::WindowTakeFocus(found.clone());
    manager.actions.push_back(act);

    //no new history if no change
    if let Some(fw) = manager.focused_window() {
        if &fw.handle == handle {
            //NOTE: we still made the action so return some
            return Some(found.clone());
        }
    }
    //clean old ones
    manager.focus_manager.window_history.truncate(10);
    //add this focus to the history
    manager.focus_manager.window_history.push_front(*handle);

    Some(found.clone())
}

pub fn move_cursor_over(manager: &mut Manager, window: &Window) {
    let act = DisplayAction::MoveMouseOver(window.handle);
    manager.actions.push_back(act);
}

pub fn validate_focus_at(manager: &mut Manager, x: i32, y: i32) -> bool {
    let current = match manager.focused_window() {
        Some(w) => w,
        None => return false,
    };
    //only look at windows we can focus
    let found: Option<Window> = manager
        .windows
        .iter()
        .filter(|x| x.can_focus())
        .find(|w| w.contains_point(x, y))
        .cloned();
    match found {
        Some(window) => {
            //only do the focus if we need to
            let handle = window.handle;
            if current.handle == handle {
                return false;
            }
            focus_window(manager, &handle)
        }
        None => false,
    }
}

pub fn move_focus_to_point(manager: &mut Manager, x: i32, y: i32) -> bool {
    let found: Option<Window> = manager
        .windows
        .iter()
        .filter(|x| x.can_focus())
        .find(|w| w.contains_point(x, y))
        .cloned();
    match found {
        Some(found) => focus_window(manager, &found.handle),
        //backup plan, move focus closest window in workspace
        None => focus_closest_window(manager, x, y),
    }
}

fn focus_closest_window(manager: &mut Manager, x: i32, y: i32) -> bool {
    let ws = match manager.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
        Some(ws) => ws,
        None => return false,
    };
    let mut dists: Vec<(i32, &Window)> = manager
        .windows
        .iter()
        .filter(|x| ws.is_managed(&x) && x.can_focus())
        .map(|w| (distance(w, x, y), w))
        .collect();
    dists.sort_by(|a, b| (a.0).cmp(&b.0));
    if let Some(first) = dists.get(0) {
        let handle = first.1.handle;
        return focus_window(manager, &handle);
    }
    false
}

fn distance(window: &Window, x: i32, y: i32) -> i32 {
    // √((x_2-x_1)²+(y_2-y_1)²)
    let (wx, wy) = window.calculated_xyhw().center();
    let xs = f64::from((wx - x) * (wx - x));
    let ys = f64::from((wy - y) * (wy - y));
    (xs + ys).sqrt().abs().floor() as i32
}

pub fn focus_workspace_under_cursor(manager: &mut Manager, x: i32, y: i32) -> bool {
    let focused_id = match manager.focused_workspace() {
        Some(fws) => fws.id,
        None => -1,
    };
    if let Some(w) = manager
        .workspaces
        .iter()
        .find(|ws| ws.contains_point(x, y) && ws.id != focused_id)
        .cloned()
    {
        return focus_workspace(manager, &w);
    }
    false
}

/// marks a tag as the focused tag
//NOTE: should only be called externally from this file
pub fn focus_tag(manager: &mut Manager, tag: &str, config: &Config) -> bool {
    if focus_tag_work(manager, tag).is_none() {
        return false;
    }
    // check each workspace, if its displaying this tag it should be focused too
    let to_focus: Vec<Workspace> = manager
        .workspaces
        .iter()
        .filter(|w| w.has_tag(tag))
        .cloned()
        .collect();
    for ws in &to_focus {
        focus_workspace_work(manager, ws.id);
    }
    //make sure the focused window is on this workspace
    if config.focus_tracks_mouse {
        let act = DisplayAction::FocusWindowUnderCursor;
        manager.actions.push_back(act);
    } else if let Some(ws) = to_focus.first() {
        let for_active_workspace = |x: &Window| -> bool {
            helpers::intersect(&ws.tags, &x.tags) && x.type_ != WindowType::Dock
        };
        let handle = match manager.windows.iter().find(|w| for_active_workspace(w)) {
            Some(w) => w.handle,
            None => return true,
        };
        focus_window_by_handle_work(manager, &handle);
    }
    true
}

fn focus_tag_work(manager: &mut Manager, tag: &str) -> Option<()> {
    //no new history if no change
    if let Some(t) = manager.focused_tag(0) {
        if t == tag {
            return None;
        }
    }
    //clean old ones
    manager.focus_manager.tag_history.truncate(10);
    //add this focus to the history
    manager
        .focus_manager
        .tag_history
        .push_front(tag.to_string());

    Some(())
}

/// Create an action to inform the DM of the new current tags.
pub fn update_current_tags(manager: &mut Manager) {
    if let Some(workspace) = manager.focused_workspace() {
        let tags = workspace.tags.clone();
        if tags.is_empty() {
            manager
                .actions
                .push_back(DisplayAction::SetCurrentTags(tags[0].clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focusing_a_workspace_should_make_it_active() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let expected = manager.workspaces[0].clone();
        focus_workspace(&mut manager, &expected);
        let actual = manager.focused_workspace().unwrap();
        assert_eq!(0, actual.id);
    }

    #[test]
    fn focusing_the_same_workspace_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let ws = manager.workspaces[0].clone();
        focus_workspace(&mut manager, &ws);
        let start_length = manager.focused_workspace_history.len();
        focus_workspace(&mut manager, &ws);
        let end_length = manager.focused_workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_window_should_make_it_active() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        window_handler::created(
            &mut manager,
            Window::new(WindowHandle::MockHandle(1), None, None),
            -1,
            -1,
            &Config::default(),
        );
        window_handler::created(
            &mut manager,
            Window::new(WindowHandle::MockHandle(2), None, None),
            -1,
            -1,
            &Config::default(),
        );
        let expected = manager.windows[0].clone();
        focus_window(&mut manager, &expected.handle);
        let actual = manager.focused_window().unwrap().handle;
        assert_eq!(expected.handle, actual);
    }

    #[test]
    fn focusing_the_same_window_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let window = Window::new(WindowHandle::MockHandle(1), None, None);
        window_handler::created(&mut manager, window.clone(), -1, -1, &Config::default());
        focus_window(&mut manager, &window.handle);
        let start_length = manager.focused_workspace_history.len();
        window_handler::created(&mut manager, window.clone(), -1, -1, &Config::default());
        focus_window(&mut manager, &window.handle);
        let end_length = manager.focused_workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_make_it_active() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let expected = "Bla".to_owned();
        focus_tag(&mut manager, &expected, &config);
        let accual = manager.focused_tag(0).unwrap();
        assert_eq!(accual, expected);
    }

    #[test]
    fn focusing_the_same_tag_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let tag = "Bla".to_owned();
        focus_tag(&mut manager, &tag, &config);
        let start_length = manager.focused_tag_history.len();
        focus_tag(&mut manager, &tag, &config);
        let end_length = manager.focused_tag_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_focus_its_workspace() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        focus_tag(&mut manager, &"1".to_owned(), &config);
        let actual = manager.focused_workspace().unwrap();
        let expected = 0;
        assert_eq!(actual.id, expected);
    }

    #[test]
    fn focusing_a_workspace_should_focus_its_tag() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let ws = manager.workspaces[1].clone();
        focus_workspace(&mut manager, &ws);
        let actual = manager.focused_tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_its_tag() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag("2");
        manager.windows.push(window.clone());
        focus_window(&mut manager, &window.handle);
        let actual = manager.focused_tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_workspace() {
        let mut manager = Manager::default();
        let config = &Config::default();
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        screen_create_handler::process(&mut manager, Screen::default(), &config);
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag("2");
        manager.windows.push(window.clone());
        focus_window(&mut manager, &window.handle);
        let actual = manager.focused_workspace().unwrap().id;
        let expected = manager.workspaces[1].id;
        assert_eq!(expected, actual);
    }
}
