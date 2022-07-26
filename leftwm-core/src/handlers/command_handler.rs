#![allow(clippy::wildcard_imports)]
#![allow(clippy::shadow_unrelated)]

// NOTE: there apears to be a clippy bug with shadow_unrelated and the (?) Operator
// allow shadow should be removed once it is resolved
// https://github.com/rust-lang/rust-clippy/issues/6563

use std::collections::VecDeque;

use super::window_handler::scratchpad_xyhw;
use super::*;
use crate::child_process::Children;
use crate::command::ReleaseScratchPadOption;
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
use crate::models::{TagId, WindowState};
use crate::state::State;
use crate::utils::helpers::relative_find;
use crate::utils::{child_process::exec_shell, helpers};
use crate::{config::Config, models::FocusBehaviour};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /* When adding a command
     * please update src/utils/command_pipe and leftwm/src/command if:
     * - a command is introduced or renamed
     * please also update src/bin/leftwm-check if any of the following apply after your update:
     * - a command now requires a value
     * - a command no longer requires a value
     * - a new command is introduced that requires a value
     *  */
    /// Processes a command and invokes the associated function.
    pub fn command_handler(&mut self, command: &Command) -> bool {
        process_internal(self, command).unwrap_or(false)
    }
}

macro_rules! move_focus_common_vars {
    ($func:ident ($state:expr $(, $arg:expr )* $(,)? )) => {{
        let handle = $state.focus_manager.window(&$state.windows)?.handle;
        let tag_id = $state.focus_manager.tag(0)?;
        let tag = $state.tags.get(tag_id)?;
        let (tags, layout) = (vec![tag_id], Some(tag.layout));

        let for_active_workspace =
            |x: &Window| -> bool { helpers::intersect(&tags, &x.tags) && !x.is_unmanaged() };

        let to_reorder = helpers::vec_extract(&mut $state.windows, for_active_workspace);
        $func($state, handle, layout, to_reorder, $($arg),*)
    }};
}

fn process_internal<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    command: &Command,
) -> Option<bool> {
    let state = &mut manager.state;
    match command {
        Command::Execute(shell_command) => execute(&mut manager.children, shell_command),

        Command::ToggleScratchPad(name) => toggle_scratchpad(manager, name),
        Command::AttachScratchPad { window, scratchpad } => {
            attach_scratchpad(*window, scratchpad.clone(), manager)
        }
        Command::ReleaseScratchPad { window, tag } => {
            release_scratchpad(window.clone(), *tag, manager)
        }

        Command::NextScratchPadWindow { scratchpad } => {
            cycle_scratchpad_window(manager, scratchpad.as_str(), Direction::Forward)
        }
        Command::PrevScratchPadWindow { scratchpad } => {
            cycle_scratchpad_window(manager, scratchpad.as_str(), Direction::Backward)
        }

        Command::ToggleFullScreen => toggle_state(state, WindowState::Fullscreen),
        Command::ToggleSticky => toggle_state(state, WindowState::Sticky),

        Command::SendWindowToTag { window, tag } => move_to_tag(*window, *tag, manager),
        Command::MoveWindowToNextTag { follow } => move_to_tag_relative(manager, *follow, 1),
        Command::MoveWindowToPreviousTag { follow } => move_to_tag_relative(manager, *follow, -1),
        Command::MoveWindowToLastWorkspace => move_to_last_workspace(state),
        Command::MoveWindowToNextWorkspace => move_window_to_workspace_change(manager, 1),
        Command::MoveWindowToPreviousWorkspace => move_window_to_workspace_change(manager, -1),
        Command::MoveWindowUp => move_focus_common_vars!(move_window_change(state, -1)),
        Command::MoveWindowDown => move_focus_common_vars!(move_window_change(state, 1)),
        Command::MoveWindowTop { swap } => move_focus_common_vars!(move_window_top(state, *swap)),

        Command::GoToTag { tag, swap } => goto_tag(state, *tag, *swap),
        Command::ReturnToLastTag => return_to_last_tag(state),

        Command::CloseWindow => close_window(state),
        Command::SwapScreens => swap_tags(state),
        Command::NextLayout => next_layout(state),
        Command::PreviousLayout => previous_layout(state),

        Command::SetLayout(layout) => set_layout(*layout, state),

        Command::FloatingToTile => floating_to_tile(state),
        Command::TileToFloating => tile_to_floating(state),
        Command::ToggleFloating => toggle_floating(state),

        Command::FocusNextTag => focus_tag_change(state, 1),
        Command::FocusPreviousTag => focus_tag_change(state, -1),
        Command::FocusWindow(param) => focus_window(state, param),
        Command::FocusWindowUp => move_focus_common_vars!(focus_window_change(state, -1)),
        Command::FocusWindowDown => move_focus_common_vars!(focus_window_change(state, 1)),
        Command::FocusWindowTop { swap } => focus_window_top(state, *swap),
        Command::FocusWorkspaceNext => focus_workspace_change(state, 1),
        Command::FocusWorkspacePrevious => focus_workspace_change(state, -1),

        Command::MouseMoveWindow => None,

        Command::SoftReload => {
            // Make sure the currently focused window is saved for the tag.
            if let Some((handle, tag)) = state
                .focus_manager
                .window(&state.windows)
                .map(|w| (w.handle, w.tags[0]))
            {
                let old_handle = state
                    .focus_manager
                    .tags_last_window
                    .entry(tag)
                    .or_insert(handle);
                *old_handle = handle;
            }
            manager.config.save_state(&manager.state);
            manager.hard_reload();
            None
        }
        Command::HardReload => {
            manager.hard_reload();
            None
        }

        Command::RotateTag => rotate_tag(state),

        Command::IncreaseMainWidth(delta) => change_main_width(state, *delta, 1),
        Command::DecreaseMainWidth(delta) => change_main_width(state, *delta, -1),
        Command::SetMarginMultiplier(multiplier) => set_margin_multiplier(state, *multiplier),
        Command::SendWorkspaceToTag(ws_index, tag_index) => {
            Some(send_workspace_to_tag(state, *ws_index, *tag_index))
        }
        Command::CloseAllOtherWindows => close_all_other_windows(state),
        Command::Other(cmd) => Some(C::command_handler(cmd, manager)),
    }
}

fn execute(children: &mut Children, shell_command: &str) -> Option<bool> {
    let _ = exec_shell(shell_command, children);
    None
}

