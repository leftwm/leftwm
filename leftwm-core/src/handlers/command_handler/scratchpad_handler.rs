/// # Module for handling the scratchpad related commands
/// Global facing structures are `ReleaseScratchPadOption` and `Direction` which are re-exported at
/// the upper levels to make it easier to use.
///
/// All the other public methods are only ment for the use as command handlers
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{
    child_process::{exec_shell, ChildID},
    models::{Handle, ScratchPadName, TagId, WindowHandle},
    Command, Config, DisplayAction, DisplayServer, Manager, Window,
};

/// Describes the options for the release scratchpad command
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum ReleaseScratchPadOption<H: Handle> {
    /// Release a window from a scratchpad given a window handle
    #[serde(bound = "")]
    Handle(WindowHandle<H>),
    /// Release a window from a scratchpad given a scratchpad name, the most upper window in the
    /// scratchpad queue will be released
    ScratchpadName(ScratchPadName),
    /// Release the currently focused window from its scratchpad
    None,
}

/// Hide scratchpad window:
/// Expects that the window handle is a valid handle to a visible scratchpad window
fn hide_scratchpad<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    manager: &mut Manager<H, C, SERVER>,
    scratchpad_window: &WindowHandle<H>,
) -> Result<(), &'static str> {
    tracing::trace!("Hide scratchpad window {:?}", scratchpad_window);
    let nsp_tag = manager
        .state
        .tags
        .get_hidden_by_label("NSP")
        .ok_or("Could not find NSP tag")?;
    let window = manager
        .state
        .windows
        .iter_mut()
        .find(|w| w.handle == *scratchpad_window)
        .ok_or("Could not find window from scratchpad_window")?;

    window.untag();
    // Hide the scratchpad.
    window.tag(&nsp_tag.id);
    window.set_visible(false);

    // Send tag changement to X
    let act = DisplayAction::SetWindowTag(*scratchpad_window, window.tag);
    manager.state.actions.push_back(act);
    manager.state.sort_windows();
    manager
        .state
        .handle_single_border(manager.config.border_width());

    // Will ignore current window handler because we just set it invisible
    let last_focused_still_visible = manager
        .state
        .focus_manager
        .window_history
        .iter()
        .find(|handle| {
            manager
                .state
                .windows
                .iter()
                .find(|window| Some(window.handle) == **handle)
                .map_or(false, Window::visible)
        })
        .copied();

    // Make sure when changing focus the lastly focused window is focused
    let handle = if let Some(prev) = last_focused_still_visible {
        prev
    } else if let Some(ws) = manager
        .state
        .focus_manager
        .workspace(&manager.state.workspaces)
    {
        manager
            .state
            .windows
            .iter()
            .find(|w| ws.is_managed(w))
            .map(|w| w.handle)
    } else {
        None
    };
    if let Some(handle) = handle {
        manager.state.handle_window_focus(&handle);
    }

    Ok(())
}

/// Makes a scratchpad window visible:
/// Expects that the window handle is a valid handle to an invisible scratchpad window
fn show_scratchpad<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    manager: &mut Manager<H, C, SERVER>,
    scratchpad_window: &WindowHandle<H>,
) -> Result<(), &'static str> {
    tracing::trace!("Show scratchpad window {:?}", scratchpad_window);
    let current_tag = &manager
        .state
        .focus_manager
        .tag(0)
        .ok_or("Could not retrieve the current tag")?;
    let window = manager
        .state
        .windows
        .iter_mut()
        .find(|w| w.handle == *scratchpad_window)
        .ok_or("Could not find window from scratchpad_window")?;
    let previous_tag = window.tag;
    window.untag();

    // Remove the entry for the previous tag to prevent the scratchpad being
    // refocused.
    if let Some(previous_tag) = previous_tag {
        manager
            .state
            .focus_manager
            .tags_last_window
            .remove(&previous_tag);
    }
    // Show the scratchpad.
    window.tag(current_tag);
    window.set_visible(true);

    // Send tag changement to X
    let act = DisplayAction::SetWindowTag(*scratchpad_window, window.tag);
    manager.state.actions.push_back(act);
    manager.state.sort_windows();
    manager
        .state
        .handle_single_border(manager.config.border_width());
    manager.state.handle_window_focus(scratchpad_window);
    manager.state.move_to_top(scratchpad_window);

    Ok(())
}

