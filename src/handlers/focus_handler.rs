#![allow(clippy::wildcard_imports)]

use super::*;
use crate::{display_action::DisplayAction, models::FocusBehaviour};

impl<CMD> Manager<CMD> {
    /// Marks a workspace as the focused workspace.
    //NOTE: should only be called externally from this file
    pub fn focus_workspace(&mut self, workspace: &Workspace) -> bool {
        if focus_workspace_work(self, workspace.id).is_some() {
            //make sure this workspaces tag is focused
            workspace.tags.iter().for_each(|t| {
                focus_tag_work(self, t);
            });
            // create an action to inform the DM
            self.update_current_tags();
            return true;
        }
        false
    }

    /// Create a `DisplayAction` to cause this window to become focused
    pub fn focus_window(&mut self, handle: &WindowHandle) -> bool {
        let window = match focus_window_by_handle_work(self, handle) {
            Some(w) => w,
            None => return false,
        };

        //make sure the focused window's workspace is focused
        let (focused_window_tag, workspace_id) =
            match self.workspaces.iter().find(|ws| ws.is_displaying(&window)) {
                Some(ws) => (
                    ws.tags.iter().find(|t| window.has_tag(t)).cloned(),
                    Some(ws.id),
                ),
                None => (None, None),
            };
        if let Some(workspace_id) = workspace_id {
            let _ = focus_workspace_work(self, workspace_id);
        }

        //make sure the focused window's tag is focused
        if let Some(tag) = focused_window_tag {
            let _ = focus_tag_work(self, &tag);
        }
        true
    }

    pub fn focus_workspace_under_cursor(&mut self, x: i32, y: i32) -> bool {
        let focused_id = match self.focused_workspace() {
            Some(fws) => fws.id,
            None => None,
        };
        if let Some(w) = self
            .workspaces
            .iter()
            .find(|ws| ws.contains_point(x, y) && ws.id != focused_id)
            .cloned()
        {
            return self.focus_workspace(&w);
        }
        false
    }

    /// marks a tag as the focused tag
    //NOTE: should only be called externally from this file
    pub fn focus_tag(&mut self, tag: &str) -> bool {
        if focus_tag_work(self, tag).is_none() {
            return false;
        }
        // check each workspace, if its displaying this tag it should be focused too
        let to_focus: Vec<Workspace> = self
            .workspaces
            .iter()
            .filter(|w| w.has_tag(tag))
            .cloned()
            .collect();
        for ws in &to_focus {
            focus_workspace_work(self, ws.id);
        }
        //make sure the focused window is on this workspace
        if self.focus_manager.behaviour == FocusBehaviour::Sloppy {
            let act = DisplayAction::FocusWindowUnderCursor;
            self.actions.push_back(act);
        } else if let Some(handle) = self.focus_manager.tags_last_window.get(tag).copied() {
            focus_window_by_handle_work(self, &handle);
        } else if let Some(ws) = to_focus.first() {
            let handle = self
                .windows
                .iter()
                .find(|w| ws.is_managed(w))
                .map(|w| w.handle);
            if let Some(h) = handle {
                focus_window_by_handle_work(self, &h);
            }
        }

        // Unfocus last window if the target tag is empty
        if let Some(window) = self.focused_window().cloned() {
            if !window.tags.contains(&tag.to_owned()) {
                self.actions.push_back(DisplayAction::Unfocus);
                self.focus_manager.window_history.push_front(None);
            }
        }
        true
    }

    pub fn validate_focus_at(&mut self, x: i32, y: i32) -> bool {
        let current = match self.focused_window() {
            Some(w) => w,
            None => return false,
        };
        //only look at windows we can focus
        let found: Option<Window> = self
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
                self.focus_window(&handle)
            }
            None => false,
        }
    }

    pub fn move_focus_to_point(&mut self, x: i32, y: i32) -> bool {
        let handle_found: Option<WindowHandle> = self
            .windows
            .iter()
            .filter(|x| x.can_focus())
            .find(|w| w.contains_point(x, y))
            .map(|w| w.handle);
        match handle_found {
            Some(found) => self.focus_window(&found),
            //backup plan, move focus closest window in workspace
            None => focus_closest_window(self, x, y),
        }
    }

    /// Create an action to inform the DM of the new current tags.
    pub fn update_current_tags(&mut self) {
        if let Some(workspace) = self.focused_workspace() {
            if let Some(tag) = workspace.tags.first().cloned() {
                self.actions.push_back(DisplayAction::SetCurrentTags(tag));
            }
        }
    }
}