/// Hide scratchpad window:
/// Expects that the window handle is a valid handle to a visible scratchpad window
fn hide_scratchpad<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    scratchpad_window: &WindowHandle,
) -> Result<(), &'static str> {
    log::trace!("Hide scratchpad window {:?}", scratchpad_window);
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

    window.clear_tags();
    // Hide the scratchpad.
    window.tag(&nsp_tag.id);

    // send tag changement to X
    let act = DisplayAction::SetWindowTags(*scratchpad_window, window.tags.clone());
    manager.state.actions.push_back(act);
    manager.state.sort_windows();

    // Make sure when changing focus the scratchpad is currently focused.
    let handle =
        if Some(&Some(*scratchpad_window)) != manager.state.focus_manager.window_history.get(0) {
            None
        } else if let Some(Some(prev)) = manager.state.focus_manager.window_history.get(1) {
            Some(*prev)
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

fn show_scratchpad<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    scratchpad_window: &WindowHandle,
) -> Result<(), &'static str> {
    log::trace!("Show scratchpad window {:?}", scratchpad_window);
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
    let previous_tag = window.tags[0];
    window.clear_tags();

    // Remove the entry for the previous tag to prevent the scratchpad being
    // refocused.
    manager
        .state
        .focus_manager
        .tags_last_window
        .remove(&previous_tag);
    // Show the scratchpad.
    window.tag(current_tag);

    // send tag changement to X
    let act = DisplayAction::SetWindowTags(*scratchpad_window, window.tags.clone());
    manager.state.actions.push_back(act);
    manager.state.sort_windows();
    manager.state.handle_window_focus(scratchpad_window);
    manager.state.move_to_top(scratchpad_window);

    Ok(())
}

/// With the introduction of `VecDeque` for scratchpads, it is possible that a window gets destroyed
/// in the middle of the `VecDeque`. This is an abstraction to retrieve the next valid pid from a
/// scratchpad. While walking the scratchpad windows, invalid pids will get removed.
fn next_valid_scratchpad_pid(
    scratchpad_windows: &mut VecDeque<u32>,
    managed_windows: &[Window],
) -> Option<u32> {
    while let Some(window) = scratchpad_windows.pop_front() {
        if managed_windows.iter().any(|w| w.pid == Some(window)) {
            scratchpad_windows.push_front(window);
            return Some(window);
        }

        log::info!(
            "Dead window in scratchpad found, discard: window PID: {}",
            window
        );
    }

    None
}

fn prev_valid_scratchpad_pid(
    scratchpad_windows: &mut VecDeque<u32>,
    managed_windows: &[Window],
) -> Option<u32> {
    while let Some(window) = scratchpad_windows.pop_back() {
        if managed_windows.iter().any(|w| w.pid == Some(window)) {
            scratchpad_windows.push_back(window);
            return Some(window);
        }

        log::info!(
            "Dead window in scratchpad found, discard: window PID: {}",
            window
        );
    }

    None
}

fn is_scratchpad_visible<C: Config, SERVER: DisplayServer>(
    manager: &Manager<C, SERVER>,
    scratchpad: &str,
) -> bool {
    let current_tag = if let Some(tag) = manager.state.focus_manager.tag(0) {
        tag
    } else {
        return false;
    };

    // Can be seen as a pipeline where each failure should short circuit the next of the steps,
    // clippy thinks this is redundant, but when first_pid is None and is directly compared with a pid
    // from a window with no pid, it will match while it most certainly is not what we are looking
    // for.
    #[allow(clippy::redundant_closure_for_method_calls)]
    manager
        .state
        .active_scratchpads
        .get(scratchpad)
        .and_then(|pids| pids.front())
        .and_then(|first_pid| {
            manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(*first_pid))
        })
        .map(|window| window.has_tag(&current_tag))
        .is_some()
}

fn toggle_scratchpad<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    name: &str,
) -> Option<bool> {
    let current_tag = &manager.state.focus_manager.tag(0)?;
    let scratchpad = manager
        .state
        .scratchpads
        .iter()
        .find(|s| name == s.name)?
        .clone();

    if let Some(id) = manager.state.active_scratchpads.get_mut(&scratchpad.name) {
        if let Some(first_in_scratchpad) =
            dbg!(next_valid_scratchpad_pid(id, &manager.state.windows))
        {
            if let Some((is_visible, window_handle)) = manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(first_in_scratchpad))
                .map(|w| (w.has_tag(current_tag), w.handle))
            {
                if dbg!(is_visible) {
                    // window is visible => Hide the scratchpad.
                    if let Err(msg) = hide_scratchpad(manager, &window_handle) {
                        log::error!("{}", msg);
                        return Some(false);
                    }
                } else {
                    // window is hidden => show the scratchpad
                    if let Err(msg) = show_scratchpad(manager, &window_handle) {
                        log::error!("{}", msg);
                        return Some(false);
                    }
                }

                return Some(true);
            }
        }
    }

    log::debug!(
        "no active scratchpad found for name {:?}. creating a new one",
        scratchpad.name
    );
    let name = scratchpad.name.clone();
    let pid = exec_shell(&scratchpad.value, &mut manager.children)?;
    //manager.state.active_scratchpads.insert(name, pid);
    match manager.state.active_scratchpads.get_mut(&scratchpad.value) {
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
fn attach_scratchpad<C: Config, SERVER: DisplayServer>(
    window: Option<WindowHandle>,
    scratchpad: String,
    manager: &mut Manager<C, SERVER>,
) -> Option<bool> {
    // if None, replace with current window
    let window_handle = window.or(manager
        .state
        .focus_manager
        .window_history
        .get(0)?
        .as_ref()
        .copied())?;

    // retrieve and prepare window information
    let window_pid = {
        let ws = manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)?;
        let to_scratchpad = manager
            .state
            .scratchpads
            .iter()
            .find(|s| s.name == scratchpad)?;
        let new_float_exact = scratchpad_xyhw(&ws.xyhw, to_scratchpad);

        let window = manager
            .state
            .windows
            .iter_mut()
            .find(|w| w.handle == window_handle)?;

        // Put window in correct position
        window.set_floating(true);
        window.normal = ws.xyhw;
        window.set_floating_exact(new_float_exact);
        log::debug!("Set window to floating: {:?}", window);

        window.pid?
    };

    if let Some(windows) = manager.state.active_scratchpads.get_mut(&scratchpad) {
        log::debug!("Scratchpad {} already active, push scratchpad", &scratchpad);
        let previous_scratchpad_handle = manager
            .state
            .windows
            .iter()
            .find(|w| w.pid.as_ref() == windows.front())
            .map(|w| w.handle);

        // check if window already in scratchpad
        if windows.iter().any(|pid| *pid == window_pid) {
            return Some(false);
        }

        windows.push_front(window_pid);
        if let Some(previous_scratchpad_handle) = previous_scratchpad_handle {
            hide_scratchpad(manager, &previous_scratchpad_handle).ok()?; // first hide current scratchpad window
        }
    } else {
        log::debug!("Scratchpad {} not active yet, open scratchpad", &scratchpad);
        manager
            .state
            .active_scratchpads
            .insert(scratchpad, VecDeque::from([window_pid]));
    }
    manager.state.sort_windows();

    Some(true)
}