/// With the introduction of `VecDeque` for scratchpads, it is possible that a window gets destroyed
/// in the middle of the `VecDeque`. This is an abstraction to retrieve the next valid pid from a
/// scratchpad. While walking the scratchpad windows, invalid pids will get removed.
fn next_valid_scratchpad_pid<H: Handle>(
    scratchpad_windows: &mut VecDeque<u32>,
    managed_windows: &[Window<H>],
    direction: Direction,
) -> Option<u32> {
    while let Some(window) = if direction == Direction::Forward {
        scratchpad_windows.pop_front()
    } else {
        scratchpad_windows.pop_back()
    } {
        if managed_windows.iter().any(|w| w.pid == Some(window)) {
            if direction == Direction::Forward {
                scratchpad_windows.push_front(window);
            } else {
                scratchpad_windows.push_back(window);
            }
            return Some(window);
        }

        tracing::info!(
            "Dead window in scratchpad found, discard: window PID: {}",
            window
        );
    }

    None
}

/// Check if the scratchpad is visible on the current tag.
/// Returns `false` immediately if the scratchpad name isn't defined in the config
fn is_scratchpad_visible<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    manager: &Manager<H, C, SERVER>,
    scratchpad_name: &ScratchPadName,
) -> bool {
    // Like Try operator but returns false and only works on `Option`s
    macro_rules! try_bool {
        ($cond:expr) => {
            if let Some(value) = $cond {
                value
            } else {
                return false;
            }
        };
    }
    let current_tag = try_bool!(manager.state.focus_manager.tag(0));
    let scratchpad = try_bool!(manager.state.active_scratchpads.get(scratchpad_name));

    // Filter out all the non existing windows (invalid pid) and map to window
    // Check if any of them is in the current tag
    scratchpad
        .iter()
        .filter_map(|pid| manager.state.windows.iter().find(|w| w.pid == Some(*pid)))
        .any(|window| window.has_tag(&current_tag))
}

/// Handle the command to toggle the scratchpad
pub fn toggle_scratchpad<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    manager: &mut Manager<H, C, SERVER>,
    name: &ScratchPadName,
) -> Option<bool> {
    let current_tag = &manager.state.focus_manager.tag(0)?;
    let scratchpad = manager
        .state
        .scratchpads
        .iter()
        .find(|s| name == &s.name)?
        .clone();

    // Check if there is a valid scratchpad, if so handle it and return immediately
    if let Some(id) = manager.state.active_scratchpads.get_mut(&scratchpad.name) {
        if let Some(first_in_scratchpad) =
            next_valid_scratchpad_pid(id, &manager.state.windows, Direction::Forward)
        {
            if let Some((is_visible, window_handle)) = manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(first_in_scratchpad))
                .map(|w| (w.has_tag(current_tag), w.handle))
            {
                let action_result = if is_visible {
                    // Window is visible => Hide the scratchpad.
                    hide_scratchpad(manager, &window_handle)
                } else {
                    // Window is hidden => show the scratchpad
                    show_scratchpad(manager, &window_handle)
                };

                // Report the result of hiding/showing the scratchpad
                return match action_result {
                    Ok(()) => Some(true),
                    Err(msg) => {
                        tracing::error!("{}", msg);
                        return Some(false);
                    }
                };
            }
        }
    }

    tracing::debug!(
        "No active scratchpad found for name {:?}. Creating a new one",
        scratchpad.name
    );
    let name = scratchpad.name.clone();
    let pid: ChildID = exec_shell(&scratchpad.value, &mut manager.children)?;

    match manager.state.active_scratchpads.get_mut(&name) {
        Some(windows) => {
            windows.push_front(pid);
        }
        None => {
            manager
                .state
                .active_scratchpads
                .insert(name, VecDeque::from([pid]));
        }
    }

    None
}

