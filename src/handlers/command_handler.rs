#![allow(clippy::wildcard_imports)]
#![allow(clippy::shadow_unrelated)]

// NOTE: there apears to be a clippy bug with shadow_unrelated and the (?) Operator
// allow shadow should be removed once it is resolved
// https://github.com/rust-lang/rust-clippy/issues/6563

use super::*;
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::models::TagId;
use crate::state::State;
use crate::utils::{child_process::exec_shell, helpers};
use crate::{config::Config, models::FocusBehaviour};
use std::str::FromStr;

/* Please also update src/bin/leftwm-check if any of the following apply after your update:
 * - a command now requires a value
 * - a command no longer requires a value
 * - a new command is introduced that requires a value
 *  */
pub fn process(
    manager: &mut Manager,
    state: &impl State,
    config: &impl Config,
    command: &Command,
    val: &Option<String>,
) -> bool {
    process_internal(manager, state, config, command, val).unwrap_or(false)
}

pub fn process_internal(
    manager: &mut Manager,
    state: &impl State,
    config: &impl Config,
    command: &Command,
    val: &Option<String>,
) -> Option<bool> {
    match command {
        Command::Execute => execute(manager, val),

        Command::ToggleScratchPad => toggle_scratchpad(manager, val),

        Command::ToggleFullScreen => toggle_fullscreen(manager),

        Command::MoveToTag => move_to_tag(val, manager),

        Command::MoveWindowUp => move_focus_common_vars(move_window_change, manager, -1),
        Command::MoveWindowDown => move_focus_common_vars(move_window_change, manager, 1),
        Command::MoveWindowTop => move_focus_common_vars(move_window_top, manager, 0),

        Command::GotoTag => goto_tag(manager, val, config),

        Command::CloseWindow => close_window(manager),
        Command::SwapTags => swap_tags(manager),
        Command::MoveToLastWorkspace => move_to_last_workspace(manager),
        Command::NextLayout => next_layout(manager),
        Command::PreviousLayout => previous_layout(manager),

        Command::SetLayout => set_layout(val, manager),

        Command::FloatingToTile => floating_to_tile(manager),

        Command::FocusNextTag => focus_tag_change(manager, 1),
        Command::FocusPreviousTag => focus_tag_change(manager, -1),
        Command::FocusWindowUp => move_focus_common_vars(focus_window_change, manager, -1),
        Command::FocusWindowDown => move_focus_common_vars(focus_window_change, manager, 1),
        Command::FocusWorkspaceNext => focus_workspace_change(manager, 1),
        Command::FocusWorkspacePrevious => focus_workspace_change(manager, -1),

        Command::MouseMoveWindow => None,

        Command::SoftReload => {
            if let Err(err) = state.save(manager) {
                log::error!("Cannot save state: {}", err);
            }
            manager.hard_reload();
            None
        }
        Command::HardReload => {
            manager.hard_reload();
            None
        }

        Command::RotateTag => rotate_tag(manager),

        Command::IncreaseMainWidth => change_main_width(manager, val, 1),
        Command::DecreaseMainWidth => change_main_width(manager, val, -1),
        Command::SetMarginMultiplier => set_margin_multiplier(manager, val),
    }
}

fn execute(manager: &mut Manager, val: &Option<String>) -> Option<bool> {
    let _ = exec_shell(val.as_ref()?, manager);
    None
}

fn toggle_scratchpad(manager: &mut Manager, val: &Option<String>) -> Option<bool> {
    let name = val.clone()?;
    let tag = &manager.focused_tag(0)?;
    let s = manager
        .scratchpads
        .iter()
        .find(|s| name == s.name.clone())?
        .clone();

    let mut handle = None;
    if let Some(ws) = manager.focused_workspace() {
        handle = manager
            .windows
            .iter()
            .find(|w| ws.is_managed(w))
            .map(|w| w.handle);
    }

    if let Some(id) = manager.active_scratchpads.get(&s.name) {
        if let Some(w) = manager.windows.iter_mut().find(|w| w.pid == *id) {
            let is_tagged = w.has_tag(tag);
            w.clear_tags();
            if is_tagged {
                w.tag("NSP");
                if let Some(Some(prev)) = manager.focus_manager.window_history.get(1) {
                    handle = Some(*prev);
                }
            } else {
                w.tag(tag);
                handle = Some(w.handle);
            }
            let act = DisplayAction::SetWindowTags(w.handle, w.tags.get(0)?.to_string());
            manager.actions.push_back(act);

            manager.sort_windows();
            if let Some(h) = handle {
                handle_focus(manager, h);
                if !is_tagged {
                    manager.move_to_top(&h);
                }
            }
            return Some(true);
        }
    }
    let name = s.name.clone();
    let pid = exec_shell(&s.value, manager);
    manager.active_scratchpads.insert(name, pid);
    None
}

