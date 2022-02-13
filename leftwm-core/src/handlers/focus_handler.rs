#![allow(clippy::wildcard_imports)]

use super::*;
use crate::models::TagId;
use crate::state::State;
use crate::{display_action::DisplayAction, models::FocusBehaviour};

impl State {
    pub fn handle_window_focus(&mut self, handle: &WindowHandle) {
        match self.focus_manager.behaviour {
            FocusBehaviour::Sloppy => {
                let act = DisplayAction::MoveMouseOver(*handle, false);
                self.actions.push_back(act);
            }
            _ => self.focus_window(handle),
        }
    }

    /// Focuses the given window.
    pub fn focus_window(&mut self, handle: &WindowHandle) {
        let window = match self.focus_window_work(handle) {
            Some(w) => w,
            None => return,
        };

        // Make sure the focused window's workspace is focused.
        let (focused_window_tag, workspace_id) =
            match self.workspaces.iter().find(|ws| ws.is_displaying(&window)) {
                Some(ws) => (
                    ws.tags.iter().find(|t| window.has_tag(t)).copied(),
                    Some(ws.id),
                ),
                None => (None, None),
            };

        if let Some(workspace_id) = workspace_id {
            let _ = self.focus_workspace_work(workspace_id);
        }

        // Make sure the focused window's tag is focused.
        if let Some(tag) = focused_window_tag {
            let _ = self.focus_tag_work(tag);
        }
    }

    /// Focuses the given workspace.
    // NOTE: Should only be called externally from this file.
    pub fn focus_workspace(&mut self, workspace: &Workspace) {
        if self.focus_workspace_work(workspace.id) {
            // Make sure this workspaces tag is focused.
            workspace.tags.iter().for_each(|t| {
                self.focus_tag_work(*t);

                if let Some(handle) = self.focus_manager.tags_last_window.get(t).copied() {
                    self.focus_window_work(&handle);
                } else {
                    self.unfocus_current_window();
                }
            });
        }
    }

    /// Focuses the given tag.
    // NOTE: Should only be called externally from this file.
    pub fn focus_tag(&mut self, tag: &TagId) {
        if !self.focus_tag_work(*tag) {
            return;
        }
        // Check each workspace, if its displaying this tag it should be focused too.
        let to_focus: Vec<Workspace> = self
            .workspaces
            .iter()
            .filter(|w| w.has_tag(tag))
            .cloned()
            .collect();
        for ws in &to_focus {
            self.focus_workspace_work(ws.id);
        }
        // Make sure the focused window is on this workspace.
        if self.focus_manager.behaviour.is_sloppy() {
            let act = DisplayAction::FocusWindowUnderCursor;
            self.actions.push_back(act);
        } else if let Some(handle) = self.focus_manager.tags_last_window.get(tag).copied() {
            self.focus_window_work(&handle);
        } else if let Some(ws) = to_focus.first() {
            let handle = self
                .windows
                .iter()
                .find(|w| ws.is_managed(w))
                .map(|w| w.handle);
            if let Some(h) = handle {
                self.focus_window_work(&h);
            }
        }

        // Unfocus last window if the target tag is empty
        if let Some(window) = self.focus_manager.window(&self.windows) {
            if !window.tags.contains(tag) {
                self.unfocus_current_window();
            }
        }
    }

    fn focus_window_work(&mut self, handle: &WindowHandle) -> Option<Window> {
        // Find the handle in our managed windows.
        let found: &Window = self.windows.iter().find(|w| &w.handle == handle)?;
        // Docks don't want to get focus. If they do weird things happen. They don't get events...
        if found.is_unmanaged() {
            return None;
        }
        let previous = self.focus_manager.window(&self.windows);
        // No new history if no change.
        if let Some(previous) = previous {
            if &previous.handle == handle {
                // Return some so we still update the visuals.
                return Some(found.clone());
            }
            for tag_id in &previous.tags {
                self.focus_manager
                    .tags_last_window
                    .insert(*tag_id, previous.handle);
            }
        }

        // Clean old history.
        self.focus_manager.window_history.truncate(10);
        // Add this focus change to the history.
        self.focus_manager.window_history.push_front(Some(*handle));

        let act = DisplayAction::WindowTakeFocus {
            window: found.clone(),
            previous_window: previous.cloned(),
        };
        self.actions.push_back(act);

        Some(found.clone())
    }