/// Attaches the `WindowHandle` or the currently selected window to the selected `scratchpad`
pub fn attach_scratchpad<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    window: Option<WindowHandle<H>>,
    scratchpad: &ScratchPadName,
    manager: &mut Manager<H, C, SERVER>,
) -> Option<bool> {
    // If `None`, replace with current window
    let window_handle = {
        let current_window = manager
            .state
            .focus_manager
            .window_history
            .front()?
            .as_ref()
            .copied();

        window.or(current_window)?
    };

    // Retrieve and prepare window information
    let window_pid = {
        let ws = manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)?;
        let to_scratchpad = manager
            .state
            .scratchpads
            .iter()
            .find(|s| &s.name == scratchpad)?;
        let new_float_exact = to_scratchpad.xyhw(&ws.xyhw);

        let window = manager
            .state
            .windows
            .iter_mut()
            .find(|w| w.handle == window_handle)?;

        // Put window in correct position
        window.set_floating(true);
        window.normal = ws.xyhw;
        window.set_floating_exact(new_float_exact);
        tracing::debug!("Set window to floating: {:?}", window);

        window.pid?
    };

    if let Some(windows) = manager.state.active_scratchpads.get_mut(scratchpad) {
        tracing::debug!(
            "Scratchpad {:?} already active, push scratchpad",
            &scratchpad
        );
        let previous_scratchpad_handle = manager
            .state
            .windows
            .iter()
            .find(|w| w.pid.as_ref() == windows.front())
            .map(|w| w.handle);

        // Check if window already in scratchpad
        if windows.iter().any(|pid| *pid == window_pid) {
            return Some(false);
        }

        windows.push_front(window_pid);
        if let Some(previous_scratchpad_handle) = previous_scratchpad_handle {
            hide_scratchpad(manager, &previous_scratchpad_handle).ok()?; // first hide current scratchpad window
        }
    } else {
        tracing::debug!(
            "Scratchpad {:?} not active yet, open scratchpad",
            &scratchpad
        );
        manager
            .state
            .active_scratchpads
            .insert(scratchpad.clone(), VecDeque::from([window_pid]));
    }
    manager.state.sort_windows();

    Some(true)
}

/// Release a scratchpad to become a normal window. When tag is None, use current active tag as the
/// destination. Window can be a handle to select a specific window, the name of a scratchpad or
/// none to select the current window.
pub fn release_scratchpad<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    window: ReleaseScratchPadOption<H>,
    tag: Option<TagId>,
    manager: &mut Manager<H, C, SERVER>,
) -> Option<bool> {
    let destination_tag =
        tag.or_else(|| manager.state.focus_manager.tag_history.front().copied())?;

    // If `None`, replace with current window
    let window = if window == ReleaseScratchPadOption::None {
        ReleaseScratchPadOption::Handle(
            manager
                .state
                .focus_manager
                .window_history
                .front()?
                .as_ref()
                .copied()?,
        )
    } else {
        window
    };

    match window {
        ReleaseScratchPadOption::Handle(window_handle) => {
            // Check if window is in active scratchpad
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.handle == window_handle)?;

            let scratchpad_name: ScratchPadName = manager
                .state
                .active_scratchpads
                .iter_mut()
                .find(|(_, id)| window.pid.as_ref() == id.front())
                .map(|(name, _)| name.clone())?;

            tracing::debug!(
                "Releasing scratchpad {:?} to tag {}",
                scratchpad_name,
                destination_tag
            );

            // If we found window in scratchpad, remove it from active_scratchpads
            if let Some(windows) = manager.state.active_scratchpads.get_mut(&scratchpad_name) {
                if windows.len() > 1 {
                    // If more than 1, pop of the stack
                    tracing::debug!("Removed 1 window from scratchpad {:?}", &scratchpad_name);
                    windows.remove(
                        windows
                            .iter()
                            .position(|w| Some(w) == window.pid.as_ref())?,
                    );
                } else {
                    // If only 1, remove entire vec, not needed anymore
                    tracing::debug!(
                        "Empty scratchpad {:?}, removing from active_scratchpads",
                        &scratchpad_name
                    );
                    manager.state.active_scratchpads.remove(&scratchpad_name);
                }
            }

            Some(manager.command_handler(&Command::SendWindowToTag {
                window: Some(window_handle),
                tag: destination_tag,
            }))
        }
        ReleaseScratchPadOption::ScratchpadName(scratchpad_name) => {
            // Remove and get value from active_scratchpad
            let window_pid = manager
                .state
                .active_scratchpads
                .get_mut(&scratchpad_name)
                .and_then(|pids| {
                    next_valid_scratchpad_pid(pids, &manager.state.windows, Direction::Forward)
                })?;
            manager // We found already a working pid, discard from scratchpad
                .state
                .active_scratchpads
                .get_mut(&scratchpad_name)?
                .pop_front();

            let window_handle = manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(window_pid))
                .map(|w| w.handle);

            tracing::debug!(
                "Releasing scratchpad {:?} to tag {}",
                scratchpad_name,
                destination_tag
            );

            Some(manager.command_handler(&Command::SendWindowToTag {
                window: window_handle,
                tag: destination_tag,
            }))
        }
        ReleaseScratchPadOption::None => unreachable!(), // Should not be possible
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

