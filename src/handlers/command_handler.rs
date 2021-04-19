#![allow(clippy::wildcard_imports)]
#![allow(clippy::shadow_unrelated)]

// NOTE: there apears to be a clippy bug with shadow_unrelated and the (?) Operator
// allow shadow should be removed once it is resolved
// https://github.com/rust-lang/rust-clippy/issues/6563

use super::*;
use crate::config::Config;
use crate::display_action::DisplayAction;
use crate::layouts::Layout;
use crate::utils::{child_process::exec_shell, helpers};
use std::str::FromStr;

/* Please also update src/bin/leftwm-check if any of the following apply after your update:
 * - a command now requires a value
 * - a command no longer requires a value
 * - a new command is introduced that requires a value
 *  */
pub fn process(
    manager: &mut Manager,
    config: &Config,
    command: &Command,
    val: &Option<String>,
) -> bool {
    process_internal(manager, config, command, val).unwrap_or(false)
}

pub fn process_internal(
    mut manager: &mut Manager,
    config: &Config,
    command: &Command,
    val: &Option<String>,
) -> Option<bool> {
    match command {
        Command::Execute => {
            exec_shell(val.as_ref()?, &mut manager);
            None
        }

        Command::MoveToTag => move_to_tag(&val, manager),

        Command::MoveWindowUp => move_focus_common_vars(move_window_change, manager, -1),
        Command::MoveWindowDown => move_focus_common_vars(move_window_change, manager, 1),
        Command::MoveWindowTop => move_focus_common_vars(move_window_top, manager, 0),

        Command::GotoTag => goto_tag(manager, &val, config),

        Command::CloseWindow => close_window(manager),
        Command::SwapTags => swap_tags(manager),
        Command::MoveToLastWorkspace => move_to_last_workspace(manager),
        Command::NextLayout => next_layout(manager),
        Command::PreviousLayout => previous_layout(manager),

        Command::SetLayout => set_layout(&val, manager),

        Command::FloatingToTile => floating_to_tile(manager),

        Command::FocusNextTag => focus_next_tag(manager),
        Command::FocusPreviousTag => focus_previous_tag(manager),
        Command::FocusWindowUp => move_focus_common_vars(focus_window_change, manager, -1),
        Command::FocusWindowDown => move_focus_common_vars(focus_window_change, manager, 1),
        Command::FocusWorkspaceNext => focus_workspace_change(manager, 1),
        Command::FocusWorkspacePrevious => focus_workspace_change(manager, -1),

        Command::MouseMoveWindow => None,

        Command::SoftReload => {
            manager.soft_reload();
            None
        }
        Command::HardReload => {
            manager.hard_reload();
            None
        }

        Command::RotateTag => rotate_tag(manager),

        Command::IncreaseMainWidth => increase_main_width(manager, &val),
        Command::DecreaseMainWidth => decrease_main_width(manager, &val),
        Command::SetMarginMultiplier => set_margin_multiplier(manager, &val),
    }
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

    let window = manager.focused_window_mut()?;
    window.clear_tags();
    window.set_floating(false);
    window.tag(&tag.id);
    window.apply_margin_multiplier(margin_multiplier);
    let act = DisplayAction::SetWindowTags(window.handle, tag.id.clone());
    manager.actions.push_back(act);
    manager.sort_windows();
    Some(true)
}

fn goto_tag(manager: &mut Manager, val: &Option<String>, config: &Config) -> Option<bool> {
    let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
    let previous_tag = manager.tag_index(&manager.focused_tag(1).unwrap_or_default());

    let input_tag = val.as_ref()?.parse().ok()?;
    let mut destination_tag = input_tag;
    if !config.disable_current_tag_swap {
        destination_tag = match (current_tag, previous_tag, input_tag) {
            (Some(curr_tag), Some(prev_tag), inp_tag) if curr_tag + 1 == inp_tag => prev_tag + 1, // if current tag is the same as the destination tag, go to the previous tag instead
            (_, _, _) => input_tag, // go to the input tag tag
        };
    }
    Some(goto_tag_handler::process(manager, destination_tag))
}

