use super::*;

/*
 * marks a workspace as the focused workspace
 */
pub fn focus_workspace(manager: &mut Manager, workspace: &Workspace) {
    //no new history for if no change
    if let Some(fws) = manager.focused_workspace() {
        if fws.name == workspace.name {
            return;
        }
    }
    //clean old ones
    while manager.focused_workspace_history.len() > 10 {
        manager.focused_workspace_history.pop_back();
    }
    //add this focus to the history
    let mut index = 0;
    for ws in &manager.workspaces {
        if ws.name == workspace.name {
            manager.focused_workspace_history.push_front(index);
        }
        index += 1;
    }
}

/*
 * marks a window as the focused window
 */
pub fn focus_window(manager: &mut Manager, window: &Window) {
    //no new history for if no change
    if let Some(fw) = manager.focused_window() {
        if fw.handle == window.handle {
            return;
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
}

pub fn focus_tag(manager: &mut Manager, tag: &String) {}

#[test]
fn focusing_a_workspace_should_make_it_active() {
    let mut manager = Manager::default();
    screen_create_handler::process(&mut manager, Screen::default());
    screen_create_handler::process(&mut manager, Screen::default());
    let expected = manager.workspaces[0].clone();
    focus_workspace(&mut manager, &expected);
    let actual = manager.focused_workspace().unwrap();
    assert_eq!("0", actual.name);
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
    focus_window(&mut manager, &expected);
    let actual = manager.focused_window().unwrap().handle.clone();
    assert_eq!(expected.handle, actual);
}

#[test]
fn focusing_the_same_window_shouldnt_add_to_the_history() {
    let mut manager = Manager::default();
    screen_create_handler::process(&mut manager, Screen::default());
    let window = Window::new(WindowHandle::MockHandle(1), None);
    window_handler::created(&mut manager, window.clone());
    focus_window(&mut manager, &window);
    let start_length = manager.focused_workspace_history.len();
    window_handler::created(&mut manager, window.clone());
    focus_window(&mut manager, &window);
    let end_length = manager.focused_workspace_history.len();
    assert_eq!(start_length, end_length, "expected no new history event");
}