/// Release a scratchpad to become a normal window. When tag is None, use current active tag as the
/// destination. Window can be a handle to select a specific window, the name of a scratchpad or
/// none to select the current window.
fn release_scratchpad<C: Config, SERVER: DisplayServer>(
    window: ReleaseScratchPadOption,
    tag: Option<TagId>,
    manager: &mut Manager<C, SERVER>,
) -> Option<bool> {
    let destination_tag =
        tag.or_else(|| manager.state.focus_manager.tag_history.get(0).copied())?;

    // if None, replace with current window
    let window = if window == ReleaseScratchPadOption::None {
        ReleaseScratchPadOption::Handle(
            manager
                .state
                .focus_manager
                .window_history
                .get(0)?
                .as_ref()
                .copied()?,
        )
    } else {
        window
    };

    match window {
        ReleaseScratchPadOption::Handle(window_handle) => {
            // check if window is in active scratchpad
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.handle == window_handle)?;

            let scratchpad_name = manager
                .state
                .active_scratchpads
                .iter_mut()
                .find(|(_, id)| window.pid.as_ref() == id.front())
                .map(|(name, _)| name.clone())?;

            log::debug!(
                "Releasing scratchpad {} to tag {}",
                scratchpad_name,
                destination_tag
            );

            // if we found window in scratchpad, remove it from active_scratchpads
            if let Some(windows) = manager.state.active_scratchpads.get_mut(&scratchpad_name) {
                if windows.len() > 1 {
                    // if more than 1, pop of the stack
                    log::debug!("Removed 1 window from scratchpad {}", &scratchpad_name);
                    windows.remove(
                        windows
                            .iter()
                            .position(|w| Some(w) == window.pid.as_ref())?,
                    );
                } else {
                    // if only 1, remove entire vec, not needed anymore
                    log::debug!(
                        "Empty scratchpad {}, removing from active_scratchpads",
                        &scratchpad_name
                    );
                    manager.state.active_scratchpads.remove(&scratchpad_name);
                }
            }

            move_to_tag(Some(window_handle), destination_tag, manager)
        }
        ReleaseScratchPadOption::ScrathpadName(scratchpad_name) => {
            // remove and get value from active_scratchpad
            let window_pid = manager
                .state
                .active_scratchpads
                .get_mut(&scratchpad_name)
                .and_then(|pids| next_valid_scratchpad_pid(pids, &manager.state.windows))?;
            manager // we found already a working pid, discard from scratchpad
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

            log::debug!(
                "Releasing scratchpad {} to tag {}",
                scratchpad_name,
                destination_tag
            );

            move_to_tag(window_handle, destination_tag, manager)
        }
        ReleaseScratchPadOption::None => unreachable!(), // should not be possible
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forward,
    Backward,
}

fn cycle_scratchpad_window<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    scratchpad_name: &str,
    direction: Direction,
) -> Option<bool> {
    // prevent cycles when scratchpad is not visible
    if !is_scratchpad_visible(manager, scratchpad_name) {
        return Some(false);
    }

    let scratchpad = dbg!(manager.state.active_scratchpads.get_mut(scratchpad_name)?);
    // get a handle to the currently visible window, so we can hide it later
    let visible_window_handle = manager
        .state
        .windows
        .iter()
        .find(|w| w.pid.as_ref() == scratchpad.front()) // scratchpad.front() ok because checked in is_scratchpad_visible
        .map(|w| w.handle);

    // reorder the scratchpads
    match direction {
        Direction::Forward => {
            next_valid_scratchpad_pid(scratchpad, &manager.state.windows)?;
            let front = scratchpad.pop_front()?;
            scratchpad.push_back(front);
        }
        Direction::Backward => {
            prev_valid_scratchpad_pid(scratchpad, &manager.state.windows)?;
            let back = scratchpad.pop_back()?;
            scratchpad.push_front(back);
        }
    };
    let new_window_pid = *scratchpad.front()?;

    // hide the previous visible window
    if let Err(msg) = hide_scratchpad(manager, &visible_window_handle?) {
        log::error!("{}", msg);
        return Some(false);
    }

    // show the new front window
    let new_window_handle = manager
        .state
        .windows
        .iter()
        .find(|w| w.pid == Some(new_window_pid))
        .map(|w| w.handle)?;
    if let Err(msg) = show_scratchpad(manager, &new_window_handle) {
        log::error!("{}", msg);
        return Some(false);
    }

    // communicate changes to the rest of manager
    manager.state.sort_windows();

    Some(true)
}

fn toggle_state(state: &mut State, window_state: WindowState) -> Option<bool> {
    let window = state.focus_manager.window(&state.windows)?;
    let handle = window.handle;
    let toggle_to = !window.has_state(&window_state);
    let act = DisplayAction::SetState(handle, toggle_to, window_state);
    state.actions.push_back(act);
    state.handle_window_focus(&handle);
    match window_state {
        WindowState::Fullscreen => Some(true),
        _ => Some(false),
    }
}

fn move_to_tag<C: Config, SERVER: DisplayServer>(
    window: Option<WindowHandle>,
    tag_num: TagId,
    manager: &mut Manager<C, SERVER>,
) -> Option<bool> {
    let tag = manager.state.tags.get(tag_num)?.clone();

    // In order to apply the correct margin multiplier we want to copy this value
    // from any window already present on the target tag
    let margin_multiplier = match manager.state.windows.iter().find(|w| w.has_tag(&tag.id)) {
        Some(w) => w.margin_multiplier(),
        None => 1.0,
    };

    let handle = window.or(*manager.state.focus_manager.window_history.get(0)?)?;
    // Only handle the focus when moving the focused window.
    let handle_focus = window.is_none();
    // Focus the next or previous window on the workspace.
    let new_handle = if handle_focus {
        manager.get_next_or_previous(&handle)
    } else {
        None
    };

    let window = manager
        .state
        .windows
        .iter_mut()
        .find(|w| w.handle == handle)?;

    window.clear_tags();
    window.set_floating(false);
    window.tag(&tag.id);
    window.apply_margin_multiplier(margin_multiplier);
    let act = DisplayAction::SetWindowTags(window.handle, vec![tag.id]);
    manager.state.actions.push_back(act);

    manager.state.sort_windows();
    if handle_focus {
        if let Some(new_handle) = new_handle {
            manager.state.focus_window(&new_handle);
        } else {
            let act = DisplayAction::Unfocus(Some(handle), false);
            manager.state.actions.push_back(act);
            manager.state.focus_manager.window_history.push_front(None);
        }
    }
    Some(true)
}