/// Cycles the currently visible scratchpad window given the scratchpads name. Only visible
/// scratchpads will be handled, otherwise ignored
pub fn cycle_scratchpad_window<H: Handle, C: Config, SERVER: DisplayServer<H>>(
    manager: &mut Manager<H, C, SERVER>,
    scratchpad_name: &ScratchPadName,
    direction: Direction,
) -> Option<bool> {
    // Prevent cycles when scratchpad is not visible
    if !is_scratchpad_visible(manager, scratchpad_name) {
        return Some(false);
    }

    let scratchpad = manager.state.active_scratchpads.get_mut(scratchpad_name)?;
    // Get a handle to the currently visible window, so we can hide it later
    let visible_window_handle = manager
        .state
        .windows
        .iter()
        .find(|w| w.pid.as_ref() == scratchpad.front()) // scratchpad.front() ok because checked in is_scratchpad_visible
        .map(|w| w.handle);

    // Reorder the scratchpads
    // Clean scratchpad and exit if no next exists
    next_valid_scratchpad_pid(scratchpad, &manager.state.windows, direction)?;
    // Perform cycle
    match direction {
        Direction::Forward => scratchpad.rotate_left(1),
        Direction::Backward => scratchpad.rotate_right(1),
    };
    let new_window_pid = *scratchpad.front()?;

    // Hide the previous visible window
    if let Err(msg) = hide_scratchpad(manager, &visible_window_handle?) {
        tracing::error!("{}", msg);
        return Some(false);
    }

    // Show the new front window
    let new_window_handle = manager
        .state
        .windows
        .iter()
        .find(|w| w.pid == Some(new_window_pid))
        .map(|w| w.handle)?;
    if let Err(msg) = show_scratchpad(manager, &new_window_handle) {
        tracing::error!("{}", msg);
        return Some(false);
    }

    // Communicate changes to the rest of manager
    manager.state.sort_windows();

    Some(true)
}

#[cfg(test)]
mod tests {
    use crate::{config::ScratchPad, models::{ScratchPadName, MockHandle}};

    use super::*;

    #[test]
    fn show_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;
        let first_tag = manager.state.tags.get(1).unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::<MockHandle>(mock_window as i32);
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        // Make sure the window is on the first tag
        manager.command_handler(&Command::SendWindowToTag {
            window: None,
            tag: first_tag,
        });

        show_scratchpad(&mut manager, &window_handle).unwrap();

        let window = manager
            .state
            .windows
            .iter_mut()
            .find(|w| w.pid == Some(mock_window))
            .unwrap();