fn focus_next_tag(manager: &mut Manager) -> Option<bool> {
    let current = manager.focused_tag(0)?;
    let mut index = manager.tags.iter().position(|x| x.id == current)? + 1;
    index += 1;
    if index > manager.tags.len() {
        index = 1;
    }
    Some(goto_tag_handler::process(manager, index))
}

fn focus_previous_tag(manager: &mut Manager) -> Option<bool> {
    let current = manager.focused_tag(0)?;
    let mut index = manager.tags.iter().position(|x| x.id == current)? + 1;
    index -= 1;
    if index < 1 {
        index = manager.tags.len();
    }
    Some(goto_tag_handler::process(manager, index))
}

fn swap_tags(manager: &mut Manager) -> Option<bool> {
    if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
        let hist_a = *manager.focused_workspace_history.get(0)?;
        let hist_b = *manager.focused_workspace_history.get(1)?;
        let mut temp = vec![];
        std::mem::swap(&mut manager.workspaces.get_mut(hist_a)?.tags, &mut temp);
        std::mem::swap(&mut manager.workspaces.get_mut(hist_b)?.tags, &mut temp);
        std::mem::swap(&mut manager.workspaces.get_mut(hist_a)?.tags, &mut temp);
        return Some(true);
    }
    if manager.workspaces.len() == 1 {
        let last = manager
            .focused_tag_history
            .get(1)
            .map(std::string::ToString::to_string)?;

        let tag_index = manager.tags.iter().position(|x| x.id == last)? + 1;
        return Some(goto_tag_handler::process(manager, tag_index));
    }
    None
}

fn close_window(manager: &mut Manager) -> Option<bool> {
    let window = manager.focused_window()?;
    if window.type_ != WindowType::Dock {
        let act = DisplayAction::KillWindow(window.handle);
        manager.actions.push_back(act);
    }
    None
}

fn move_to_last_workspace(manager: &mut Manager) -> Option<bool> {
    if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
        let index = *manager.focused_workspace_history.get(1)?;
        let wp_tags = &manager.workspaces.get(index)?.tags.clone();
        let window = manager.focused_window_mut()?;
        window.tags = vec![wp_tags.get(0)?.clone()];
        return Some(true);
    }
    None
}

fn next_layout(manager: &mut Manager) -> Option<bool> {
    let workspace = manager.focused_workspace_mut()?;
    workspace.next_layout();
    Some(true)
}

fn previous_layout(manager: &mut Manager) -> Option<bool> {
    let workspace = manager.focused_workspace_mut()?;
    workspace.prev_layout();
    Some(true)
}

fn set_layout(val: &Option<String>, manager: &mut Manager) -> Option<bool> {
    let layout = Layout::from_str(val.as_ref()?).ok()?;
    let workspace = manager.focused_workspace_mut()?;
    workspace.set_layout(layout);
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
    let act = DisplayAction::MoveMouseOver(window.handle);
    manager.actions.push_back(act);
    Some(true)
}

fn move_focus_common_vars<F>(func: F, manager: &mut Manager, val: i32) -> Option<bool>
where
    F: Fn(&mut Manager, i32, WindowHandle, &Option<Layout>, Vec<Window>) -> bool,
{
    let handle = manager.focused_window()?.handle;
    let w = manager.focused_workspace()?;
    let (tags, layout) = (w.tags.clone(), Some(w.layout.clone()));

    let for_active_workspace =
        |x: &Window| -> bool { helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock };

    let to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
    Some(func(manager, val, handle, &layout, to_reorder))
}