fn toggle_fullscreen(manager: &mut Manager) -> Option<bool> {
    let window = manager.focused_window_mut()?;
    let handle = window.handle;
    let act = window.toggle_fullscreen()?;
    manager.actions.push_back(act);
    Some(handle_focus(manager, handle))
}

fn move_to_tag(val: &Option<String>, manager: &mut Manager) -> Option<bool> {
    let tag_num: usize = val.as_ref()?.parse().ok()?;
    let tag = manager.tags.get(tag_num - 1)?.clone();

    // In order to apply the correct margin multiplier we want to copy this value
    // from any window already present on the target tag
    let margin_multiplier = match manager.windows.iter().find(|w| w.has_tag(&tag.id)) {
        Some(w) => w.margin_multiplier(),
        None => 1.0,
    };

    let handle = manager.focused_window()?.handle;
    //Focus the next or previous window on the workspace
    let new_handle = window_handler::get_next_or_previous(manager, &handle);

    let window = manager.focused_window_mut()?;
    window.clear_tags();
    window.set_floating(false);
    window.tag(&tag.id);
    window.apply_margin_multiplier(margin_multiplier);
    let act = DisplayAction::SetWindowTags(window.handle, tag.id);
    manager.actions.push_back(act);

    manager.sort_windows();
    if let Some(new_handle) = new_handle {
        focus_handler::focus_window(manager, &new_handle);
    }
    Some(true)
}

fn goto_tag(manager: &mut Manager, val: &Option<String>, config: &impl Config) -> Option<bool> {
    let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
    let previous_tag = manager.tag_index(&manager.focused_tag(1).unwrap_or_default());

    let input_tag = val.as_ref()?.parse().ok()?;
    let mut destination_tag = input_tag;
    if !config.disable_current_tag_swap() {
        destination_tag = match (current_tag, previous_tag, input_tag) {
            (Some(curr_tag), Some(prev_tag), inp_tag) if curr_tag + 1 == inp_tag => prev_tag + 1, // if current tag is the same as the destination tag, go to the previous tag instead
            (_, _, _) => input_tag, // go to the input tag tag
        };
    }
    Some(goto_tag_handler::process(manager, destination_tag))
}

fn focus_tag_change(manager: &mut Manager, delta: i8) -> Option<bool> {
    let current = manager.focused_tag(0)?;
    let active_tags: Vec<(usize, TagId)> = manager
        .tags
        .iter()
        .enumerate()
        .filter(|(_, tag)| !tag.hidden)
        .map(|(i, tag)| (i + 1, tag.id.clone()))
        .collect();
    let mut index = active_tags
        .iter()
        .position(|(_, tag_id)| *tag_id == current)?;
    if delta.is_negative() {
        index = match index.checked_sub(delta.abs() as usize) {
            Some(i) => i,
            None => active_tags.len() - 1,
        }
    } else {
        index += delta as usize;
        if index >= active_tags.len() {
            index = 0;
        }
    }
    let (next, _) = *active_tags.get(index)?;
    Some(goto_tag_handler::process(manager, next))
}

fn swap_tags(manager: &mut Manager) -> Option<bool> {
    if manager.workspaces.len() >= 2 && manager.focus_manager.workspace_history.len() >= 2 {
        let hist_a = *manager.focus_manager.workspace_history.get(0)?;
        let hist_b = *manager.focus_manager.workspace_history.get(1)?;
        //Update workspace tags
        let mut temp = vec![];
        std::mem::swap(&mut manager.workspaces.get_mut(hist_a)?.tags, &mut temp);
        std::mem::swap(&mut manager.workspaces.get_mut(hist_b)?.tags, &mut temp);
        std::mem::swap(&mut manager.workspaces.get_mut(hist_a)?.tags, &mut temp);
        //Update dock tags
        manager.update_docks();
        return Some(true);
    }
    if manager.workspaces.len() == 1 {
        let last = manager
            .focus_manager
            .tag_history
            .get(1)
            .map(std::string::ToString::to_string)?;

        let tag_index = manager.tags.iter().position(|x| x.id == last)? + 1;
        return Some(goto_tag_handler::process(manager, tag_index));
    }
    None
}

fn close_window(manager: &mut Manager) -> Option<bool> {
    let window = manager.focused_window()?;
    if !window.is_unmanaged() {
        let act = DisplayAction::KillWindow(window.handle);
        manager.actions.push_back(act);
    }
    None
}

