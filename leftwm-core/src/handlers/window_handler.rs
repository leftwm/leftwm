use super::{Manager, Window, WindowChange, WindowType, Workspace};
use crate::config::{Config, ScratchPad};
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
use crate::models::{Size, WindowHandle, WindowState, Xyhw, XyhwBuilder};
use crate::utils::helpers;
use crate::{child_process::exec_shell, models::FocusBehaviour};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn window_created_handler(&mut self, mut window: Window, x: i32, y: i32) -> bool {
        //don't add the window if the manager already knows about it
        if self.state.windows.iter().any(|w| w.handle == window.handle) {
            return false;
        }

        let mut is_first = false;
        //Random value
        let mut layout: Layout = Layout::MainAndVertStack;
        let is_scratchpad = is_scratchpad(self, &window);
        setup_window(
            self,
            &mut window,
            x,
            y,
            is_scratchpad,
            &mut layout,
            &mut is_first,
        );
        insert_window(self, &mut window, is_scratchpad, layout);

        let follow_mouse = self.state.focus_manager.focus_new_windows
            || self.state.focus_manager.behaviour == FocusBehaviour::Sloppy;
        //let the DS know we are managing this window
        let act = DisplayAction::AddedWindow(window.handle, follow_mouse);
        self.state.actions.push_back(act);

        //let the DS know the correct desktop to find this window
        if !window.tags.is_empty() {
            let act = DisplayAction::SetWindowTags(window.handle, window.tags[0].clone());
            self.state.actions.push_back(act);
        }

        // tell the WM the new display order of the windows
        //new windows should be on the top of the stack
        self.sort_windows();

        if self.state.focus_manager.focus_new_windows || is_first {
            self.focus_window(&window.handle);
        }

        if let Some(cmd) = &self.state.config.on_new_window_cmd() {
            exec_shell(cmd, self);
        }

        true
    }

    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn window_destroyed_handler(&mut self, handle: &WindowHandle) -> bool {
        //Find the next or previous window on the workspace
        let new_handle = self.get_next_or_previous(handle);
        self.state
            .focus_manager
            .tags_last_window
            .retain(|_, h| h != handle);
        self.state.windows.retain(|w| &w.handle != handle);

        //make sure the workspaces do not draw on the docks
        self.update_workspace_avoid_list();

        let focused = self.state.focus_manager.window_history.get(0);
        //make sure focus is recalculated if we closed the currently focused window
        if focused == Some(&Some(*handle)) {
            if self.state.focus_manager.behaviour == FocusBehaviour::Sloppy {
                let act = DisplayAction::FocusWindowUnderCursor;
                self.state.actions.push_back(act);
            } else if let Some(h) = new_handle {
                self.focus_window(&h);
            }
        }

        true
    }

    pub fn window_changed_handler(&mut self, change: WindowChange) -> bool {
        let mut changed = false;
        let strut_changed = change.strut.is_some();
        if let Some(w) = self
            .state
            .windows
            .iter_mut()
            .find(|w| w.handle == change.handle)
        {
            log::debug!("WINDOW CHANGED {:?} {:?}", &w, change);
            changed = change.update(w);
            if w.type_ == WindowType::Dock {
                self.update_workspace_avoid_list();
                //don't left changes from docks re-render the worker. This will result in an
                //infinite loop. Just be patient a rerender will occur.
            }
        }
        if strut_changed {
            self.update_static();
        }
        changed
    }

    pub fn update_workspace_avoid_list(&mut self) {
        let mut avoid = vec![];
        self.state
            .windows
            .iter()
            .filter(|w| w.type_ == WindowType::Dock)
            .filter_map(|w| w.strut.map(|strut| (w.handle, strut)))
            .for_each(|(handle, to_avoid)| {
                log::debug!("AVOID STRUT:[{:?}] {:?}", handle, to_avoid);
                avoid.push(to_avoid);
            });
        for ws in &mut self.state.workspaces {
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

    //Find the next or previous window on the workspace
    pub fn get_next_or_previous(&mut self, handle: &WindowHandle) -> Option<WindowHandle> {
        if self.state.focus_manager.behaviour != FocusBehaviour::Sloppy {
            let ws = self.focused_workspace().cloned()?;
            let for_active_workspace = |x: &Window| -> bool { ws.is_managed(x) };
            let mut windows = helpers::vec_extract(&mut self.state.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { &x.handle == handle };
            let p = helpers::relative_find(&windows, is_handle, -1, false);
            let new_handle = helpers::relative_find(&windows, is_handle, 1, false)
                .or(p) //Backup
                .map(|w| w.handle);
            self.state.windows.append(&mut windows);
            return new_handle;
        }
        None
    }
}

impl Window {
    pub fn snap_to_workspace(&mut self, workspace: &Workspace) -> bool {
        self.set_floating(false);

        //we are reparenting
        if self.tags != workspace.tags {
            self.tags = workspace.tags.clone();
            let mut offset = self.get_floating_offsets().unwrap_or_default();
            let mut start_loc = self.start_loc.unwrap_or_default();
            let x = offset.x() + self.normal.x();
            let y = offset.y() + self.normal.y();
            offset.set_x(x - workspace.xyhw.x());
            offset.set_y(y - workspace.xyhw.y());
            self.set_floating_offsets(Some(offset));

            let x = start_loc.x() + self.normal.x();
            let y = start_loc.y() + self.normal.y();
            start_loc.set_x(x - workspace.xyhw.x());
            start_loc.set_y(y - workspace.xyhw.y());
            self.start_loc = Some(start_loc);
        }
        true
    }
}

fn setup_window<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    window: &mut Window,
    x: i32,
    y: i32,
    is_scratchpad: bool,
    layout: &mut Layout,
    is_first: &mut bool,
) {
    //When adding a window we add to the workspace under the cursor, This isn't necessarily the
    //focused workspace. If the workspace is empty, it might not have received focus. This is so
    //the workspace that has windows on its is still active not the empty workspace.
    let ws: Option<&Workspace> = manager
        .state
        .workspaces
        .iter()
        .find(|ws| {
            ws.xyhw.contains_point(x, y)
                && manager.state.focus_manager.behaviour == FocusBehaviour::Sloppy
        })
        .or_else(|| manager.focused_workspace()); //backup plan

    if let Some(ws) = ws {
        let for_active_workspace =
            |x: &Window| -> bool { helpers::intersect(&ws.tags, &x.tags) && !x.is_unmanaged() };
        *is_first = !manager
            .state
            .windows
            .iter()
            .any(|w| for_active_workspace(w));
        window.tags = ws.tags.clone();
        *layout = ws.layout;

        if is_scratchpad {
            window.set_floating(true);
            if let Some((scratchpad_name, _)) = manager
                .state
                .active_scratchpads
                .iter()
                .find(|(_, &id)| id == window.pid)
            {
                if let Some(s) = manager
                    .state
                    .scratchpads
                    .iter()
                    .find(|s| *scratchpad_name == s.name)
                {
                    let new_float_exact = scratchpad_xyhw(&ws.xyhw, s);
                    window.normal = ws.xyhw;
                    window.set_floating_exact(new_float_exact);
                }
            }
        }
        if window.type_ == WindowType::Normal {
            window.apply_margin_multiplier(ws.margin_multiplier);
        }
        //if dialog, center in workspace
        if window.type_ == WindowType::Dialog {
            window.set_floating(true);
            let new_float_exact = ws.center_halfed();
            window.normal = ws.xyhw;
            window.set_floating_exact(new_float_exact);
        }
        if window.type_ == WindowType::Splash {
            if let Some(requested) = window.requested {
                window.normal = ws.xyhw;
                requested.update_window_floating(window);
                let mut xhyw = window.get_floating_offsets().unwrap_or_default();
                xhyw.center_relative(ws.xyhw, window.border, window.requested);
                window.set_floating_offsets(Some(xhyw));
            } else {
                window.set_floating(true);
                let new_float_exact = ws.center_halfed();
                window.normal = ws.xyhw;
                window.set_floating_exact(new_float_exact);
            }
        }
    } else {
        window.tags = vec![manager.state.tags[0].id.clone()];
        if is_scratchpad {
            window.tag("NSP");
            window.set_floating(true);
        }
    }

    if let Some(parent) = find_transient_parent(manager, window) {
        window.set_floating(true);
        let new_float_exact = parent.calculated_xyhw().center_halfed();
        window.normal = parent.normal;
        window.set_floating_exact(new_float_exact);
    }

    window.update_for_theme(&manager.state.config);
}

fn insert_window<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    window: &mut Window,
    is_scratchpad: bool,
    layout: Layout,
) {
    // If the tag contains a fullscreen window, minimize it
    let for_active_workspace =
        |x: &Window| -> bool { helpers::intersect(&window.tags, &x.tags) && !x.is_unmanaged() };
    let mut was_fullscreen = false;
    if let Some(fsw) = manager
        .state
        .windows
        .iter_mut()
        .find(|w| for_active_workspace(w) && w.is_fullscreen())
    {
        let act =
            DisplayAction::SetState(fsw.handle, !fsw.is_fullscreen(), WindowState::Fullscreen);
        manager.state.actions.push_back(act);
        was_fullscreen = true;
    }

    if matches!(layout, Layout::Monocle | Layout::MainAndDeck) && window.type_ == WindowType::Normal
    {
        let mut to_reorder = helpers::vec_extract(&mut manager.state.windows, for_active_workspace);
        if layout == Layout::Monocle || to_reorder.is_empty() {
            if was_fullscreen {
                let act = DisplayAction::SetState(
                    window.handle,
                    !window.is_fullscreen(),
                    WindowState::Fullscreen,
                );
                manager.state.actions.push_back(act);
            }
            to_reorder.insert(0, window.clone());
        } else {
            to_reorder.insert(1, window.clone());
        }
        manager.state.windows.append(&mut to_reorder);
    } else if window.type_ == WindowType::Dialog
        || window.type_ == WindowType::Splash
        || is_scratchpad
    {
        //Slow
        manager.state.windows.insert(0, window.clone());
    } else {
        manager.state.windows.push(window.clone());
    }
}