fn focus_workspace_work<CMD>(manager: &mut Manager<CMD>, workspace_id: Option<i32>) -> Option<()> {
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
fn focus_window_by_handle_work<CMD>(
    manager: &mut Manager<CMD>,
    handle: &WindowHandle,
) -> Option<Window> {
    //Docks don't want to get focus. If they do weird things happen. They don't get events...
    //Do the focus, Add the action to the list of action
    let found: &Window = manager.windows.iter().find(|w| &w.handle == handle)?;
    if found.is_unmanaged() {
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
    manager
        .focus_manager
        .window_history
        .push_front(Some(*handle));

    Some(found.clone())
}

fn focus_closest_window<CMD>(manager: &mut Manager<CMD>, x: i32, y: i32) -> bool {
    let ws = match manager.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
        Some(ws) => ws,
        None => return false,
    };
    let mut dists: Vec<(i32, &Window)> = manager
        .windows
        .iter()
        .filter(|x| ws.is_managed(x) && x.can_focus())
        .map(|w| (distance(w, x, y), w))
        .collect();
    dists.sort_by(|a, b| (a.0).cmp(&b.0));
    if let Some(first) = dists.get(0) {
        let handle = first.1.handle;
        return manager.focus_window(&handle);
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

fn focus_tag_work<CMD>(manager: &mut Manager<CMD>, tag: &str) -> Option<()> {
    //no new history if no change
    if let Some(t) = manager.focus_manager.tag(0) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focusing_a_workspace_should_make_it_active() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let expected = manager.workspaces[0].clone();
        manager.focus_workspace(&expected);
        let actual = manager.focused_workspace().unwrap();
        assert_eq!(Some(0), actual.id);
    }

    #[test]
    fn focusing_the_same_workspace_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let ws = manager.workspaces[0].clone();
        manager.focus_workspace(&ws);
        let start_length = manager.focus_manager.workspace_history.len();
        manager.focus_workspace(&ws);
        let end_length = manager.focus_manager.workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_window_should_make_it_active() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(1), None, None),
            -1,
            -1,
        );
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(2), None, None),
            -1,
            -1,
        );
        let expected = manager.windows[0].clone();
        manager.focus_window(&expected.handle);
        let actual = manager.focused_window().unwrap().handle;
        assert_eq!(expected.handle, actual);
    }

    #[test]
    fn focusing_the_same_window_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let window = Window::new(WindowHandle::MockHandle(1), None, None);
        manager.window_created_handler(window.clone(), -1, -1);
        manager.focus_window(&window.handle);
        let start_length = manager.focus_manager.workspace_history.len();
        manager.window_created_handler(window.clone(), -1, -1);
        manager.focus_window(&window.handle);
        let end_length = manager.focus_manager.workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_make_it_active() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let expected = "Bla".to_owned();
        manager.focus_tag(&expected);
        let accual = manager.focus_manager.tag(0).unwrap();
        assert_eq!(accual, expected);
    }

    #[test]
    fn focusing_the_same_tag_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let tag = "Bla".to_owned();
        manager.focus_tag(&tag);
        let start_length = manager.focus_manager.tag_history.len();
        manager.focus_tag(&tag);
        let end_length = manager.focus_manager.tag_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_focus_its_workspace() {
        let mut manager = Manager::new_test(vec!["1".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.focus_tag(&"1".to_owned());
        let actual = manager.focused_workspace().unwrap();
        let expected = Some(0);
        assert_eq!(actual.id, expected);
    }

    #[test]
    fn focusing_a_workspace_should_focus_its_tag() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let ws = manager.workspaces[1].clone();
        manager.focus_workspace(&ws);
        let actual = manager.focus_manager.tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_its_tag() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag("2");
        manager.windows.push(window.clone());
        manager.focus_window(&window.handle);
        let actual = manager.focus_manager.tag(0).unwrap();
        assert_eq!("2", actual);
    }

    #[test]
    fn focusing_a_window_should_focus_workspace() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag("2");
        manager.windows.push(window.clone());
        manager.focus_window(&window.handle);
        let actual = manager.focused_workspace().unwrap().id;
        let expected = manager.workspaces[1].id;
        assert_eq!(expected, actual);
    }

    #[test]
    fn focusing_an_empty_tag_should_unfocus_any_focused_window() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag("1");
        manager.windows.push(window.clone());
        manager.focus_window(&window.handle);
        manager.focus_tag("2");
        let focused = manager.focused_window();
        assert!(focused.is_none());
    }
}