/// Move currently focused window to tag relative to current tag
///
/// Conditionally allow focus to follow the window to the target tag
fn move_to_tag_relative<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    follow: bool,
    delta: i32,
) -> Option<bool> {
    // Map indexing from 1..len to 0..(len - 1)
    let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default() - 1;
    // apply euclidean division reminder to the result of offseting to wrap around tags vector
    // and add 1 to remap back to 1..len indexing
    let tags_len = manager.state.tags.normal().len() as isize;
    let desired_tag = (current_tag as isize + delta as isize).rem_euclid(tags_len) + 1;
    let desired_tag = desired_tag as usize;

    move_to_tag(None, desired_tag, manager);
    if follow {
        let moved_window = *manager.state.focus_manager.window_history.get(1)?;
        manager.state.goto_tag_handler(desired_tag);
        manager.state.handle_window_focus(&moved_window?);
    }
    Some(true)
}

fn move_window_to_workspace_change<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    delta: i32,
) -> Option<bool> {
    let current = manager
        .state
        .focus_manager
        .workspace(&manager.state.workspaces)?;
    let workspace =
        helpers::relative_find(&manager.state.workspaces, |w| w == current, delta, true)?.clone();

    let tag_num = workspace.tags.first()?;
    move_to_tag(None, *tag_num, manager)
}

fn goto_tag(state: &mut State, input_tag: TagId, current_tag_swap: bool) -> Option<bool> {
    let current_tag = state.focus_manager.tag(0).unwrap_or_default();
    let previous_tag = state.focus_manager.tag(1).unwrap_or_default();
    let destination_tag = if current_tag_swap && current_tag == input_tag {
        previous_tag
    } else {
        input_tag
    };
    state.goto_tag_handler(destination_tag)
}

fn return_to_last_tag(state: &mut State) -> Option<bool> {
    let previous_tag = state.focus_manager.tag(1).unwrap_or_default();
    state.goto_tag_handler(previous_tag)
}

fn focus_window(state: &mut State, param: &str) -> Option<bool> {
    match param.parse::<usize>() {
        Ok(index) if index > 0 => {
            //1-based index seems more user-friendly to me in this context
            let handle = state
                .windows
                .iter()
                .filter(|w| w.visible())
                .nth(index - 1)?
                .handle;

            state.handle_window_focus(&handle);
            None
        }
        Err(_) => focus_window_by_class(state, param),
        Ok(_) => None,
    }
}

fn focus_window_by_class(state: &mut State, window_class: &str) -> Option<bool> {
    let is_target = |w: &Window| -> bool {
        w.res_name
            .as_ref()
            .zip(w.res_class.as_ref())
            .map_or(false, |(res_name, res_class)| {
                window_class == res_name || window_class == res_class
            })
    };

    let current_window = state.focus_manager.window(&state.windows)?;
    let target_window = if is_target(current_window) {
        let previous_window_handle = state.focus_manager.window_history.get(1);
        state
            .windows
            .iter()
            .find(|w| Some(&Some(w.handle)) == previous_window_handle)
            .cloned()
    } else {
        state.windows.iter().find(|w| is_target(*w)).cloned()
    }?;

    let handle = target_window.handle;

    if target_window.visible() {
        state.handle_window_focus(&handle);
        return None;
    }

    let tag_id = target_window.tags.first()?;
    state.goto_tag_handler(*tag_id)?;

    match state
        .focus_manager
        .workspace(&state.workspaces)
        .map(|ws| ws.layout)
    {
        Some(layout) if layout == Layout::Monocle || layout == Layout::MainAndDeck => {
            let mut windows = helpers::vec_extract(&mut state.windows, |w| {
                w.has_tag(tag_id) && !w.is_unmanaged() && !w.floating()
            });

            let cycle = |wins: &mut Vec<Window>, s: &mut State| {
                let window_index = wins.iter().position(|w| w.handle == handle).unwrap_or(0);
                let _ = helpers::cycle_vec(wins, -(window_index as i32));
                s.windows.append(wins);
            };

            if layout == Layout::Monocle && windows.len() > 1 {
                cycle(&mut windows, state);
            } else if layout == Layout::MainAndDeck && windows.len() > 2 {
                let main_window = windows.remove(0);
                state.windows.push(main_window);
                cycle(&mut windows, state);
            } else {
                state.windows.append(&mut windows);
            }

            state.handle_window_focus(&handle);
            Some(true)
        }
        Some(_) => {
            state.handle_window_focus(&handle);
            Some(true)
        }
        None => None,
    }
}

/// Focus the adjacent tags, depending on the delta.
/// A delta of 1 means "next tag", a delta of -1 means "previous tag".
fn focus_tag_change(state: &mut State, delta: i8) -> Option<bool> {
    let current_tag = state.focus_manager.tag(0)?;
    let tags = state.tags.normal();
    let relative_tag_id = relative_find(tags, |tag| tag.id == current_tag, i32::from(delta), true)
        .map(|tag| tag.id)?;
    state.goto_tag_handler(relative_tag_id)
}

fn swap_tags(state: &mut State) -> Option<bool> {
    if state.workspaces.len() >= 2 && state.focus_manager.workspace_history.len() >= 2 {
        let hist_a = *state.focus_manager.workspace_history.get(0)?;
        let hist_b = *state.focus_manager.workspace_history.get(1)?;
        //Update workspace tags
        let mut temp = vec![];
        std::mem::swap(&mut state.workspaces.get_mut(hist_a)?.tags, &mut temp);
        std::mem::swap(&mut state.workspaces.get_mut(hist_b)?.tags, &mut temp);
        std::mem::swap(&mut state.workspaces.get_mut(hist_a)?.tags, &mut temp);
        // Update dock tags and layouts.
        state.update_static();
        state
            .layout_manager
            .update_layouts(&mut state.workspaces, state.tags.all_mut());

        return Some(true);
    }
    if state.workspaces.len() == 1 {
        let last = *state.focus_manager.tag_history.get(1).unwrap();
        return state.goto_tag_handler(last);
    }
    None
}

fn close_window(state: &mut State) -> Option<bool> {
    let window = state.focus_manager.window(&state.windows)?;
    if !window.is_unmanaged() {
        let act = DisplayAction::KillWindow(window.handle);
        state.actions.push_back(act);
    }
    None
}

