use super::{Manager, Window, WindowChange, WindowType, Workspace};
use crate::child_process::exec_shell;
use crate::config::{Config, InsertBehavior};
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
use crate::models::{WindowHandle, WindowState, Xyhw};
use crate::state::State;
use crate::utils::helpers;
use std::env;
use std::str::FromStr;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn window_created_handler(&mut self, mut window: Window, x: i32, y: i32) -> bool {
        // Don't add the window if the manager already knows about it.
        if self.state.windows.iter().any(|w| w.handle == window.handle) {
            return false;
        }

        // Setup any predifined hooks.
        self.config.setup_predefined_window(&mut window);
        let mut is_first = false;
        let mut on_same_tag = true;
        // Random value
        let mut layout: Layout = Layout::MainAndVertStack;
        setup_window(
            &mut self.state,
            &mut window,
            (x, y),
            &mut layout,
            &mut is_first,
            &mut on_same_tag,
        );
        self.config.load_window(&mut window);
        insert_window(&mut self.state, &mut window, layout);

        let follow_mouse = self.state.focus_manager.focus_new_windows
            && self.state.focus_manager.behaviour.is_sloppy()
            && self.state.focus_manager.sloppy_mouse_follows_focus
            && on_same_tag;
        // Let the DS know we are managing this window.
        let act = DisplayAction::AddedWindow(window.handle, window.floating(), follow_mouse);
        self.state.actions.push_back(act);

        // Let the DS know the correct desktop to find this window.
        if window.tag.is_some() {
            let act = DisplayAction::SetWindowTag(window.handle, window.tag);
            self.state.actions.push_back(act);
        }

        // Tell the WM to reevaluate the stacking order, so the new window is put in the correct layer
        self.state.sort_windows();

        if (self.state.focus_manager.focus_new_windows || is_first) && on_same_tag {
            self.state.focus_window(&window.handle);
        }

        if let Some(cmd) = &self.config.on_new_window_cmd() {
            exec_shell(cmd, &mut self.children);
        }

        true
    }

    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn window_destroyed_handler(&mut self, handle: &WindowHandle) -> bool {
        // Find the next or previous window on the workspace.
        let new_handle = self.get_next_or_previous_handle(handle);
        // If there is a parent we would want to focus it.
        let (transient, floating, visible) =
            match self.state.windows.iter().find(|w| &w.handle == handle) {
                Some(window) => (window.transient, window.floating(), window.visible()),
                None => return false,
            };
        self.state
            .focus_manager
            .tags_last_window
            .retain(|_, h| h != handle);
        self.state.windows.retain(|w| &w.handle != handle);

        // Make sure the workspaces do not draw on the docks.
        update_workspace_avoid_list(&mut self.state);

        let focused = self.state.focus_manager.window_history.get(0);
        // Make sure focus is recalculated if we closed the currently focused window
        if focused == Some(&Some(*handle)) {
            if self.state.focus_manager.behaviour.is_sloppy()
                && self.state.focus_manager.sloppy_mouse_follows_focus
            {
                let act = DisplayAction::FocusWindowUnderCursor;
                self.state.actions.push_back(act);
            } else if let Some(parent) =
                find_transient_parent(&self.state.windows, transient).map(|p| p.handle)
            {
                self.state.focus_window(&parent);
            } else if let Some(handle) = new_handle {
                self.state.focus_window(&handle);
            } else {
                let act = DisplayAction::Unfocus(Some(*handle), floating);
                self.state.actions.push_back(act);
                self.state.focus_manager.window_history.push_front(None);
            }
        }

        // Only update windows if this window is visible.
        visible
    }

    pub fn window_changed_handler(&mut self, change: WindowChange) -> bool {
        let mut changed = false;
        let mut fullscreen_changed = false;
        let strut_changed = change.strut.is_some();
        let windows = self.state.windows.clone();
        if let Some(window) = self
            .state
            .windows
            .iter_mut()
            .find(|w| w.handle == change.handle)
        {
            if let Some(ref states) = change.states {
                let change_contains = states.contains(&WindowState::Fullscreen);
                fullscreen_changed = change_contains || window.is_fullscreen();
            }
            let container = match find_transient_parent(&windows, window.transient) {
                Some(parent) => Some(parent.exact_xyhw()),
                None if window.r#type == WindowType::Dialog => self
                    .state
                    .workspaces
                    .iter()
                    .find(|ws| ws.tag == window.tag)
                    .map(|ws| ws.xyhw),
                _ => None,
            };

            changed = change.update(window, container);
            if window.r#type == WindowType::Dock {
                update_workspace_avoid_list(&mut self.state);
                // Don't let changes from docks re-render the worker. This will result in an
                // infinite loop. Just be patient a rerender will occur.
            }
        }
        if fullscreen_changed {
            // Update `dock` windows once, so they can recieve mouse click events again.
            // This is necessary, since we exclude them from the general update loop above.
            if let Some(windows) = self
                .state
                .windows
                .iter()
                .find(|w| w.r#type == WindowType::Dock)
            {
                self.display_server.update_windows(vec![windows]);
            }

            // Reorder windows.
            self.state.sort_windows();
        }
        if strut_changed {
            self.state.update_static();
        }
        changed
    }

    /// Find the next or previous window on the currently focused workspace.
    /// May return `None` if no other window is present.
    pub fn get_next_or_previous_handle(&mut self, handle: &WindowHandle) -> Option<WindowHandle> {
        let focused_workspace = self.state.focus_manager.workspace(&self.state.workspaces)?;
        let on_focused_workspace = |x: &Window| -> bool { focused_workspace.is_managed(x) };
        let mut windows_on_workspace =
            helpers::vec_extract(&mut self.state.windows, on_focused_workspace);
        let is_handle = |x: &Window| -> bool { &x.handle == handle };
        let new_handle = helpers::relative_find(&windows_on_workspace, is_handle, 1, false)
            .or_else(|| helpers::relative_find(&windows_on_workspace, is_handle, -1, false))
            .map(|w| w.handle);
        self.state.windows.append(&mut windows_on_workspace);
        new_handle
    }
}

