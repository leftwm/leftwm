use super::*;
use crate::display_action::DisplayAction;

/// Marks a workspace as the focused workspace.
pub fn focus_workspace(manager: &mut Manager, workspace: &Workspace) -> bool {
    //no new history for if no change
    if let Some(fws) = manager.focused_workspace() {
        if fws.id == workspace.id {
            return false;
        }
    }
    //clean old ones
    while manager.focused_workspace_history.len() > 10 {
        manager.focused_workspace_history.pop_back();
    }
    //add this focus to the history
    for (index, ws) in manager.workspaces.iter().enumerate() {
        if ws.id == workspace.id {
            manager.focused_workspace_history.push_front(index);
        }
    }
    //make sure this workspaces tag is focused
    workspace.tags.iter().for_each(|t| {
        focus_tag(manager, t);
    });
    // create an action to inform the DM
    update_current_tags(manager);
    true
}

/// Marks a window as the focused window.
pub fn focus_window_by_handle(
    manager: &mut Manager,
    handle: &WindowHandle,
    x: i32,
    y: i32,
) -> bool {
    let found: Vec<Window> = manager
        .windows
        .iter()
        .filter(|w| &w.handle == handle)
        .cloned()
        .collect();
    if found.len() == 1 {
        return focus_window(manager, &found[0], x, y);
    }
    false
}

pub fn focus_window(manager: &mut Manager, window: &Window, x: i32, y: i32) -> bool {
    //Docks don't want to get focus. If they do weird things happen. They don't get events...
    if window.type_ == WindowType::Dock {
        return false;
    }

    let result = _focus_window_work(manager, window);
    if !result {
        return false;
    }

    // if the x,y mouse location is inside the to workspace for this window, it needs to be focused
    // this is so the focus of the window gets passed to the underlying workspace
    if !window.tags.is_empty() {
        let main_tag = window.tags[0].clone();
        let to_focus: Vec<Workspace> = manager
            .workspaces
            .iter()
            .filter(|w| w.has_tag(&main_tag))
            .cloned()
            .collect();
        to_focus.iter().for_each(|w| {
            if w.contains_point(x, y) {
                focus_workspace(manager, &w);
            }
        });
    }
    result
}

fn _focus_window_work(manager: &mut Manager, window: &Window) -> bool {
    //no new history for if no change
    if let Some(fw) = manager.focused_window() {
        if fw.handle == window.handle {
            return false;
        }
    }
    //clean old ones
    while manager.focused_window_history.len() > 10 {
        manager.focused_window_history.pop_back();
    }
    //add this focus to the history
    manager
        .focused_window_history
        .push_front(window.handle.clone());
    // inform the window it will be taking focus
    let act = DisplayAction::WindowTakeFocus(window.clone());
    manager.actions.push_back(act);
    true
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

/// Loops over the history and focuses the last window that still exists.
pub fn focus_last_window_that_exists(manager: &mut Manager) -> bool {
    let history = manager.focused_window_history.clone();
    for handle in history {
        for w in manager.windows.clone() {
            if w.handle == handle {
                return _focus_window_work(manager, &w);
            }
        }
    }
    false
}

/*
 * marks a tag as the focused tag
 */
pub fn focus_tag(manager: &mut Manager, tag: &str) -> bool {
    //no new history for if no change
    if let Some(t) = manager.focused_tag() {
        if t == tag {
            return false;
        }
    }
    //clean old ones
    while manager.focused_tag_history.len() > 10 {
        manager.focused_tag_history.pop_back();
    }
    //add this focus to the history
    manager.focused_tag_history.push_front(tag.to_string());
    // check each workspace, if its displaying this tag it should be focused too
    let to_focus: Vec<Workspace> = manager
        .workspaces
        .iter()
        .filter(|w| w.has_tag(tag))
        .cloned()
        .collect();
    to_focus.iter().for_each(|w| {
        focus_workspace(manager, &w);
    });
    true
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
        window_handler::created(&mut manager, Window::new(WindowHandle::MockHandle(1), None));
        window_handler::created(&mut manager, Window::new(WindowHandle::MockHandle(2), None));
        let expected = manager.windows[0].clone();
        focus_window(&mut manager, &expected, 0, 0);
        let actual = manager.focused_window().unwrap().handle.clone();
        assert_eq!(expected.handle, actual);
    }

    #[test]
    fn focusing_the_same_window_shouldnt_add_to_the_history() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        let window = Window::new(WindowHandle::MockHandle(1), None);
        window_handler::created(&mut manager, window.clone());
        focus_window(&mut manager, &window, 0, 0);
        let start_length = manager.focused_workspace_history.len();
        window_handler::created(&mut manager, window.clone());
        focus_window(&mut manager, &window, 0, 0);
        let end_length = manager.focused_workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_make_it_active() {
        let mut manager = Manager::default();
        screen_create_handler::process(&mut manager, Screen::default());
        let expected = "Bla".to_owned();
        focus_tag(&mut manager, &expected);
        let accual = manager.focused_tag().unwrap();
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
        let actual = manager.focused_tag().unwrap();
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
        focus_window(&mut manager, &window, 0, 0);
        let actual = manager.focused_tag().unwrap();
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
        focus_window(&mut manager, &window, 0, 0);
        let actual = manager.focused_workspace().unwrap().id;
        let expected = manager.workspaces[1].id;
        assert_eq!(expected, actual);
    }
}