fn move_to_last_workspace(state: &mut State) -> Option<bool> {
    if state.workspaces.len() >= 2 && state.focus_manager.workspace_history.len() >= 2 {
        let index = *state.focus_manager.workspace_history.get(1)?;
        let wp_tags = &state.workspaces.get(index)?.tags.clone();
        let window = state.focus_manager.window_mut(&mut state.windows)?;
        window.tags = vec![*wp_tags.get(0)?];
        return Some(true);
    }
    None
}

fn next_layout(state: &mut State) -> Option<bool> {
    let workspace = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    let layout = state.layout_manager.next_layout(workspace);
    set_layout(layout, state)
}

fn previous_layout(state: &mut State) -> Option<bool> {
    let workspace = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    let layout = state.layout_manager.previous_layout(workspace);
    set_layout(layout, state)
}

fn set_layout(layout: Layout, state: &mut State) -> Option<bool> {
    let tag_id = state.focus_manager.tag(0)?;
    // When switching to Monocle or MainAndDeck layout while in Driven
    // or ClickTo focus mode, we check if the focus is given to a visible window.
    if state.focus_manager.behaviour != FocusBehaviour::Sloppy {
        //if the currently focused window is floating, nothing will be done
        let focused_window = state.focus_manager.window_history.get(0);
        let is_focused_floating = match state
            .windows
            .iter()
            .find(|w| Some(&Some(w.handle)) == focused_window)
        {
            Some(w) => w.floating(),
            None => false,
        };
        if !is_focused_floating {
            let mut to_focus: Option<Window> = None;

            if layout == Layout::Monocle {
                to_focus = state
                    .windows
                    .iter()
                    .find(|w| w.has_tag(&tag_id) && !w.is_unmanaged() && !w.floating())
                    .cloned();
            } else if layout == Layout::MainAndDeck {
                let tags_windows = state
                    .windows
                    .iter()
                    .filter(|w| w.has_tag(&tag_id) && !w.is_unmanaged() && !w.floating())
                    .collect::<Vec<&Window>>();
                if let (Some(mw), Some(tdw)) = (tags_windows.get(0), tags_windows.get(1)) {
                    // If the focused window is the main or the top of the deck, we don't do
                    // anything.
                    if let Some(&Some(h)) = focused_window {
                        if mw.handle != h && tdw.handle != h {
                            if let Some(w) = tags_windows.get(1).copied() {
                                to_focus = Some(w.clone());
                            }
                        }
                    }
                }
            }

            if let Some(w) = to_focus {
                state.focus_window(&w.handle);
            }
        }
    }
    let workspace = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    workspace.layout = layout;
    let tag = state.tags.get_mut(tag_id)?;
    match layout {
        Layout::RightWiderLeftStack | Layout::LeftWiderRightStack => {
            tag.set_layout(layout, layout.main_width());
        }
        _ => tag.set_layout(layout, workspace.main_width_percentage),
    }
    Some(true)
}

fn floating_to_tile(state: &mut State) -> Option<bool> {
    let workspace = state.focus_manager.workspace(&state.workspaces)?;
    let window = state.focus_manager.window_mut(&mut state.windows)?;
    if window.must_float() {
        return None;
    }
    //Not ideal as is_floating and must_float are connected so have to check
    //them separately
    if !window.floating() {
        return None;
    }
    let handle = window.handle;
    if window.snap_to_workspace(workspace) {
        state.sort_windows();
    }
    state.handle_window_focus(&handle);
    Some(true)
}

fn tile_to_floating(state: &mut State) -> Option<bool> {
    let width = state.default_width;
    let height = state.default_height;
    let window = state.focus_manager.window_mut(&mut state.windows)?;
    if window.must_float() {
        return None;
    }
    //Not ideal as is_floating and must_float are connected so have to check
    //them separately
    if window.floating() {
        return None;
    }

    let mut normal = window.normal;
    let offset = window.container_size.unwrap_or_default();

    normal.set_x(normal.x() + window.margin.left as i32);
    normal.set_y(normal.y() + window.margin.top as i32);
    normal.set_w(width);
    normal.set_h(height);
    let floating = normal - offset;

    window.set_floating_offsets(Some(floating));
    window.start_loc = Some(floating);
    window.set_floating(true);
    state.sort_windows();

    Some(true)
}

fn toggle_floating(state: &mut State) -> Option<bool> {
    let window = state.focus_manager.window(&state.windows)?;
    if window.floating() {
        floating_to_tile(state)
    } else {
        tile_to_floating(state)
    }
}

fn move_window_change(
    state: &mut State,
    mut handle: WindowHandle,
    layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
    val: i32,
) -> Option<bool> {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    if layout == Some(Layout::Monocle) {
        handle = helpers::relative_find(&to_reorder, is_handle, -val, true)?.handle;
        let _ = helpers::cycle_vec(&mut to_reorder, val);
    } else if layout == Some(Layout::MainAndDeck) {
        if let Some(index) = to_reorder.iter().position(|x: &Window| !x.floating()) {
            let mut window_group = to_reorder.split_off(index + 1);
            if !to_reorder.iter().any(|w| w.handle == handle) {
                handle = helpers::relative_find(&window_group, is_handle, -val, true)?.handle;
            }
            let _ = helpers::cycle_vec(&mut window_group, val);
            to_reorder.append(&mut window_group);
        }
    } else {
        let _ = helpers::reorder_vec(&mut to_reorder, is_handle, val);
    }
    state.windows.append(&mut to_reorder);
    state.handle_window_focus(&handle);
    Some(true)
}

//val and layout aren't used which is a bit awkward
fn move_window_top(
    state: &mut State,
    handle: WindowHandle,
    _layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
    swap: bool,
) -> Option<bool> {
    // Moves the selected window at index 0 of the window list.
    // If the selected window is already at index 0, it is sent to index 1.
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    let list = &mut to_reorder;
    let len = list.len();
    let index = list.iter().position(|x| is_handle(x))?;
    let item = list.get(index)?.clone();
    list.remove(index);
    let mut new_index: usize = match index {
        0 if swap => 1,
        _ => 0,
    };
    if new_index >= len {
        new_index -= len;
    }
    list.insert(new_index, item);

    state.windows.append(&mut to_reorder);
    // focus follows the window if it was not already on top of the stack
    if index > 0 {
        state.handle_window_focus(&handle);
    }
    Some(true)
}