// Helper functions.

fn find_terminal(state: &State, pid: Option<u32>) -> Option<&Window> {
    // Get $SHELL, e.g. /bin/zsh
    let shell_path = env::var("SHELL").ok()?;
    // Remove /bin/
    let shell = shell_path.split('/').last()?;
    // Try and find the shell that launched this app, if such a thing exists.
    let is_terminal = |pid: u32| -> Option<bool> {
        let parent = std::fs::read(format!("/proc/{}/comm", pid)).ok()?;
        let parent_bytes = parent.split(|&c| c == b' ').next()?;
        let parent_str = std::str::from_utf8(parent_bytes).ok()?.strip_suffix('\n')?;
        Some(parent_str == shell)
    };

    let get_parent = |pid: u32| -> Option<u32> {
        let stat = std::fs::read(format!("/proc/{}/stat", pid)).ok()?;
        let ppid_bytes = stat.split(|&c| c == b' ').nth(3)?;
        let ppid_str = std::str::from_utf8(ppid_bytes).ok()?;
        let ppid_u32 = u32::from_str(ppid_str).ok()?;
        Some(ppid_u32)
    };

    let pid = pid?;
    let shell_id = get_parent(pid)?;
    if is_terminal(shell_id)? {
        let terminal = get_parent(shell_id)?;
        return state.windows.iter().find(|w| w.pid == Some(terminal));
    }

    None
}

fn find_transient_parent(windows: &[Window], transient: Option<WindowHandle>) -> Option<&Window> {
    let mut transient = transient?;
    loop {
        transient = if let Some(found) = windows
            .iter()
            .find(|x| x.handle == transient)
            .and_then(|x| x.transient)
        {
            found
        } else {
            return windows.iter().find(|x| x.handle == transient);
        };
    }
}