fn move_window_change(
    manager: &mut Manager,
    val: i32,
    handle: WindowHandle,
    layout: &Option<Layout>,
    mut to_reorder: Vec<Window>,
) -> bool {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    let mut act = DisplayAction::MoveMouseOver(handle);
    if let Some(crate::layouts::Layout::Monocle) = layout {
        // For Monocle we want to also move windows up/down
        // Not the best solution but results
        // in desired behaviour
        let new_handle = match helpers::relative_find(&to_reorder, is_handle, -val) {
            Some(h) => h.handle,
            None => return false,
        };
        helpers::cycle_vec(&mut to_reorder, val);
        act = DisplayAction::MoveMouseOver(new_handle);
    } else if let Some(crate::layouts::Layout::MainAndDeck) = layout {
        if to_reorder.len() > 1 {
            let main = to_reorder.remove(0);
            if main.handle != handle {
                let new_handle = match helpers::relative_find(&to_reorder, is_handle, -val) {
                    Some(h) => h.handle,
                    None => return false,
                };
                act = DisplayAction::MoveMouseOver(new_handle);
            }
            helpers::cycle_vec(&mut to_reorder, val);
            to_reorder.insert(0, main);
        }
    } else {
        let _ = helpers::reorder_vec(&mut to_reorder, is_handle, val);
    }
    manager.windows.append(&mut to_reorder);
    manager.actions.push_back(act);
    true
}

//val and layout aren't used which is a bit awkward
fn move_window_top(
    manager: &mut Manager,
    _val: i32,
    handle: WindowHandle,
    _layout: &Option<Layout>,
    mut to_reorder: Vec<Window>,
) -> bool {
    // Moves the selected window at index 0 of the window list.
    // If the selected window is already at index 0, it is sent to index 1.
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    let list = &mut to_reorder;
    let len = list.len();
    let (index, item) = match list.iter().enumerate().find(|&x| is_handle(&x.1)) {
        Some(x) => (x.0, x.1.clone()),
        None => return false,
    };
    list.remove(index);
    let mut new_index: usize = match index {
        0 => 1,
        _ => 0,
    };
    if new_index >= len {
        new_index -= len
    }
    list.insert(new_index, item);

    manager.windows.append(&mut to_reorder);
    // focus follows the window if it was not already on top of the stack
    if index > 0 {
        let act = DisplayAction::MoveMouseOver(handle);
        manager.actions.push_back(act);
    }
    true
}

fn focus_window_change(
    manager: &mut Manager,
    val: i32,
    handle: WindowHandle,
    layout: &Option<Layout>,
    mut to_reorder: Vec<Window>,
) -> bool {
    let is_handle = |x: &Window| -> bool { x.handle == handle };
    if let Some(crate::layouts::Layout::Monocle) = layout {
        let new_handle = match helpers::relative_find(&to_reorder, is_handle, -val) {
            Some(h) => h.handle,
            None => return false,
        };
        helpers::cycle_vec(&mut to_reorder, val);
        let act = DisplayAction::MoveMouseOver(new_handle);
        manager.actions.push_back(act);
    } else if let Some(crate::layouts::Layout::MainAndDeck) = layout {
        //Only change focus on first 2 windows
        let window_group = &to_reorder[..2];
        if let Some(new_focused) = helpers::relative_find(&window_group, is_handle, val) {
            let act = DisplayAction::MoveMouseOver(new_focused.handle);
            manager.actions.push_back(act);
        }
    } else if let Some(new_focused) = helpers::relative_find(&to_reorder, is_handle, val) {
        let act = DisplayAction::MoveMouseOver(new_focused.handle);
        manager.actions.push_back(act);
    }
    manager.windows.append(&mut to_reorder);
    true
}