fn focus_window_change(
    state: &mut State,
    mut handle: WindowHandle,
    layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
    val: i32,
) -> Option<bool> {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    if layout == Some(Layout::Monocle) {
        // For Monocle we want to also move windows up/down
        // Not the best solution but results
        // in desired behaviour
        handle = helpers::relative_find(&to_reorder, is_handle, -val, true)?.handle;
        let _ = helpers::cycle_vec(&mut to_reorder, val);
    } else if layout == Some(Layout::MainAndDeck) {
        let len = to_reorder.len() as i32;
        if len > 0 {
            let index = match to_reorder.iter().position(|x: &Window| !x.floating()) {
                Some(i) => {
                    if i as i32 == len - 1 {
                        i
                    } else {
                        i + 1
                    }
                }
                None => len.saturating_sub(1) as usize,
            };
            let window_group = &to_reorder[..=index];
            handle = helpers::relative_find(window_group, is_handle, -val, true)?.handle;
        }
    } else if let Some(new_focused) = helpers::relative_find(&to_reorder, is_handle, val, true) {
        handle = new_focused.handle;
    }
    state.windows.append(&mut to_reorder);
    state.handle_window_focus(&handle);
    Some(layout == Some(Layout::Monocle))
}

fn focus_window_top(state: &mut State, swap: bool) -> Option<bool> {
    let tag = state.focus_manager.tag(0)?;
    let cur = state.focus_manager.window(&state.windows).map(|w| w.handle);
    let prev = state.focus_manager.tags_last_window.get(&tag).copied();
    let next = state
        .windows
        .iter()
        .find(|x| x.tags.contains(&tag) && !x.floating() && !x.is_unmanaged())
        .map(|w| w.handle);

    match (next, cur, prev) {
        (Some(next), Some(cur), Some(prev)) if next == cur && swap => {
            state.handle_window_focus(&prev);
        }
        (Some(next), Some(cur), _) if next != cur => state.handle_window_focus(&next),
        _ => {}
    }
    None
}

fn close_all_other_windows(state: &mut State) -> Option<bool> {
    let current_window: Option<WindowHandle> =
        state.focus_manager.window(&state.windows).map(|w| w.handle);
    let current_workspace = state.focus_manager.workspace(&state.workspaces);

    for window in &state.windows {
        if window.handle.ne(&current_window?)
            && current_workspace?.is_displaying(window)
            && window.r#type.ne(&WindowType::Normal)
        {
            let act = DisplayAction::KillWindow(window.handle);
            state.actions.push_back(act);
        }
    }
    Some(true)
}

fn focus_workspace_change(state: &mut State, val: i32) -> Option<bool> {
    let current = state.focus_manager.workspace(&state.workspaces)?;
    let workspace = helpers::relative_find(&state.workspaces, |w| w == current, val, true)?.clone();

    if state.focus_manager.behaviour.is_sloppy() && state.focus_manager.sloppy_mouse_follows_focus {
        let action = workspace
            .tags
            .first()
            .and_then(|tag| state.focus_manager.tags_last_window.get(tag))
            .map_or_else(
                || DisplayAction::MoveMouseOverPoint(workspace.xyhw.center()),
                |h| DisplayAction::MoveMouseOver(*h, true),
            );
        state.actions.push_back(action);
    }
    state.focus_workspace(&workspace);
    None
}

fn rotate_tag(state: &mut State) -> Option<bool> {
    let tag_id = state.focus_manager.tag(0)?;
    let tag = state.tags.get_mut(tag_id)?;
    tag.rotate_layout()?;
    Some(true)
}

fn change_main_width(state: &mut State, delta: i8, factor: i8) -> Option<bool> {
    let workspace = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    workspace.change_main_width(delta * factor);
    let tag_id = state.focus_manager.tag(0)?;
    let tag = state.tags.get_mut(tag_id)?;
    tag.change_main_width(delta * factor);
    Some(true)
}

fn set_margin_multiplier(state: &mut State, margin_multiplier: f32) -> Option<bool> {
    let ws = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    ws.set_margin_multiplier(margin_multiplier);
    let tags = ws.tags.clone();
    if state.windows.iter().any(|w| w.r#type == WindowType::Normal) {
        let for_active_workspace = |x: &Window| -> bool {
            helpers::intersect(&tags, &x.tags) && x.r#type == WindowType::Normal
        };
        let mut to_apply_margin_multiplier =
            helpers::vec_extract(&mut state.windows, for_active_workspace);
        for w in &mut to_apply_margin_multiplier {
            if let Some(ws) = state.focus_manager.workspace(&state.workspaces) {
                w.apply_margin_multiplier(ws.margin_multiplier());
            }
        }
        state.windows.append(&mut to_apply_margin_multiplier);
    }
    Some(true)
}