fn is_scratchpad<C: Config, SERVER: DisplayServer>(
    manager: &Manager<C, SERVER>,
    window: &Window,
) -> bool {
    manager
        .state
        .active_scratchpads
        .iter()
        .any(|(_, &id)| window.pid == id)
}

fn find_transient_parent<'w, C: Config, SERVER: DisplayServer>(
    manager: &'w Manager<C, SERVER>,
    window: &Window,
) -> Option<&'w Window> {
    let mut transient = window.transient?;
    loop {
        transient = if let Some(found) = manager
            .state
            .windows
            .iter()
            .find(|x| x.handle == transient)
            .and_then(|x| x.transient)
        {
            found
        } else {
            return manager.state.windows.iter().find(|x| x.handle == transient);
        };
    }
}

// Get size and position of scratchpad from config and workspace size
pub fn scratchpad_xyhw(xyhw: &Xyhw, scratch_pad: &ScratchPad) -> Xyhw {
    let x_sane = sane_dimension(scratch_pad.x, 0.25, xyhw.w());
    let y_sane = sane_dimension(scratch_pad.y, 0.25, xyhw.h());
    let height_sane = sane_dimension(scratch_pad.height, 0.50, xyhw.h());
    let width_sane = sane_dimension(scratch_pad.width, 0.50, xyhw.w());

    XyhwBuilder {
        x: xyhw.x() + x_sane,
        y: xyhw.y() + y_sane,
        h: height_sane,
        w: width_sane,
        ..XyhwBuilder::default()
    }
    .into()
}

fn sane_dimension(config_value: Option<Size>, default_percent: f32, max_pixel: i32) -> i32 {
    match config_value {
        Some(size) => match size {
            Size::Percentage(percentage) if (0.0..0.9).contains(&percentage) => {
                size.into_absolute(100.0) as i32 * max_pixel / 100
            }
            Size::Pixel(pixel) if (0..(max_pixel as f32 * 0.9) as i32).contains(&pixel) => pixel,
            _ => (default_percent * max_pixel as f32) as i32,
        },
        _ => (default_percent * max_pixel as f32) as i32,
    }
}