fn focus_workspace_change(manager: &mut Manager, val: i32) -> Option<bool> {
    let current = manager.focused_workspace()?;
    let mut index = manager
        .workspaces
        .iter()
        .enumerate()
        .find(|&x| x.1 == current)?
        .0 as i32;
    let len = manager.workspaces.len() as i32;
    index += val;
    if index < 0 {
        index = len - 1;
    }
    if index >= len {
        index = 0;
    }
    let workspace = manager.workspaces.get(index as usize)?.clone();
    focus_handler::focus_workspace(manager, &workspace);
    let act = DisplayAction::MoveMouseOverPoint(workspace.xyhw.center());
    manager.actions.push_back(act);
    let window = manager
        .windows
        .iter()
        .find(|w| workspace.is_displaying(w) && w.type_ == WindowType::Normal)?
        .clone();
    focus_handler::move_cursor_over(manager, &window);
    let act = DisplayAction::MoveMouseOver(window.handle);
    manager.actions.push_back(act);
    Some(true)
}

fn rotate_tag(manager: &mut Manager) -> Option<bool> {
    let workspace = manager.focused_workspace_mut()?;
    let _ = workspace.rotate_layout();
    Some(true)
}

fn increase_main_width(manager: &mut Manager, val: &Option<String>) -> Option<bool> {
    let workspace = manager.focused_workspace_mut()?;
    let delta: u8 = val.as_ref()?.parse().ok()?;
    workspace.increase_main_width(delta);
    Some(true)
}

fn decrease_main_width(manager: &mut Manager, val: &Option<String>) -> Option<bool> {
    let workspace = manager.focused_workspace_mut()?;
    let delta: u8 = val.as_ref()?.parse().ok()?;
    workspace.decrease_main_width(delta);
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
        to_apply_margin_multiplier.iter_mut().for_each(|w| {
            if let Some(ws) = manager.focused_workspace() {
                w.apply_margin_multiplier(ws.margin_multiplier())
            }
        });
        manager.windows.append(&mut to_apply_margin_multiplier);
    }
    Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TagModel;
    #[test]
    fn go_to_tag_should_return_false_if_no_screen_is_created() {
        let mut manager = Manager::default();
        let config = Config::default();
        // no screen creation here
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("6".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("2".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("15".to_string())
            ),
            false
        );
    }

    #[test]
    fn go_to_tag_should_create_at_least_one_tag_per_screen_no_more() {
        let mut manager = Manager::default();
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());
        // no tag creation here but one tag per screen is created
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            &Some("2".to_string())
        ));
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            &Some("1".to_string())
        ));
        // we only have one tag per screen created automatically
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("3".to_string())
            ),
            false
        );
    }

    #[test]
    fn go_to_tag_should_return_false_on_invalid_input() {
        let mut manager = Manager::default();
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        manager.tags = vec![
            TagModel::new("A15"),
            TagModel::new("B24"),
            TagModel::new("C"),
            TagModel::new("6D4"),
            TagModel::new("E39"),
            TagModel::new("F67"),
        ];
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("abc".to_string())
            ),
            false
        );
        assert_eq!(
            process(
                &mut manager,
                &config,
                &Command::GotoTag,
                &Some("ab45c".to_string())
            ),
            false
        );
        assert_eq!(
            process(&mut manager, &config, &Command::GotoTag, &None),
            false
        );
    }

    #[test]
    fn go_to_tag_should_go_to_tag_and_set_history() {
        let mut manager = Manager {
            tags: vec![
                TagModel::new("A15"),
                TagModel::new("B24"),
                TagModel::new("C"),
                TagModel::new("6D4"),
                TagModel::new("E39"),
                TagModel::new("F67"),
            ],
            ..Manager::default()
        };
        let config = Config::default();
        screen_create_handler::process(&mut manager, Screen::default());
        screen_create_handler::process(&mut manager, Screen::default());

        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            &Some("6".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(5));
        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            &Some("2".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(1));

        assert!(process(
            &mut manager,
            &config,
            &Command::GotoTag,
            &Some("3".to_string())
        ));
        let current_tag = manager.tag_index(&manager.focused_tag(0).unwrap_or_default());
        assert_eq!(current_tag, Some(2));

        assert!(process(
            &mut manager,
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