fn insert_window(state: &mut State, window: &mut Window, layout: Layout) {
    let mut was_fullscreen = false;
    if window.r#type == WindowType::Normal {
        let for_active_workspace = |x: &Window| -> bool { window.tag == x.tag && x.is_managed() };
        // Only minimize when the new window is type normal.
        if let Some(fsw) = state
            .windows
            .iter_mut()
            .find(|w| for_active_workspace(w) && w.is_fullscreen())
        {
            let act =
                DisplayAction::SetState(fsw.handle, !fsw.is_fullscreen(), WindowState::Fullscreen);
            state.actions.push_back(act);
            was_fullscreen = true;
        }
        if matches!(layout, Layout::Monocle | Layout::MainAndDeck) {
            // Extract the current windows on the same workspace.
            let mut to_reorder = helpers::vec_extract(&mut state.windows, for_active_workspace);
            if layout == Layout::Monocle || to_reorder.is_empty() {
                // When in monocle we want the new window to be fullscreen if the previous window was
                // fullscreen.
                if was_fullscreen {
                    let act = DisplayAction::SetState(
                        window.handle,
                        !window.is_fullscreen(),
                        WindowState::Fullscreen,
                    );
                    state.actions.push_back(act);
                }
                // Place the window above the other windows on the workspace.
                to_reorder.insert(0, window.clone());
            } else {
                // Place the window second within the other windows on the workspace.
                to_reorder.insert(1, window.clone());
            }
            state.windows.append(&mut to_reorder);
            return;
        }
    }

    // If a window is a dialog, splash, or scractchpad we want it to be at the top.
    if window.r#type == WindowType::Dialog
        || window.r#type == WindowType::Splash
        || window.r#type == WindowType::Utility
        || is_scratchpad(state, window)
    {
        state.windows.insert(0, window.clone());
        return;
    }

    let current_index = state
        .focus_manager
        .window(&state.windows)
        .and_then(|current| {
            state
                .windows
                .iter()
                .position(|w| w.handle == current.handle)
        })
        .unwrap_or(0);

    // Past special cases we just insert the window based on the configured insert behavior
    match state.insert_behavior {
        InsertBehavior::Top => state.windows.insert(0, window.clone()),
        InsertBehavior::Bottom => state.windows.push(window.clone()),
        InsertBehavior::AfterCurrent if current_index < state.windows.len() => {
            state.windows.insert(current_index + 1, window.clone());
        }
        InsertBehavior::AfterCurrent | InsertBehavior::BeforeCurrent => {
            state.windows.insert(current_index, window.clone());
        }
    }
}

fn is_scratchpad(state: &State, window: &Window) -> bool {
    state
        .active_scratchpads
        .iter()
        .any(|(_, id)| id.iter().any(|id| window.pid == Some(*id)))
}

fn set_relative_floating(window: &mut Window, ws: &Workspace, outer: Xyhw) {
    window.set_floating(true);
    window.normal = ws.xyhw;
    let xyhw = window.requested.map_or_else(
        || ws.center_halfed(),
        |mut requested| {
            requested.center_relative(outer, window.border);
            if ws.xyhw.contains_xyhw(&requested) {
                requested
            } else {
                requested.center_relative(ws.xyhw, window.border);
                requested
            }
        },
    );
    window.set_floating_exact(xyhw);
}

fn setup_window(
    state: &mut State,
    window: &mut Window,
    xy: (i32, i32),
    layout: &mut Layout,
    is_first: &mut bool,
    on_same_tag: &mut bool,
) {
    // When adding a window we add to the workspace under the cursor, This isn't necessarily the
    // focused workspace. If the workspace is empty, it might not have received focus. This is so
    // the workspace that has windows on its is still active not the empty workspace.
    let ws: Option<&Workspace> = state
        .workspaces
        .iter()
        .find(|ws| ws.xyhw.contains_point(xy.0, xy.1) && state.focus_manager.behaviour.is_sloppy())
        .or_else(|| state.focus_manager.workspace(&state.workspaces)); // Backup plan.

    if let Some(ws) = ws {
        // Setup basic variables.
        let for_active_workspace = |x: &Window| -> bool { ws.tag == x.tag && x.is_managed() };
        *is_first = !state.windows.iter().any(|w| for_active_workspace(w));
        // May have been set by a predefined tag.
        if window.tag.is_none() {
            window.tag =
                find_terminal(state, window.pid).map_or_else(|| ws.tag, |terminal| terminal.tag);
        }
        *on_same_tag = ws.tag == window.tag;
        *layout = ws.layout;

        // Setup a scratchpad window.
        if let Some((scratchpad_name, _)) = state
            .active_scratchpads
            .iter()
            .find(|(_, id)| id.iter().any(|id| Some(*id) == window.pid))
        {
            window.set_floating(true);
            if let Some(s) = state
                .scratchpads
                .iter()
                .find(|s| *scratchpad_name == s.name)
            {
                let new_float_exact = s.xyhw(&ws.xyhw);
                window.normal = ws.xyhw;
                window.set_floating_exact(new_float_exact);
                return;
            }
        }

        // Setup a child window.
        if let Some(parent) = find_transient_parent(&state.windows, window.transient) {
            // This is currently for vlc, this probably will need to be more general if another
            // case comes up where we don't want to move the window.
            if window.r#type != WindowType::Utility {
                set_relative_floating(window, ws, parent.exact_xyhw());
                return;
            }
        }

        // Setup window based on type.
        match window.r#type {
            WindowType::Normal => {
                window.apply_margin_multiplier(ws.margin_multiplier);
                if window.floating() {
                    set_relative_floating(window, ws, ws.xyhw);
                }
            }
            WindowType::Dialog => {
                if window.can_resize() {
                    window.set_floating(true);
                    let new_float_exact = ws.center_halfed();
                    window.normal = ws.xyhw;
                    window.set_floating_exact(new_float_exact);
                } else {
                    set_relative_floating(window, ws, ws.xyhw);
                }
            }
            WindowType::Splash => set_relative_floating(window, ws, ws.xyhw),
            _ => {}
        }
        return;
    }

    // Setup a window is workspace is `None`. This shouldn't really happen.
    window.tag = Some(1);
    if is_scratchpad(state, window) {
        if let Some(scratchpad_tag) = state.tags.get_hidden_by_label("NSP") {
            window.tag(&scratchpad_tag.id);
            window.set_floating(true);
        }
    }
}