fn send_workspace_to_tag(state: &mut State, ws_index: usize, tag_index: usize) -> bool {
    // todo: address inconsistency of using the index instead of the id here
    if ws_index < state.workspaces.len() && tag_index < state.tags.len_normal() {
        let workspace = &state.workspaces[ws_index].clone();
        state.focus_workspace(workspace);
        state.goto_tag_handler(tag_index + 1);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::ScratchPad, models::Tags};

    #[test]
    fn return_to_last_tag_should_go_back_to_last_tag() {
        let mut manager = Manager::new_test(vec![
            "A15".to_string(),
            "B24".to_string(),
            "C".to_string(),
            "6D4".to_string(),
            "E39".to_string(),
            "F67".to_string(),
        ]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 1,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap();
        assert_eq!(current_tag, 1);

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 2,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 2);

        manager.command_handler(&Command::ReturnToLastTag);
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 1);
    }

    #[test]
    fn go_to_tag_should_return_false_if_no_screen_is_created() {
        let mut manager = Manager::new_test(vec![]);
        // no screen creation here
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 6,
            swap: false
        }));
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 2,
            swap: false
        }));
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 15,
            swap: false
        }));
    }

    #[test]
    fn go_to_tag_should_create_at_least_one_tag_per_screen_no_more() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        // no tag creation here but one tag per screen is created
        assert!(manager.command_handler(&Command::GoToTag {
            tag: 2,
            swap: false
        }));
        assert!(manager.command_handler(&Command::GoToTag {
            tag: 1,
            swap: false
        }));
        // we only have one tag per screen created automatically
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 3,
            swap: false
        }));
    }

    #[test]
    fn go_to_tag_should_return_false_on_invalid_input() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.state.tags = Tags::new();
        manager.state.tags.add_new("A15", Layout::default());
        manager.state.tags.add_new("B24", Layout::default());
        manager.state.tags.add_new("C", Layout::default());
        manager.state.tags.add_new("6D4", Layout::default());
        manager.state.tags.add_new("E39", Layout::default());
        manager.state.tags.add_new("F67", Layout::default());
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 0,
            swap: false
        }));
        assert!(!manager.command_handler(&Command::GoToTag {
            tag: 999,
            swap: false
        }));
    }

    #[test]
    fn go_to_tag_should_go_to_tag_and_set_history() {
        let mut manager = Manager::new_test(vec![
            "A15".to_string(),
            "B24".to_string(),
            "C".to_string(),
            "6D4".to_string(),
            "E39".to_string(),
            "F67".to_string(),
        ]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 6,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap();
        assert_eq!(current_tag, 6);

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 2,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 2);

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 3,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 3);

        assert!(manager.command_handler(&Command::GoToTag {
            tag: 4,
            swap: false
        }));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 4);

        // test tag history
        assert_eq!(manager.state.focus_manager.tag(1).unwrap_or_default(), 3);
        assert_eq!(manager.state.focus_manager.tag(2).unwrap_or_default(), 2);
        assert_eq!(manager.state.focus_manager.tag(3).unwrap_or_default(), 6);
    }

    #[test]
    fn focus_tag_change_should_go_to_previous_and_next_tag() {
        let mut manager = Manager::new_test(vec![
            "A15".to_string(),
            "B24".to_string(),
            "C".to_string(),
            "6D4".to_string(),
            "E39".to_string(),
            "F67".to_string(),
        ]);
        manager.screen_create_handler(Screen::default());
        let state = &mut manager.state;

        state.focus_tag(&2);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 2);

        focus_tag_change(state, 1);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 3);

        focus_tag_change(state, -1);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 2);

        focus_tag_change(state, 2);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 4);

        focus_tag_change(state, -5);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 5);

        focus_tag_change(state, 3);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 2);

        focus_tag_change(state, 13);
        assert_eq!(state.focus_manager.tag(0).unwrap(), 3);
    }

    #[test]
    fn focus_window_top() {
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
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(3), None, None),
            -1,
            -1,
        );

        let expected = manager.state.windows[0].clone();
        let initial = manager.state.windows[1].clone();

        manager.state.focus_window(&initial.handle);

        manager.command_handler(&Command::FocusWindowTop { swap: false });
        let actual = manager
            .state
            .focus_manager
            .window(&manager.state.windows)
            .unwrap()
            .handle;
        assert_eq!(expected.handle, actual);

        manager.command_handler(&Command::FocusWindowTop { swap: false });
        let actual = manager
            .state
            .focus_manager
            .window(&manager.state.windows)
            .unwrap()
            .handle;
        assert_eq!(expected.handle, actual);

        manager.command_handler(&Command::FocusWindowTop { swap: true });
        let actual = manager
            .state
            .focus_manager
            .window(&manager.state.windows)
            .unwrap()
            .handle;
        assert_eq!(initial.handle, actual);
    }

    #[test]
    fn move_window_top() {
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
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(3), None, None),
            -1,
            -1,
        );

        let expected = manager.state.windows[0].clone();
        let initial = manager.state.windows[1].clone();

        manager.state.focus_window(&initial.handle);

        manager.command_handler(&Command::MoveWindowTop { swap: false });
        assert_eq!(manager.state.windows[0].handle, initial.handle);

        manager.command_handler(&Command::MoveWindowTop { swap: false });
        assert_eq!(manager.state.windows[0].handle, initial.handle);

        manager.command_handler(&Command::MoveWindowTop { swap: true });
        assert_eq!(manager.state.windows[0].handle, expected.handle);
    }

    #[test]
    fn move_window_to_next_or_prev_tag_should_be_able_to_cycle() {
        let mut manager = Manager::new_test(vec![
            "AO".to_string(),
            "EU".to_string(),
            "ID".to_string(),
            "HT".to_string(),
            "NS".to_string(),
        ]);
        manager.screen_create_handler(Screen::default());
        manager.window_created_handler(
            Window::new(WindowHandle::MockHandle(1), None, None),
            -1,
            -1,
        );

        let first_tag = manager.state.tags.get(1).unwrap().id;
        let third_tag = manager.state.tags.get(3).unwrap().id;
        let last_tag = manager.state.tags.get(5).unwrap().id;

        assert!(manager.state.windows[0].has_tag(&first_tag));

        manager.command_handler(&Command::MoveWindowToPreviousTag { follow: true });
        assert!(manager.state.windows[0].has_tag(&last_tag));

        (0..3).for_each(|_| {
            manager.command_handler(&Command::MoveWindowToNextTag { follow: false });
            manager.command_handler(&Command::FocusNextTag);
        });
        assert!(manager.state.windows[0].has_tag(&third_tag));
    }

    #[test]
    fn move_window_to_next_or_prev_tag_should_be_able_to_keep_window_focused() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
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
        let expected_tag = manager.state.tags.get(2).unwrap().id;
        manager.command_handler(&Command::SendWindowToTag {
            window: None,
            tag: expected_tag,
        });
        let initial = manager.state.windows[0].clone();

        manager.command_handler(&Command::MoveWindowToNextTag { follow: true });

        assert_eq!(
            *manager.state.focus_manager.tag_history.get(0).unwrap(),
            expected_tag
        );
        assert_eq!(manager.state.windows[0].handle, initial.handle);
    }

    #[test]
    fn show_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;
        let first_tag = manager.state.tags.get(1).unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::MockHandle(mock_window as i32);
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        // make sure the window is on the first tag
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

        assert!(!window.tags.iter().any(|tag| *tag == nsp_tag));
    }

    #[test]
    fn hide_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;
        let first_tag = manager.state.tags.get(1).unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::MockHandle(mock_window as i32);
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        // make sure the window is on the first tag
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

        assert!(window.tags.iter().any(|tag| *tag == nsp_tag));
    }

    #[test]
    fn toggle_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        let mock_window = 1_u32;
        let window_handle = WindowHandle::MockHandle(mock_window as i32);
        let scratchpad_name = "Alacritty";
        manager.window_created_handler(Window::new(window_handle, None, Some(mock_window)), -1, -1);
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.to_owned(),
            value: "".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name.to_owned(), VecDeque::from([mock_window]));

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name.to_owned()));

        // assert window is hidden
        {
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(mock_window))
                .unwrap();

            assert!(window.tags.iter().any(|tag| *tag == nsp_tag));
        }

        manager.command_handler(&Command::ToggleScratchPad(scratchpad_name.to_owned()));

        // assert window is hidden
        {
            let window = manager
                .state
                .windows
                .iter_mut()
                .find(|w| w.pid == Some(mock_window))
                .unwrap();

            assert!(!window.tags.iter().any(|tag| *tag == nsp_tag));
        }
    }

    #[test]
    /// Test release scratchpad command for 1 window in the scratchpad
    /// After releasing, the scratchpad should not be active anymore (no more windows)
    fn release_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());

        // setup
        let mock_window1 = 10_u32;
        let scratchpad_name = "Alacritty";
        manager
            .state
            .active_scratchpads
            .insert(scratchpad_name.to_owned(), VecDeque::from([mock_window1]));
        manager.window_created_handler(
            Window::new(
                WindowHandle::MockHandle(mock_window1 as i32),
                None,
                Some(mock_window1),
            ),
            -1,
            -1,
        );

        let expected_tag = manager.state.tags.get(1).unwrap().id;

        // Release Scratchpad
        manager.command_handler(&Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::Handle(WindowHandle::MockHandle(mock_window1 as i32)),
            tag: Some(expected_tag),
        });

        // assert
        assert!(manager
            .state
            .active_scratchpads
            .get(scratchpad_name)
            .is_none());
        assert_eq!(
            *manager.state.focus_manager.tag_history.get(0).unwrap(),
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

        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name = "Alacritty";
        manager.state.active_scratchpads.insert(
            scratchpad_name.to_owned(),
            VecDeque::from([mock_window1, mock_window2, mock_window3]),
        );
        for window in [mock_window1, mock_window2, mock_window3] {
            manager.window_created_handler(
                Window::new(WindowHandle::MockHandle(window as i32), None, Some(window)),
                -1,
                -1,
            );
        }

        let expected_tag = manager.state.tags.get(1).unwrap().id;

        // Release Scratchpad
        manager.command_handler(&Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::Handle(WindowHandle::MockHandle(mock_window1 as i32)),
            tag: Some(expected_tag),
        });

        // assert
        let scratchpad = manager
            .state
            .active_scratchpads
            .get_mut(scratchpad_name)
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
            *manager.state.focus_manager.tag_history.get(0).unwrap(),
            expected_tag
        );
    }

    #[test]
    fn attach_scratchpad_test() {
        let mut manager = Manager::new_test(vec!["AO".to_string(), "EU".to_string()]);
        manager.screen_create_handler(Default::default());
        let nsp_tag = manager.state.tags.get_hidden_by_label("NSP").unwrap().id;

        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name = "Alacritty";
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.to_owned(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager.state.active_scratchpads.insert(
            scratchpad_name.to_owned(),
            VecDeque::from([mock_window2, mock_window3]),
        );
        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::MockHandle(mock_window as i32),
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
            window: Some(WindowHandle::MockHandle(mock_window1 as i32)),
            scratchpad: scratchpad_name.to_owned(),
        });

        // assert
        let scratchpad = manager
            .state
            .active_scratchpads
            .get_mut(scratchpad_name)
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
                .map(|w| dbg!(w.has_tag(&nsp_tag)))
                .unwrap());
        }
        assert_eq!(scratchpad.pop_front(), None);
    }

    #[test]
    fn next_valid_pid_test() {
        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let mock_window4 = 4_u32;

        let mut managed_windows = vec![mock_window1, mock_window2, mock_window3, mock_window4]
            .iter()
            .map(|pid| Window::new(WindowHandle::MockHandle(*pid as i32), None, Some(*pid)))
            .collect::<Vec<Window>>();
        let mut scratchpad =
            VecDeque::from([mock_window1, mock_window2, mock_window3, mock_window4]);

        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(1)
        );

        managed_windows.remove(1);
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(1)
        );

        scratchpad.pop_front();
        assert_eq!(
            next_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(3)
        );
        assert_eq!(scratchpad.len(), 2);
    }

    #[test]
    fn prev_valid_pid_test() {
        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let mock_window4 = 4_u32;

        let mut managed_windows = vec![mock_window1, mock_window2, mock_window3, mock_window4]
            .iter()
            .map(|pid| Window::new(WindowHandle::MockHandle(*pid as i32), None, Some(*pid)))
            .collect::<Vec<Window>>();
        let mut scratchpad =
            VecDeque::from([mock_window1, mock_window2, mock_window3, mock_window4]);

        assert_eq!(
            prev_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(4)
        );

        managed_windows.remove(2);
        assert_eq!(
            prev_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(4)
        );

        scratchpad.pop_back();
        assert_eq!(
            prev_valid_scratchpad_pid(&mut scratchpad, &managed_windows),
            Some(2)
        );
        assert_eq!(scratchpad.len(), 2);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn cycle_scratchpad_window_test() {
        fn is_visible<C: Config, SERVER: DisplayServer>(
            manager: &Manager<C, SERVER>,
            pid: u32,
            nsp_tag: TagId,
        ) -> bool {
            manager
                .state
                .windows
                .iter()
                .find(|w| w.pid == Some(pid))
                .map(|w| !w.has_tag(&nsp_tag))
                .unwrap()
        }
        fn is_only_first_visible<C: Config, SERVER: DisplayServer>(
            manager: &Manager<C, SERVER>,
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

        // setup
        let mock_window1 = 1_u32;
        let mock_window2 = 2_u32;
        let mock_window3 = 3_u32;
        let scratchpad_name = "Alacritty";

        for mock_window in [mock_window1, mock_window2, mock_window3] {
            let mut window = Window::new(
                WindowHandle::MockHandle(mock_window as i32),
                None,
                Some(mock_window),
            );
            if mock_window != mock_window1 {
                window.tag(&nsp_tag);
            }

            manager.window_created_handler(window, -1, -1);
        }
        manager.state.scratchpads.push(ScratchPad {
            name: scratchpad_name.to_owned(),
            value: "scratchpad".to_string(),
            x: None,
            y: None,
            height: None,
            width: None,
        });
        manager.state.active_scratchpads.insert(
            scratchpad_name.to_owned(),
            VecDeque::from([mock_window1, mock_window2, mock_window3]),
        );

        cycle_scratchpad_window(&mut manager, scratchpad_name, Direction::Forward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, scratchpad_name, Direction::Forward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, scratchpad_name, Direction::Backward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), None);

        cycle_scratchpad_window(&mut manager, scratchpad_name, Direction::Backward);
        let mut scratchpad_iterator = manager
            .state
            .active_scratchpads
            .get(scratchpad_name)
            .unwrap()
            .iter();
        assert!(is_only_first_visible(
            &manager,
            scratchpad_iterator.clone().copied(),
            nsp_tag
        ));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window1));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window2));
        assert_eq!(scratchpad_iterator.next(), Some(&mock_window3));
        assert_eq!(scratchpad_iterator.next(), None);
    }
}