        assert!(
            !window.has_tag(&nsp_tag),
            "Scratchpad window is still in hidden NSP tag"
        );
        assert!(
            window.visible(),
            "Scratchpad window still is marked as invisible"
        );
    }

    #[test]
    fn hide_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;
        let first_tag = manager.state.tags.get(1).unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::<MockHandle>(mock_window as i32);
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        // Make sure the window is on the first tag
        manager.command_handler(&Command::SendWindowToTag {
            window: None,
            tag: first_tag,
        });

        hide_scratchpad(&mut manager, &window_handle).unwrap();

        let window = manager
            .state
            .windows
            .iter_mut()
            .find(|w| w.pid == Some(mock_window))
            .unwrap();

        assert!(
            window.has_tag(&nsp_tag),
            "Scratchpad window is not in hidden NSP tag"
        );
        assert!(
            !window.visible(),
            "Scratchpad window is not marked as invisible"
        );
    }

    #[test]
    fn toggle_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::<MockHandle>(mock_window as i32);
        let scratchpad_name: ScratchPadName = "Alacritty".into();
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: String::new(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name.clone(), VecDeque::from([mock_window]));

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name.clone()));

        // Assert window is hidden
        {
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(mock_window))
                .unwrap();

            assert!(
                window.has_tag(&nsp_tag),
                "Scratchpad window is not in hidden NSP tag"
            );
            assert!(!window.visible(), "Scratchpad is still marked as visible");
        }

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name));

        // Assert window is revealed
        {
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(mock_window))
                .unwrap();

            assert!(
                !window.has_tag(&nsp_tag),
                "Scratchpad window should not be in the hidden NSP tag"
            );
            assert!(
                window.visible(),
                "Scratchpad window is still marked as invisible"
            );
        }
    }

    #[test]
    /// Test release scratchpad command for 1 window in the scratchpad
    /// After releasing, the scratchpad should not be active anymore (no more windows)
    fn release_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());

        // Setup
        let mock_window1 = 10_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name.clone(), VecDeque::from([mock_window1]));
        manager.window_created_handler(
            Window::new(
                WindowHandle::<MockHandle>(mock_window1 as i32),
                None,
                Some(mock_window1),
            ),
            -1,
            -1,
        );

        let expected_tag = manager.state.tags.get(1).unwrap().id;

        // Release Scratchpad
        manager.command_handler(&Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::Handle(WindowHandle::<MockHandle>(mock_window1 as i32)),
            tag: Some(expected_tag),
        });

        // Assert
        assert!(manager
            .state
            .active_scratchpads
            .get(&scratchpad_name)
            .is_none());
        assert_eq!(
            *manager.state.focus_manager.tag_history.front().unwrap(),
            expected_tag
        );
    }

    #[test]
    /// Testing release scratchpad command with more than 1 window in a scratchpad
    /// After releasing 1 window, the rest should still be in the scratchpad
    fn release_scratchpad_multiple_windows_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();
        manager.state.active_scratchpads.insert(
            scratchpad_name.clone(),
            VecDeque::from([mock_window1, mock_window2, mock_window3]),
        );
        for window in [mock_window1, mock_window2, mock_window3] {
            manager.window_created_handler(
                Window::new(WindowHandle::<MockHandle>(window as i32), None, Some(window)),
                -1,
                -1,
            );
        }

        let expected_tag = manager.state.tags.get(1).unwrap().id;

        // Release Scratchpad
        manager.command_handler(&Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::Handle(WindowHandle::<MockHandle>(mock_window1 as i32)),
            tag: Some(expected_tag),
        });

        // Assert
        let scratchpad = manager
            .state
            .active_scratchpads
            .get_mut(&scratchpad_name)
            .unwrap();

        assert!(manager
            .state
            .windows
            .iter()
            .find(|w| w.pid == Some(mock_window1))
            .map(|w| !w.has_tag(&nsp_tag))
            .unwrap());
        for mock_window_pid in [mock_window2, mock_window3] {
            let window_pid = scratchpad.pop_front();
            assert_eq!(window_pid, Some(mock_window_pid));
            assert!(!manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == window_pid)
                .map(|w| w.has_tag(&nsp_tag))
                .unwrap());
        }
        assert_eq!(scratchpad.pop_front(), None);

        assert_eq!(
            *manager.state.focus_manager.tag_history.front().unwrap(),
            expected_tag
        );
    }

    #[test]
    fn attach_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager.state.active_scratchpads.insert(
            scratchpad_name.clone(),
            VecDeque::from([mock_window2, mock_window3]),
        );
        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::<MockHandle>(mock_window as i32),
                None,
                Some(mock_window),
            );
            if mock_window != mock_window1 {
                window.tag(&nsp_tag);
            }

            manager.window_created_handler(window, -1, -1);
        }

        // Attach Scratchpad
        manager.command_handler(&Command::AttachScratchPad {
            window: Some(WindowHandle::<MockHandle>(mock_window1 as i32)),
            scratchpad: scratchpad_name.clone(),
        });

        // Assert
        let scratchpad = manager
            .state
            .active_scratchpads
            .get_mut(&scratchpad_name)
            .unwrap();

        assert_eq!(scratchpad.pop_front(), Some(mock_window1));
        assert!(manager
            .state
            .windows
            .iter()
            .find(|w| w.pid == Some(mock_window1))
            .map(|w| !w.has_tag(&nsp_tag))
            .unwrap());
        for mock_window_pid in [mock_window2, mock_window3] {
            let window_pid = scratchpad.pop_front();
            assert_eq!(window_pid, Some(mock_window_pid));
            assert!(manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == window_pid)
                .map(|w| w.has_tag(&nsp_tag))
                .unwrap());
        }
        assert_eq!(scratchpad.pop_front(), None);
    }

    #[test]
    fn next_valid_pid_forward_test() {
        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let mock_window4 = 4_u32;

        let mut managed_windows = [mock_window1, mock_window2, mock_window3, mock_window4]
            .iter()
            .map(|pid| Window::new(WindowHandle::<MockHandle>(*pid as i32), None, Some(*pid)))
            .collect::<Vec<Window>>();
        let mut scratchpad =
            VecDeque::from([mock_window1, mock_window2, mock_window3, mock_window4]);

        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Forward),
            Some(1)
        );

        managed_windows.remove(1);
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Forward),
            Some(1)
        );

        scratchpad.pop_front();
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Forward),
            Some(3)
        );
        assert_eq!(scratchpad.len(), 2);
    }

    #[test]
    fn next_valid_pid_backward_test() {
        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let mock_window4 = 4_u32;

        let mut managed_windows = [mock_window1, mock_window2, mock_window3, mock_window4]
            .iter()
            .map(|pid| Window::new(WindowHandle::<MockHandle>(*pid as i32), None, Some(*pid)))
            .collect::<Vec<Window>>();
        let mut scratchpad =
            VecDeque::from([mock_window1, mock_window2, mock_window3, mock_window4]);

        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Backward),
            Some(4)
        );

        managed_windows.remove(2);
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Backward),
            Some(4)
        );

        scratchpad.pop_back();
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows, Direction::Backward),
            Some(2)
        );
        assert_eq!(scratchpad.len(), 2);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn cycle_scratchpad_window_test() {
        fn is_visible<H: Handle, C: Config, SERVER: DisplayServer<H>>(
            manager: &Manager<H, C, SERVER>,
            pid: u32,
            nsp_tag: TagId,
        ) -> bool {
            manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(pid))
                .map(|w| w.visible() && !w.has_tag(&nsp_tag))
                .unwrap()
        }
        fn is_only_first_visible<H: Handle, C: Config, SERVER: DisplayServer<H>>(
            manager: &Manager<H, C, SERVER>,
            mut pids: impl Iterator<Item = u32>,
            nsp_tag: TagId,
        ) -> bool {
            if !is_visible(manager, pids.next().unwrap(), nsp_tag) {
                return false;
            }
            for pid in pids {
                if is_visible(manager, pid, nsp_tag) {
                    return false;
                }
            }

            true
        }

        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();

        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::<MockHandle>(mock_window as i32),
                None,
                Some(mock_window),
            );
            if mock_window != mock_window1 {
                window.tag(&nsp_tag);
            }

            manager.window_created_handler(window, -1, -1);
        }
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager.state.active_scratchpads.insert(
            scratchpad_name.clone(),
            VecDeque::from([mock_window1, mock_window2, mock_window3]),
        );

        cycle_scratchpad_window(&mut manager, &scratchpad_name, Direction::Forward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(&scratchpad_name)
            .unwrap()
            .iter();
        assert!(
            is_only_first_visible(&manager, scratchpad_iterator.clone().copied(), nsp_tag),
            "On the first forward cycle, the first window is not visible or the other windows are visible"
        );
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, &scratchpad_name, Direction::Forward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(&scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ),
            "On the second forward cycle, the first window is not visible or the other windows are visible"
        );
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, &scratchpad_name, Direction::Backward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(&scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ),
            "After 2 forward and 1 backward cycles, the first window is not visible or the other windows are visible"
        );
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, &scratchpad_name, Direction::Backward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(&scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ),
            "After 2 forward and 2 backward cycles, the first window is not visible or the other windows are visible"
        );
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), None);
    }

    #[test]
    fn change_focus_with_open_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());

        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();

        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::<MockHandle>(mock_window as i32),
                None,
                Some(mock_window),
            );
            window.set_visible(true);
            window.tag(&1);

            manager.window_created_handler(window, -1, -1);
        }
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name, VecDeque::from([mock_window3]));

        // Focus first window
        let focus_window_handler = manager.state.windows[0].handle;
        manager.state.handle_window_focus(&focus_window_handler);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(1),
            "Initially the first window (1) should be focused"
        );

        manager.command_handler(&Command::FocusWindowDown);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(2),
            "After 1 down window (2) should be focused"
        );

        manager.command_handler(&Command::FocusWindowDown);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(3),
            "After 2 down window (3) should be focused"
        );

        manager.command_handler(&Command::FocusWindowDown);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(1),
            "After 3 down window (1) should be focused (cycle back)"
        );

        manager.command_handler(&Command::FocusWindowUp);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(3),
            "After 3 down and 1 up window (3) should be focused (cycle back)"
        );

        manager.command_handler(&Command::FocusWindowUp);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(2),
            "After 3 down and 2 up window (2) should be focused"
        );
    }

    #[test]
    fn focus_top_from_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());

        // Setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name: ScratchPadName = "Alacritty".into();

        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::<MockHandle>(mock_window as i32),
                None,
                Some(mock_window),
            );
            window.set_visible(true);
            window.tag(&1);

            manager.window_created_handler(window, -1, -1);
        }
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name, VecDeque::from([mock_window3]));

        // Focus first window
        let focus_window_handler = manager.state.windows[0].handle;
        manager.state.handle_window_focus(&focus_window_handler);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(1),
            "Initially the first window (1) should be focused"
        );

        manager.command_handler(&Command::FocusWindowUp);
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(3),
            "After 1 up window (3) should be focused (scratchpad window)"
        );

        manager.command_handler(&Command::FocusWindowTop { swap: false });
        assert_eq!(
            manager
                .state
                .focus_manager
                .window(&manager.state.windows)
                .unwrap()
                .handle,
            WindowHandle::<MockHandle>(1),
            "After focusing the scratchpad and then focusing the top, window (1) should be focused"
        );
    }

    #[test]
    fn toggle_scratchpad_also_toggles_single_window_borders() {
        let mut manager = Manager::new_test_with_border(vec!["1".to_string(), "2".to_string()], 1);
        manager.screen_create_handler(Default::default());
        let second_tag = manager.state.tags.get(2).unwrap().id;

        let scratchpad_pid = 1_u32;
        let scratchpad_handle = WindowHandle::<MockHandle>(scratchpad_pid as i32);
        let scratchpad_name: ScratchPadName = "Alacritty".into();
        let mut scratchpad = Window::new(scratchpad_handle, None, Some(scratchpad_pid));
        scratchpad.tag = Some(second_tag);
        manager.window_created_handler(scratchpad, -1, -1);
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.clone(),
            value: String::new(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name.clone(), VecDeque::from([scratchpad_pid]));

        let window_pid = 2_u32;
        let window_handle = WindowHandle::<MockHandle>(window_pid as i32);
        manager.window_created_handler(Window::new(window_handle, None, Some(window_pid)), -1, -1);

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name.clone()));

        {
            let scratchpad = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(scratchpad_pid))
                .unwrap();
            assert_eq!(scratchpad.border(), 1);

            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(window_pid))
                .unwrap();
            assert_eq!(window.border(), 1);
        }

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name));

        {
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(window_pid))
                .unwrap();

            assert_eq!(window.border(), 0);
        }
    }
}
