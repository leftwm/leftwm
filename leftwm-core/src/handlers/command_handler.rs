#![allow(clippy::wildcard_imports)]
#![allow(clippy::shadow_unrelated)]

// NOTE: there apears to be a clippy bug with shadow_unrelated and the (?) Operator
// allow shadow should be removed once it is resolved
// https://github.com/rust-lang/rust-clippy/issues/6563

use super::*;
use crate::child_process::Children;
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
use crate::models::{TagId, WindowState};
use crate::state::State;
use crate::utils::helpers::relative_find;
use crate::utils::{child_process::exec_shell, helpers};
use crate::{config::Config, models::FocusBehaviour};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /* Please also update src/bin/leftwm-check if any of the following apply after your update:
     * - a command now requires a value
     * - a command no longer requires a value
     * - a new command is introduced that requires a value
     *  */
    /// Processes a command and invokes the associated function.
    pub fn command_handler(&mut self, command: &Command) -> bool {
        process_internal(self, command).unwrap_or(false)
    }
}

fn process_internal<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    command: &Command,
) -> Option<bool> {
    let state = &mut manager.state;
    match command {
        Command::Execute(shell_command) => execute(&mut manager.children, shell_command),

        Command::ToggleScratchPad(name) => toggle_scratchpad(manager, name),

        Command::ToggleFullScreen => toggle_state(state, WindowState::Fullscreen),
        Command::ToggleSticky => toggle_state(state, WindowState::Sticky),

        Command::SendWindowToTag(tag) => move_to_tag(*tag, manager),
        Command::MoveWindowToNextWorkspace => move_window_to_workspace_change(manager, 1),
        Command::MoveWindowToPreviousWorkspace => move_window_to_workspace_change(manager, -1),
        Command::MoveWindowUp => move_focus_common_vars(move_window_change, state, -1),
        Command::MoveWindowDown => move_focus_common_vars(move_window_change, state, 1),
        Command::MoveWindowTop => move_focus_common_vars(move_window_top, state, 0),

        Command::GotoTag(tag) => goto_tag(state, *tag),

        Command::CloseWindow => close_window(state),
        Command::SwapScreens => swap_tags(state),
        Command::MoveWindowToLastWorkspace => move_to_last_workspace(state),
        Command::NextLayout => next_layout(state),
        Command::PreviousLayout => previous_layout(state),

        Command::SetLayout(layout) => set_layout(*layout, state),

        Command::FloatingToTile => floating_to_tile(state),
        Command::TileToFloating => tile_to_floating(state),
        Command::ToggleFloating => toggle_floating(state),

        Command::FocusNextTag => focus_tag_change(state, 1),
        Command::FocusPreviousTag => focus_tag_change(state, -1),
        Command::FocusWindowUp => move_focus_common_vars(focus_window_change, state, -1),
        Command::FocusWindowDown => move_focus_common_vars(focus_window_change, state, 1),
        Command::FocusWorkspaceNext => focus_workspace_change(state, 1),
        Command::FocusWorkspacePrevious => focus_workspace_change(state, -1),

        Command::MouseMoveWindow => None,

        Command::SoftReload => {
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
        Command::Other(cmd) => Some(C::command_handler(cmd, manager)),
    }
}

fn execute(children: &mut Children, shell_command: &str) -> Option<bool> {
    let _ = exec_shell(shell_command, children);
    None
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
        .find(|s| name == s.name.clone())?
        .clone();

    let mut handle = None;
    if let Some(ws) = manager
        .state
        .focus_manager
        .workspace(&manager.state.workspaces)
    {
        handle = manager
            .state
            .windows
            .iter()
            .find(|w| ws.is_managed(w))
            .map(|w| w.handle);
    }

    if let Some(nsp_tag) = manager.state.tags.get_hidden_by_label("NSP") {
        if let Some(id) = manager.state.active_scratchpads.get(&scratchpad.name) {
            if let Some(window) = manager.state.windows.iter_mut().find(|w| w.pid == *id) {
                let previous_tag = window.tags[0];
                let is_visible = window.has_tag(current_tag);
                window.clear_tags();
                if is_visible {
                    // Hide the scratchpad.
                    window.tag(&nsp_tag.id);
                    // Make sure when changing focus the scratchpad is currently focused.
                    if Some(&Some(window.handle))
                        != manager.state.focus_manager.window_history.get(0)
                    {
                        handle = None;
                    } else if let Some(Some(prev)) =
                        manager.state.focus_manager.window_history.get(1)
                    {
                        handle = Some(*prev);
                    }
                } else {
                    // Remove the entry for the previous tag to prevent the scratchpad being
                    // refocused.
                    manager
                        .state
                        .focus_manager
                        .tags_last_window
                        .remove(&previous_tag);
                    // Show the scratchpad.
                    window.tag(current_tag);
                    handle = Some(window.handle);
                }
                let act = DisplayAction::SetWindowTags(window.handle, window.tags.clone());
                manager.state.actions.push_back(act);
                manager.state.sort_windows();
                if let Some(h) = handle {
                    handle_focus(&mut manager.state, h);
                    if !is_visible {
                        manager.state.move_to_top(&h);
                    }
                }

                return Some(true);
            }
        }

        log::debug!(
            "no active scratchpad found for name {:?}. creating a new one",
            scratchpad.name
        );
        let name = scratchpad.name.clone();
        let pid = exec_shell(&scratchpad.value, &mut manager.children);
        manager.state.active_scratchpads.insert(name, pid);
        return None;
    }
    log::warn!("unable to find NSP tag");
    None
}

fn toggle_state(state: &mut State, window_state: WindowState) -> Option<bool> {
    let window = state.focus_manager.window(&state.windows)?;
    let handle = window.handle;
    let toggle_to = !window.has_state(&window_state);
    let act = DisplayAction::SetState(handle, toggle_to, window_state);
    state.actions.push_back(act);
    match window_state {
        WindowState::Fullscreen => Some(handle_focus(state, handle)),
        _ => Some(true),
    }
}

fn move_to_tag<C: Config, SERVER: DisplayServer>(
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

    let handle = manager
        .state
        .focus_manager
        .window(&manager.state.windows)?
        .handle;
    //Focus the next or previous window on the workspace
    let new_handle = manager.get_next_or_previous(&handle);

    let window = manager
        .state
        .focus_manager
        .window_mut(&mut manager.state.windows)?;
    window.clear_tags();
    window.set_floating(false);
    window.tag(&tag.id);
    window.apply_margin_multiplier(margin_multiplier);
    let act = DisplayAction::SetWindowTags(window.handle, vec![tag.id]);
    manager.state.actions.push_back(act);

    manager.state.sort_windows();
    if let Some(new_handle) = new_handle {
        manager.state.focus_window(&new_handle);
    } else {
        let act = DisplayAction::Unfocus;
        manager.state.actions.push_back(act);
        manager.state.focus_manager.window_history.push_front(None);
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
    move_to_tag(*tag_num, manager)
}

fn goto_tag(state: &mut State, input_tag: TagId) -> Option<bool> {
    let current_tag = state.focus_manager.tag(0).unwrap_or_default();
    let previous_tag = state.focus_manager.tag(1).unwrap_or_default();
    let destination_tag = if !state.disable_current_tag_swap && current_tag == input_tag {
        previous_tag
    } else {
        input_tag
    };
    state.goto_tag_handler(destination_tag)
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
    let layout = state.layout_manager.next_layout(workspace.layout);
    set_layout(layout, state)
}

fn previous_layout(state: &mut State) -> Option<bool> {
    let workspace = state.focus_manager.workspace_mut(&mut state.workspaces)?;
    let layout = state.layout_manager.previous_layout(workspace.layout);
    set_layout(layout, state)
}

fn set_layout(layout: Layout, state: &mut State) -> Option<bool> {
    let tag_id = state.focus_manager.tag(0)?;
    // When switching to Monocle or MainAndDeck layout while in Driven
    // or ClickTo focus mode, we check if the focus is given to a visible window.
    if state.focus_manager.behaviour != FocusBehaviour::Sloppy {
        //if the currently focused window is floatin, nothing will be done
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
    tag.set_layout(layout, workspace.main_width_percentage);
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
    window.snap_to_workspace(workspace);
    let handle = window.handle;
    Some(handle_focus(state, handle))
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

fn move_focus_common_vars<F>(func: F, state: &mut State, val: i32) -> Option<bool>
where
    F: Fn(&mut State, i32, WindowHandle, Option<Layout>, Vec<Window>) -> Option<bool>,
{
    let handle = state.focus_manager.window(&state.windows)?.handle;
    let tag_id = state.focus_manager.tag(0)?;
    let tag = state.tags.get(tag_id)?;
    let (tags, layout) = (vec![tag_id], Some(tag.layout));

    let for_active_workspace =
        |x: &Window| -> bool { helpers::intersect(&tags, &x.tags) && !x.is_unmanaged() };

    let to_reorder = helpers::vec_extract(&mut state.windows, for_active_workspace);
    func(state, val, handle, layout, to_reorder)
}

fn move_window_change(
    state: &mut State,
    val: i32,
    mut handle: WindowHandle,
    layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
) -> Option<bool> {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    if let Some(Layout::Monocle) = layout {
        handle = helpers::relative_find(&to_reorder, is_handle, -val, true)?.handle;
        let _ = helpers::cycle_vec(&mut to_reorder, val);
    } else if let Some(Layout::MainAndDeck) = layout {
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
    Some(handle_focus(state, handle))
}

//val and layout aren't used which is a bit awkward
fn move_window_top(
    state: &mut State,
    _val: i32,
    handle: WindowHandle,
    _layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
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
        0 => 1,
        _ => 0,
    };
    if new_index >= len {
        new_index -= len;
    }
    list.insert(new_index, item);

    state.windows.append(&mut to_reorder);
    // focus follows the window if it was not already on top of the stack
    if index > 0 {
        return Some(handle_focus(state, handle));
    }
    Some(true)
}

fn focus_window_change(
    state: &mut State,
    val: i32,
    mut handle: WindowHandle,
    layout: Option<Layout>,
    mut to_reorder: Vec<Window>,
) -> Option<bool> {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    if let Some(Layout::Monocle) = layout {
        // For Monocle we want to also move windows up/down
        // Not the best solution but results
        // in desired behaviour
        handle = helpers::relative_find(&to_reorder, is_handle, -val, true)?.handle;
        let _ = helpers::cycle_vec(&mut to_reorder, val);
    } else if let Some(Layout::MainAndDeck) = layout {
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
    Some(handle_focus(state, handle))
}

fn focus_workspace_change(state: &mut State, val: i32) -> Option<bool> {
    let current = state.focus_manager.workspace(&state.workspaces)?;
    let workspace = helpers::relative_find(&state.workspaces, |w| w == current, val, true)?.clone();
    state.focus_workspace(&workspace);
    if state.focus_manager.behaviour == FocusBehaviour::Sloppy {
        let act = DisplayAction::MoveMouseOverPoint(workspace.xyhw.center());
        state.actions.push_back(act);
    }
    let window = state
        .windows
        .iter()
        .find(|w| workspace.is_displaying(w) && w.r#type == WindowType::Normal)?
        .clone();
    Some(handle_focus(state, window.handle))
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

fn handle_focus(state: &mut State, handle: WindowHandle) -> bool {
    match state.focus_manager.behaviour {
        FocusBehaviour::Sloppy => {
            let act = DisplayAction::MoveMouseOver(handle);
            state.actions.push_back(act);
            true
        }
        _ => state.focus_window(&handle),
    }
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
    use crate::models::Tags;

    #[test]
    fn go_to_tag_should_return_false_if_no_screen_is_created() {
        let mut manager = Manager::new_test(vec![]);
        // no screen creation here
        assert!(!manager.command_handler(&Command::GotoTag(6)));
        assert!(!manager.command_handler(&Command::GotoTag(2)));
        assert!(!manager.command_handler(&Command::GotoTag(15)));
    }

    #[test]
    fn go_to_tag_should_create_at_least_one_tag_per_screen_no_more() {
        let mut manager = Manager::new_test(vec![]);
        manager.screen_create_handler(Screen::default());
        manager.screen_create_handler(Screen::default());
        // no tag creation here but one tag per screen is created
        assert!(manager.command_handler(&Command::GotoTag(2)));
        assert!(manager.command_handler(&Command::GotoTag(1)));
        // we only have one tag per screen created automatically
        assert!(!manager.command_handler(&Command::GotoTag(3)));
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
        assert!(!manager.command_handler(&Command::GotoTag(0)));
        assert!(!manager.command_handler(&Command::GotoTag(999)));
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

        assert!(manager.command_handler(&Command::GotoTag(6)));
        let current_tag = manager.state.focus_manager.tag(0).unwrap();
        assert_eq!(current_tag, 6);

        assert!(manager.command_handler(&Command::GotoTag(2)));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 2);

        assert!(manager.command_handler(&Command::GotoTag(3)));
        let current_tag = manager.state.focus_manager.tag(0).unwrap_or_default();
        assert_eq!(current_tag, 3);

        assert!(manager.command_handler(&Command::GotoTag(4)));
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
}