    fn focus_workspace_work(&mut self, workspace_id: Option<i32>) -> bool {
        //no new history if no change
        if let Some(fws) = self.focus_manager.workspace(&self.workspaces) {
            if fws.id == workspace_id {
                return false;
            }
        }
        // Clean old history.
        self.focus_manager.workspace_history.truncate(10);
        // Add this focus to the history.
        if let Some(index) = self.workspaces.iter().position(|x| x.id == workspace_id) {
            self.focus_manager.workspace_history.push_front(index);
            return true;
        }
        false
    }

    fn focus_tag_work(&mut self, tag: TagId) -> bool {
        if let Some(current_tag) = self.focus_manager.tag(0) {
            if current_tag == tag {
                return false;
            }
        };
        // Clean old history.
        self.focus_manager.tag_history.truncate(10);
        // Add this focus to the history.
        self.focus_manager.tag_history.push_front(tag);

        let act = DisplayAction::SetCurrentTags(vec![tag]);
        self.actions.push_back(act);
        true
    }

    pub fn focus_workspace_under_cursor(&mut self, x: i32, y: i32) {
        let focused_id = match self.focus_manager.workspace(&self.workspaces) {
            Some(fws) => fws.id,
            None => None,
        };
        if let Some(w) = self
            .workspaces
            .iter()
            .find(|ws| ws.contains_point(x, y) && ws.id != focused_id)
            .cloned()
        {
            self.focus_workspace(&w);
        }
    }

    pub fn validate_focus_at(&mut self, handle: &WindowHandle) {
        // If the window is already focused do nothing.
        if let Some(current) = self.focus_manager.window(&self.windows) {
            if &current.handle == handle {
                return;
            }
        }
        // Focus the window only if it is also focusable.
        if self
            .windows
            .iter()
            .any(|w| w.can_focus() && &w.handle == handle)
        {
            self.focus_window(handle);
        }
    }

    pub fn move_focus_to_point(&mut self, x: i32, y: i32) {
        let handle_found: Option<WindowHandle> = self
            .windows
            .iter()
            .filter(|x| x.can_focus())
            .find(|w| w.contains_point(x, y))
            .map(|w| w.handle);
        match handle_found {
            Some(found) => self.focus_window(&found),
            //backup plan, move focus closest window in workspace
            None => self.focus_closest_window(x, y),
        }
    }

    fn focus_closest_window(&mut self, x: i32, y: i32) {
        let ws = match self.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
            Some(ws) => ws,
            None => return,
        };
        let mut dists: Vec<(i32, &Window)> = self
            .windows
            .iter()
            .filter(|x| ws.is_managed(x) && x.can_focus())
            .map(|w| (distance(w, x, y), w))
            .collect();
        dists.sort_by(|a, b| (a.0).cmp(&b.0));
        if let Some(first) = dists.get(0) {
            let handle = first.1.handle;
            self.focus_window(&handle);
        }
    }

    fn unfocus_current_window(&mut self) {
        if let Some(window) = self.focus_manager.window(&self.windows) {
            self.actions.push_back(DisplayAction::Unfocus(
                Some(window.handle),
                window.floating(),
            ));
            self.focus_manager.window_history.push_front(None);
            for tag_id in &window.tags {
                self.focus_manager
                    .tags_last_window
                    .insert(*tag_id, window.handle);
            }
        }
    }
}

