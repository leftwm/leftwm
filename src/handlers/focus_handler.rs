#![allow(clippy::wildcard_imports)]
use super::*;
use crate::display_action::DisplayAction;

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
    while manager.focused_workspace_history.len() > 10 {
        manager.focused_workspace_history.pop_back();
    }
    //add this focus to the history
    for (index, ws) in manager.workspaces.iter().enumerate() {
        if ws.id == workspace_id {
            manager.focused_workspace_history.push_front(index);
        }
    }
    Some(())
}

/// Create a `DisplayAction` to cause this window to become focused  
pub fn focus_window(manager: &mut Manager, handle: &WindowHandle) -> bool {
    let window = match focus_window_by_handle_work(manager, handle) {
        Some(w) => w,
        None => return false,
    };

    let mut tags = vec![];
    let mut workspace_id: Option<i32> = None;
    //make sure the focused window's workspace is focused
    for ws in &manager.workspaces {
        if ws.is_displaying(&window) {
            tags = ws.tags.clone();
            workspace_id = Some(ws.id);
            break;
        }
    }
    if let Some(workspace_id) = workspace_id {
        let _ = focus_workspace_work(manager, workspace_id);
    }

    //make sure the focused window's tag is focused
    for tag in &tags {
        if window.has_tag(tag) {
            let _ = focus_tag_work(manager, tag);
            break;
        }
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
    //NOTE: we are intentionally creating the focus event even if we thing this window
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
    while manager.focused_window_history.len() > 10 {
        manager.focused_window_history.pop_back();
    }
    //add this focus to the history
    manager.focused_window_history.push_front(*handle);

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
        .filter(|x| !x.never_focus && x.type_ != WindowType::Dock && x.visible())
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
        .filter(|x| !x.never_focus && x.type_ != WindowType::Dock && x.visible())
        .find(|w| w.contains_point(x, y))
        .cloned();
    match found {
        Some(found) => focus_window(manager, &found.handle),
        None => {
            //backup plan, move focus first window in workspace
            focus_closest_window(manager, x, y)
        }
    }
}

fn focus_closest_window(manager: &mut Manager, x: i32, y: i32) -> bool {
    let mut dists: Vec<(i32, &Window)> = manager
        .windows
        .iter()
        .filter(|x| !x.never_focus && x.type_ != WindowType::Dock && x.visible())
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
    let xs = ((wx - x) * (wx - x)) as f64;
    let ys = ((wy - y) * (wy - y)) as f64;
    (xs + ys).sqrt().abs().floor() as i32
}

pub fn focus_workspace_under_cursor(manager: &mut Manager, x: i32, y: i32) -> bool {
    let mut focused_id = -1;
    if let Some(f) = manager.focused_workspace() {
        focused_id = f.id;
    }
    let to_focus: Option<Workspace> = {
        let mut f: Option<Workspace> = None;
        for w in &manager.workspaces {
            if w.contains_point(x, y) {
                if w.id != focused_id {
                    f = Some(w.clone());
                }
                break;
            }
        }
        f
    };
    if let Some(w) = to_focus {
        return focus_workspace(manager, &w);
    }
    false
}

/// marks a tag as the focused tag
//NOTE: should only be called externally from this file
pub fn focus_tag(manager: &mut Manager, tag: &str) -> bool {
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
    to_focus.iter().for_each(|w| {
        focus_workspace_work(manager, w.id);
    });
    //make sure the focused window is on this workspace
    let act = DisplayAction::FocusWindowUnderCursor;
    manager.actions.push_back(act);
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
    while manager.focused_tag_history.len() > 10 {
        manager.focused_tag_history.pop_back();
    }

    //add this focus to the history
    manager.focused_tag_history.push_front(tag.to_string());

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
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        let expected = manager.workspaces[0].clone();
        focus_workspace(&mut manager, &expected);
        let actual = manager.focused_workspace().unwrap();
        assert_eq!(0, actual.id);
    }

    #[test]
    fn focusing_the_same_workspace_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
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
        screen_create_handler::process(&mut manager, Screen::default());
        window_handler::created(
            &mut manager,
            Window::new(WindowHandle::MockHandle(1), None),
            -1,
            -1,
        );
        window_handler::created(
            &mut manager,
            Window::new(WindowHandle::MockHandle(2), None),
            -1,
            -1,
        );
        let expected = manager.windows[0].clone();
        focus_window(&mut manager, &expected.handle);
        let actual = manager.focused_window().unwrap().handle;
        assert_eq!(expected.handle, actual);
    }

    #[test]
    fn focusing_the_same_window_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        let window = Window::new(WindowHandle::MockHandle(1), None);
        window_handler::created(&mut manager, window.clone(), -1, -1);
        focus_window(&mut manager, &window.handle);
        let start_length = manager.focused_workspace_history.len();
        window_handler::created(&mut manager, window.clone(), -1, -1);
        focus_window(&mut manager, &window.handle);
        let end_length = manager.focused_workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_make_it_active() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        let expected = "Bla".to_owned();
        focus_tag(&mut manager, &expected);
        let accual = manager.focused_tag(0).unwrap();
        assert_eq!(accual, expected);
    }

    #[test]
    fn focusing_the_same_tag_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        let tag = "Bla".to_owned();
        focus_tag(&mut manager, &tag);
        let start_length = manager.focused_tag_history.len();
        focus_tag(&mut manager, &tag);
        let end_length = manager.focused_tag_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_focus_its_workspace() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        focus_tag(&mut manager, &"1".to_owned());
        let actual = manager.focused_workspace().unwrap();
        let expected = 0;
        assert_eq!(actual.id, expected);
    }

    #[test]
    fn focusing_a_workspace_should_focus_its_tag() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        let ws = manager.workspaces[1].clone();
        focus_workspace(&mut manager, &ws);
        let actual = manager.focused_tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_its_tag() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None);
        window.tag("2");
        manager.windows.push(window.clone());
        focus_window(&mut manager, &window.handle);
        let actual = manager.focused_tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_workspace() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None);
        window.tag("2");
        manager.windows.push(window.clone());
        focus_window(&mut manager, &window.handle);
        let actual = manager.focused_workspace().unwrap().id;
        let expected = manager.workspaces[1].id;
        assert_eq!(expected, actual);
    }
}