fn move_to_last_workspace(manager: &mut Manager) -> Option<bool> {
    if manager.workspaces.len() >= 2 && manager.focus_manager.workspace_history.len() >= 2 {
        let index = *manager.focus_manager.workspace_history.get(1)?;
        let wp_tags = &manager.workspaces.get(index)?.tags.clone();
        let window = manager.focused_window_mut()?;
        window.tags = vec![wp_tags.get(0)?.clone()];
        return Some(true);
    }
    None
}

fn next_layout(manager: &mut Manager) -> Option<bool> {
    let workspace = manager
        .focus_manager
        .workspace_mut(&mut manager.workspaces)?;
    workspace.next_layout(&mut manager.tags);
    Some(true)
}

fn previous_layout(manager: &mut Manager) -> Option<bool> {
    let workspace = manager
        .focus_manager
        .workspace_mut(&mut manager.workspaces)?;
    workspace.prev_layout(&mut manager.tags);
    Some(true)
}

fn set_layout(val: &Option<String>, manager: &mut Manager) -> Option<bool> {
    let layout = Layout::from_str(val.as_ref()?).ok()?;
    let workspace = manager
        .focus_manager
        .workspace_mut(&mut manager.workspaces)?;
    workspace.set_layout(&mut manager.tags, layout);
    Some(true)
}

fn floating_to_tile(manager: &mut Manager) -> Option<bool> {
    let workspace = manager.focused_workspace()?.clone();
    let window = manager.focused_window_mut()?;
    if window.must_float() {
        return None;
    }
    //Not ideal as is_floating and must_float are connected so have to check
    //them separately
    if !window.floating() {
        return None;
    }
    window_handler::snap_to_workspace(window, &workspace);
    let handle = window.handle;
    Some(handle_focus(manager, handle))
}

fn move_focus_common_vars<F>(func: F, manager: &mut Manager, val: i32) -> Option<bool>
where
    F: Fn(&mut Manager, i32, WindowHandle, &Option<Layout>, Vec<Window>) -> Option<bool>,
{
    let handle = manager.focused_window()?.handle;
    let w = manager.focused_workspace()?;
    let (tags, layout) = (w.tags.clone(), Some(w.layout.clone()));

    let for_active_workspace =
        |x: &Window| -> bool { helpers::intersect(&tags, &x.tags) && !x.is_unmanaged() };

    let to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
    func(manager, val, handle, &layout, to_reorder)
}

fn move_window_change(
    manager: &mut Manager,
    val: i32,
    mut handle: WindowHandle,
    layout: &Option<Layout>,
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
    manager.windows.append(&mut to_reorder);
    Some(handle_focus(manager, handle))
}

//val and layout aren't used which is a bit awkward
fn move_window_top(
    manager: &mut Manager,
    _val: i32,
    handle: WindowHandle,
    _layout: &Option<Layout>,
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

    manager.windows.append(&mut to_reorder);
    // focus follows the window if it was not already on top of the stack
    if index > 0 {
        return Some(handle_focus(manager, handle));
    }
    Some(true)
}

fn focus_window_change(
    manager: &mut Manager,
    val: i32,
    mut handle: WindowHandle,
    layout: &Option<Layout>,
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
    manager.windows.append(&mut to_reorder);
    Some(handle_focus(manager, handle))
}

fn focus_workspace_change(manager: &mut Manager, val: i32) -> Option<bool> {
    let current = manager.focused_workspace()?;
    let workspace =
        helpers::relative_find(&manager.workspaces, |w| w == current, val, true)?.clone();
    focus_handler::focus_workspace(manager, &workspace);
    if manager.focus_manager.behaviour == FocusBehaviour::Sloppy {
        let act = DisplayAction::MoveMouseOverPoint(workspace.xyhw.center());
        manager.actions.push_back(act);
    }
    let window = manager
        .windows
        .iter()
        .find(|w| workspace.is_displaying(w) && w.type_ == WindowType::Normal)?
        .clone();
    Some(handle_focus(manager, window.handle))
}

fn rotate_tag(manager: &mut Manager) -> Option<bool> {
    let workspace = manager
        .focus_manager
        .workspace_mut(&mut manager.workspaces)?;
    let _ = workspace.rotate_layout(&mut manager.tags);
    Some(true)
}

fn change_main_width(manager: &mut Manager, val: &Option<String>, factor: i8) -> Option<bool> {
    let workspace = manager
        .focus_manager
        .workspace_mut(&mut manager.workspaces)?;
    let delta: i8 = val.as_ref()?.parse().ok()?;
    workspace.change_main_width(&mut manager.tags, delta * factor);
    Some(true)
}