// Square root not needed as we are only interested in the comparison.
fn distance(window: &Window, x: i32, y: i32) -> i32 {
    // (x_2-x_1)²+(y_2-y_1)²
    let (wx, wy) = window.calculated_xyhw().center();
    let xs = (wx - x) * (wx - x);
    let ys = (wy - y) * (wy - y);
    xs + ys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manager;

    #[test]
    fn focusing_a_workspace_should_make_it_active() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let expected = manager.state.workspaces[0].clone();
        manager.state.focus_workspace(&expected);
        let actual = manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)
            .unwrap();
        assert_eq!(Some(0), actual.id);
    }

    #[test]
    fn focusing_a_workspace_should_focus_its_last_active_window() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager
            .state
            .focus_workspace(&manager.state.workspaces[0].clone());
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

        manager
            .state
            .focus_workspace(&manager.state.workspaces[0].clone());

        let expected = manager.state.windows.get(1).map(|w| w.handle);
        manager.state.focus_window(&expected.unwrap());

        manager
            .state
            .focus_workspace(&manager.state.workspaces[1].clone());
        manager
            .state
            .focus_workspace(&manager.state.workspaces[0].clone());

        let actual = manager
            .state
            .focus_manager
            .window(&manager.state.windows)
            .map(|w| w.handle);

        assert_eq!(expected, actual);
    }

    #[test]
    fn focusing_the_same_workspace_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let ws = manager.state.workspaces[0].clone();
        manager.state.focus_workspace(&ws);
        let start_length = manager.state.focus_manager.workspace_history.len();
        manager.state.focus_workspace(&ws);
        let end_length = manager.state.focus_manager.workspace_history.len();
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
        let expected = manager.state.windows[0].clone();
        manager.state.focus_window(&expected.handle);
        let actual = manager
            .state
            .focus_manager
            .window(&manager.state.windows)
            .unwrap()
            .handle;
        assert_eq!(expected.handle, actual);
    }

    #[test]
    fn focusing_the_same_window_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let window = Window::new(WindowHandle::MockHandle(1), None, None);
        manager.window_created_handler(window.clone(), -1, -1);
        manager.state.focus_window(&window.handle);
        let start_length = manager.state.focus_manager.workspace_history.len();
        manager.window_created_handler(window.clone(), -1, -1);
        manager.state.focus_window(&window.handle);
        let end_length = manager.state.focus_manager.workspace_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_make_it_active() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let state = &mut manager.state;
        let expected: usize = 1;
        state.focus_tag(&expected);
        let actual = state.focus_manager.tag(0).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn focusing_the_same_tag_shouldnt_add_to_the_history() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let state = &mut manager.state;
        let tag: usize = 1;
        state.focus_tag(&tag);
        let start_length = state.focus_manager.tag_history.len();
        state.focus_tag(&tag);
        let end_length = state.focus_manager.tag_history.len();
        assert_eq!(start_length, end_length, "expected no new history event");
    }

    #[test]
    fn focusing_a_tag_should_focus_its_workspace() {
        let mut manager = Manager::new_test(vec!["1".to_string()]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.state.focus_tag(&1);
        let actual = manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)
            .unwrap();
        assert_eq!(actual.id, Some(0));
    }

    #[test]
    fn focusing_a_workspace_should_focus_its_tag() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let ws = manager.state.workspaces[1].clone();
        manager.state.focus_workspace(&ws);
        let actual = manager.state.focus_manager.tag(0).unwrap();
        assert_eq!(2, actual);
    }

    #[test]
    fn focusing_a_window_should_focus_its_tag() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag(&2);
        manager.state.windows.push(window.clone());
        manager.state.focus_window(&window.handle);
        let actual = manager.state.focus_manager.tag(0).unwrap();
        assert_eq!(2, actual);
    }

    #[test]
    fn focusing_a_window_should_focus_workspace() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag(&2);
        manager.state.windows.push(window.clone());
        manager.state.focus_window(&window.handle);
        let actual = manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)
            .unwrap()
            .id;
        let expected = manager.state.workspaces[1].id;
        assert_eq!(expected, actual);
    }

    #[test]
    fn focusing_an_empty_tag_should_unfocus_any_focused_window() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        let mut window = Window::new(WindowHandle::MockHandle(1), None, None);
        window.tag(&1);
        manager.state.windows.push(window.clone());
        manager.state.focus_window(&window.handle);
        manager.state.focus_tag(&2);
        let focused = manager.state.focus_manager.window(&manager.state.windows);
        assert!(focused.is_none());
    }
}
