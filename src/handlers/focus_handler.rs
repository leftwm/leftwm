use super::*;

/*
 * focuses a workspace. 
 */
pub fn focus_workspace(manager: &mut Manager, workspace: &Workspace) {
    //no new history for if no change
    if let Some(fws) = manager.focused_workspace() {
        if fws.name == workspace.name { return } 
    }
    let mut index = 0;
    //clean old ones
    while manager.focused_workspace_history.len() > 10 {
        manager.focused_workspace_history.pop_back();
    }
    //add this focus to the history
    for ws in &manager.workspaces {
        if ws.name == workspace.name {
            manager.focused_workspace_history.push_front( index );
        }
        index += 1;
    }
}

pub fn focus_window(manager: &mut Manager, workspace: &Window) {}

pub fn focus_tag(manager: &mut Manager, tag: &String) {}


#[test]
fn focusing_a_workspace_should_make_it_retrevable() {
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