fn set_margin_multiplier(manager: &mut Manager, val: &Option<String>) -> Option<bool> {
    let margin_multiplier: f32 = val.as_ref()?.parse().ok()?;
    let ws = manager.focused_workspace_mut()?;
    ws.set_margin_multiplier(margin_multiplier);
    let tags = ws.tags.clone();
    if manager
        .windows
        .iter()
        .any(|w| w.type_ == WindowType::Normal)
    {
        let for_active_workspace = |x: &Window| -> bool {
            helpers::intersect(&tags, &x.tags) && x.type_ == WindowType::Normal
        };
        let mut to_apply_margin_multiplier =
            helpers::vec_extract(&mut manager.windows, for_active_workspace);
        for w in &mut to_apply_margin_multiplier {
            if let Some(ws) = manager.focused_workspace() {
                w.apply_margin_multiplier(ws.margin_multiplier());
            }
        }
        manager.windows.append(&mut to_apply_margin_multiplier);
    }
    Some(true)
}

fn handle_focus(manager: &mut Manager, handle: WindowHandle) -> bool {
    match manager.focus_manager.behaviour {
        FocusBehaviour::Sloppy => {
            let act = DisplayAction::MoveMouseOver(handle);
            manager.actions.push_back(act);
            true
        }
        _ => focus_handler::focus_window(manager, &handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, FocusBehaviour, Keybind, ScratchPad, Workspace};
    use crate::errors::Result;
    use crate::models::Tag;
    use crate::state::State;

    struct TestConfig;

    impl Config for TestConfig {
        fn mapped_bindings(&self) -> Vec<Keybind> {
            todo!()
        }
        fn create_list_of_tags(&self) -> Vec<String> {
            todo!()
        }
        fn workspaces(&self) -> Option<&[Workspace]> {
            todo!()
        }
        fn focus_behaviour(&self) -> FocusBehaviour {
            todo!()
        }
        fn mousekey(&self) -> &str {
            todo!()
        }
        fn disable_current_tag_swap(&self) -> bool {
            todo!()
        }
    }

    struct TestState;

    impl State for TestState {
        fn save(&self, manager: &Manager) -> Result<()> {
            todo!()
        }
        fn load(&self, manager: &mut Manager) {
            todo!()
        }
    }

    #[test]
    fn go_to_tag_should_return_false_if_no_screen_is_created() {
        let mut manager = Manager::default();
        let config = TestConfig;
        // no screen creation here
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("6".to_string())
        ));
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("2".to_string())
        ));
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("15".to_string())
        ),);
    }

    #[test]
    fn go_to_tag_should_create_at_least_one_tag_per_screen_no_more() {
        let mut manager = Manager::default();
        let config = TestConfig;
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        // no tag creation here but one tag per screen is created
        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("2".to_string())
        ));
        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("1".to_string())
        ));
        // we only have one tag per screen created automatically
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("3".to_string())
        ),);
    }

    #[test]
    fn go_to_tag_should_return_false_on_invalid_input() {
        let mut manager = Manager::default();
        let config = TestConfig;
        screen_create_handler::process(&mut manager, Screen::default());
        manager.tags = vec![
            Tag::new("A15"),
            Tag::new("B24"),
            Tag::new("C"),
            Tag::new("6D4"),
            Tag::new("E39"),
            Tag::new("F67"),
        ];
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("abc".to_string())
        ),);
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("ab45c".to_string())
        ));
        assert!(!process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &None
        ));
    }

    #[test]
    fn go_to_tag_should_go_to_tag_and_set_history() {
        let mut manager = Manager {
            tags: vec![
                Tag::new("A15"),
                Tag::new("B24"),
                Tag::new("C"),
                Tag::new("6D4"),
                Tag::new("E39"),
                Tag::new("F67"),
            ],
            ..Manager::default()
        };
        let config = TestConfig;
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());

        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("6".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(5));
        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("2".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(1));

        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("3".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(2));

        assert!(process(
            &mut manager,
            &TestState,
            &config,
            &Command::GotoTag,
            &Some("4".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(3));
        assert_eq!(
            manager.tag_index(&manager.focused_tag(1).unwrap_or_default()),
            Some(2)
        );
        assert_eq!(
            manager.tag_index(&manager.focused_tag(2).unwrap_or_default()),
            Some(1)
        );
        assert_eq!(
            manager.tag_index(&manager.focused_tag(3).unwrap_or_default()),
            Some(5)
        );
    }
}