fn update_workspace_avoid_list(state: &mut State) {
    let mut avoid = vec![];
    state
        .windows
        .iter()
        .filter(|w| w.r#type == WindowType::Dock)
        .filter_map(|w| w.strut.map(|strut| (w.handle, strut)))
        .for_each(|(handle, to_avoid)| {
            tracing::debug!("AVOID STRUT:[{:?}] {:?}", handle, to_avoid);
            avoid.push(to_avoid);
        });
    for ws in &mut state.workspaces {
        let struts = avoid
            .clone()
            .into_iter()
            .filter(|s| {
                let (x, y) = s.center();
                ws.contains_point(x, y)
            })
            .collect();
        ws.avoid = struts;
        ws.update_avoided_areas();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Screen;
    use crate::Manager;

    #[test]
    fn insert_behavior_bottom_add_window_at_the_end_of_the_stack() {
        let mut manager = Manager::new_test(vec![]);
        manager.state.insert_behavior = InsertBehavior::Bottom;

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

        let expected = vec![WindowHandle::MockHandle(1), WindowHandle::MockHandle(2)];

        let actual: Vec<WindowHandle> = manager.state.windows.iter().map(|w| w.handle).collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn insert_behavior_top_add_window_at_the_top_of_the_stack() {
        let mut manager = Manager::new_test(vec![]);
        manager.state.insert_behavior = InsertBehavior::Top;

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

        let expected = vec![WindowHandle::MockHandle(2), WindowHandle::MockHandle(1)];
        let actual: Vec<WindowHandle> = manager.state.windows.iter().map(|w| w.handle).collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn insert_behavior_after_current_add_window_after_the_current_window() {
        let mut manager = Manager::new_test(vec![]);
        manager.state.insert_behavior = InsertBehavior::AfterCurrent;

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
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(3), None, None),
            -1,
            -1,
        );

        let expected = vec![
            WindowHandle::MockHandle(1),
            WindowHandle::MockHandle(3),
            WindowHandle::MockHandle(2),
        ];
        let actual: Vec<WindowHandle> = manager.state.windows.iter().map(|w| w.handle).collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn insert_behavior_before_current_add_window_before_the_current_window() {
        let mut manager = Manager::new_test(vec![]);
        manager.state.insert_behavior = InsertBehavior::BeforeCurrent;

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

        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(3), None, None),
            -1,
            -1,
        );

        let expected = vec![
            WindowHandle::MockHandle(2),
            WindowHandle::MockHandle(3),
            WindowHandle::MockHandle(1),
        ];
        let actual: Vec<WindowHandle> = manager.state.windows.iter().map(|w| w.handle).collect();

        assert_eq!(actual, expected);
    }
}
