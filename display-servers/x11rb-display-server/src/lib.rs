//! x11rb backend for leftwm
//! TODO: Error Handling
//! TODO: Refactoring

use leftwm_core::{
    models::{Screen, TagId, WindowHandle, WindowState},
    Config, DisplayAction, DisplayEvent, DisplayServer, Mode, Window, Workspace,
};
use x11rb::protocol::xproto;

use crate::xwrap::XWrap;

mod event_translate;
mod xatom;
mod xwrap;

pub struct X11rbDisplayServer {
    xw: XWrap,
    root: xproto::Window,
    initial_events: Vec<DisplayEvent>,
}

impl DisplayServer for X11rbDisplayServer {
    fn new(config: &impl Config) -> Self {
        let mut xwrap = XWrap::new();

        xwrap.init(config);

        let root = xwrap.get_default_root();
        let mut instance = Self {
            xw: xwrap,
            root,
            initial_events: Vec::new(),
        };
        instance.initial_events = instance.initial_events(config);

        instance
    }

    fn load_config(
        &mut self,
        config: &impl Config,
        focused: Option<&Option<WindowHandle>>,
        windows: &[leftwm_core::Window],
    ) {
        self.xw.load_config(config, focused, windows);
    }

    fn update_windows(&self, windows: Vec<&Window>) {
        for window in &windows {
            self.xw.update_window(window);
        }
    }

    fn update_workspaces(&self, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            self.xw.set_current_desktop(focused.tag);
        }
    }

    fn get_next_events(&mut self) -> Vec<leftwm_core::DisplayEvent> {
        let mut events = std::mem::take(&mut self.initial_events);

        while let Some(ev) = self.xw.poll_next_event() {
            if let Some(ev) = event_translate::translate(ev, &mut self.xw) {
                events.push(ev);
            }
        }

        for event in &events {
            if let DisplayEvent::WindowDestroy(WindowHandle::X11rbHandle(w)) = event {
                self.xw.force_unmapped(*w);
            }
        }

        events
    }

    fn execute_action(&mut self, act: DisplayAction) -> Option<DisplayEvent> {
        tracing::trace!("DisplayAction: {:?}", act);
        let xw = &mut self.xw;
        let event: Option<DisplayEvent> = match act {
            DisplayAction::KillWindow(h) => from_kill_window(xw, h),
            DisplayAction::AddedWindow(h, f, fm) => from_added_window(xw, h, f, fm),
            DisplayAction::MoveMouseOver(h, f) => from_move_mouse_over(xw, h, f),
            DisplayAction::MoveMouseOverPoint(p) => from_move_mouse_over_point(xw, p),
            DisplayAction::DestroyedWindow(h) => from_destroyed_window(xw, h),
            DisplayAction::Unfocus(h, f) => from_unfocus(xw, h, f),
            DisplayAction::ReplayClick(h, b) => from_replay_click(xw, h, b.try_into().unwrap_or(0)),
            DisplayAction::SetState(h, t, s) => from_set_state(xw, h, t, s),
            DisplayAction::SetWindowOrder(fs, ws) => from_set_window_order(xw, fs, ws),
            DisplayAction::MoveToTop(h) => from_move_to_top(xw, h),
            DisplayAction::ReadyToMoveWindow(h) => from_ready_to_move_window(xw, h),
            DisplayAction::ReadyToResizeWindow(h) => from_ready_to_resize_window(xw, h),
            DisplayAction::SetCurrentTags(t) => from_set_current_tags(xw, t),
            DisplayAction::SetWindowTag(h, t) => from_set_window_tag(xw, h, t),
            DisplayAction::ConfigureXlibWindow(w) => from_configure_xlib_window(xw, &w),

            DisplayAction::WindowTakeFocus {
                window,
                previous_window,
            } => from_window_take_focus(xw, &window, &previous_window),

            DisplayAction::FocusWindowUnderCursor => from_focus_window_under_cursor(xw),
            DisplayAction::NormalMode => from_normal_mode(xw),
        };
        if event.is_some() {
            tracing::trace!("DisplayEvent: {:?}", event);
        }
        event
    }

    fn wait_readable(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()>>> {
        todo!()
    }

    fn flush(&self) {
        self.xw.flush();
    }

    fn generate_verify_focus_event(&self) -> Option<leftwm_core::DisplayEvent> {
        let handle = self.xw.get_cursor_window().ok()?;
        Some(DisplayEvent::VerifyFocusedAt(handle))
    }
}

impl X11rbDisplayServer {
    fn initial_events(&self, config: &impl Config) -> Vec<DisplayEvent> {
        let mut events = vec![];
        if let Some(workspaces) = config.workspaces() {
            let screens = self.xw.get_screens();
            for (i, wsc) in workspaces.iter().enumerate() {
                let mut screen = Screen::from(wsc);
                screen.root = WindowHandle::X11rbHandle(self.root);
                // If there is a screen corresponding to the given output, create the workspace
                match screens.iter().find(|i| i.output == wsc.output) {
                    Some(output_match) => {
                        if wsc.relative.unwrap_or(false) {
                            screen.bbox.add(output_match.bbox);
                        }
                        screen.id = Some(i + 1);
                    }
                    None => continue,
                }
                let e = DisplayEvent::ScreenCreate(screen);
                events.push(e);
            }

            let auto_derive_workspaces: bool = if config.auto_derive_workspaces() {
                true
            } else if events.is_empty() {
                tracing::warn!("No Workspace in Workspace config matches connected screen. Falling back to \"auto_derive_workspaces: true\".");
                true
            } else {
                false
            };

            let mut next_id = workspaces.len() + 1;

            // If there is no hardcoded workspace layout, add every screen not mentioned in the config.
            if auto_derive_workspaces {
                screens
                    .iter()
                    .filter(|screen| !workspaces.iter().any(|wsc| wsc.output == screen.output))
                    .for_each(|screen| {
                        let mut s = screen.clone();
                        s.id = Some(next_id);
                        next_id += 1;
                        events.push(DisplayEvent::ScreenCreate(s));
                    });
            }
        }

        // Tell manager about existing windows.
        events.append(&mut self.find_all_windows());

        events
    }

    fn find_all_windows(&self) -> Vec<DisplayEvent> {
        let mut all: Vec<DisplayEvent> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => handles.into_iter().for_each(|handle| {
                let Ok(attrs) = self.xw.get_window_attrs(handle) else {
                    return;
                };
                let Some((state, _)) = self.xw.get_wm_state(handle) else {
                    return;
                };
                if attrs.map_state == xproto::MapState::VIEWABLE
                    || state == xatom::WMStateWindowState::Iconic
                {
                    if let Some(event) = self.xw.setup_window(handle) {
                        all.push(event);
                    }
                }
            }),
            Err(err) => {
                println!("ERROR: {err}");
            }
        }
        all
    }
}

// Display actions.
fn from_kill_window(xw: &mut XWrap, handle: WindowHandle) -> Option<DisplayEvent> {
    xw.kill_window(&handle);
    None
}

fn from_added_window(
    xw: &mut XWrap,
    handle: WindowHandle,
    floating: bool,
    follow_mouse: bool,
) -> Option<DisplayEvent> {
    xw.setup_managed_window(handle, floating, follow_mouse)
}

fn from_move_mouse_over(xw: &mut XWrap, handle: WindowHandle, force: bool) -> Option<DisplayEvent> {
    match (handle, xw.get_cursor_window()) {
        (WindowHandle::X11rbHandle(window), Ok(WindowHandle::X11rbHandle(cursor_window)))
            if force || cursor_window != window =>
        {
            _ = xw.move_cursor_to_window(window);
        }
        _ => {}
    }
    None
}

fn from_move_mouse_over_point(xw: &mut XWrap, point: (i32, i32)) -> Option<DisplayEvent> {
    _ = xw.move_cursor_to_point(point);
    None
}

fn from_destroyed_window(xw: &mut XWrap, handle: WindowHandle) -> Option<DisplayEvent> {
    xw.teardown_managed_window(&handle, true);
    None
}

fn from_unfocus(
    xw: &mut XWrap,
    handle: Option<WindowHandle>,
    floating: bool,
) -> Option<DisplayEvent> {
    xw.unfocus(handle, floating);
    None
}

fn from_replay_click(xw: &mut XWrap, handle: WindowHandle, button: xproto::Button) -> Option<DisplayEvent> {
    if let WindowHandle::X11rbHandle(handle) = handle {
        xw.replay_click(handle, button);
    }
    None
}

fn from_set_state(
    xw: &mut XWrap,
    handle: WindowHandle,
    toggle_to: bool,
    window_state: WindowState,
) -> Option<DisplayEvent> {
    // TODO: impl from for windowstate and xlib::Atom
    let state = match window_state {
        WindowState::Modal => xw.atoms.NetWMStateModal,
        WindowState::Sticky => xw.atoms.NetWMStateSticky,
        WindowState::MaximizedVert => xw.atoms.NetWMStateMaximizedVert,
        WindowState::MaximizedHorz => xw.atoms.NetWMStateMaximizedHorz,
        WindowState::Shaded => xw.atoms.NetWMStateShaded,
        WindowState::SkipTaskbar => xw.atoms.NetWMStateSkipTaskbar,
        WindowState::SkipPager => xw.atoms.NetWMStateSkipPager,
        WindowState::Hidden => xw.atoms.NetWMStateHidden,
        WindowState::Fullscreen => xw.atoms.NetWMStateFullscreen,
        WindowState::Above => xw.atoms.NetWMStateAbove,
        WindowState::Below => xw.atoms.NetWMStateBelow,
    };
    xw.set_state(handle, toggle_to, state);
    None
}

fn from_set_window_order(
    xw: &mut XWrap,
    fullscreen: Vec<WindowHandle>,
    windows: Vec<WindowHandle>,
) -> Option<DisplayEvent> {
    // Unmanaged windows.
    let unmanaged: Vec<WindowHandle> = xw
        .get_all_windows()
        .unwrap_or_default()
        .iter()
        .filter(|&w| *w != xw.get_default_root())
        .map(|&w| WindowHandle::X11rbHandle(w))
        .filter(|&h| !windows.iter().any(|&w| w == h) || !fullscreen.iter().any(|&w| w == h))
        .collect();
    let all: Vec<WindowHandle> = [fullscreen, unmanaged, windows].concat();
    xw.restack(all);
    None
}

fn from_move_to_top(xw: &mut XWrap, handle: WindowHandle) -> Option<DisplayEvent> {
    xw.move_to_top(&handle);
    None
}

fn from_ready_to_move_window(xw: &mut XWrap, handle: WindowHandle) -> Option<DisplayEvent> {
    xw.set_mode(Mode::ReadyToMove(handle));
    None
}

fn from_ready_to_resize_window(xw: &mut XWrap, handle: WindowHandle) -> Option<DisplayEvent> {
    xw.set_mode(Mode::ReadyToResize(handle));
    None
}

fn from_set_current_tags(xw: &mut XWrap, tag: Option<TagId>) -> Option<DisplayEvent> {
    xw.set_current_desktop(tag);
    None
}

fn from_set_window_tag(
    xw: &mut XWrap,
    handle: WindowHandle,
    tag: Option<TagId>,
) -> Option<DisplayEvent> {
    if let WindowHandle::X11rbHandle(window) = handle {
        let tag = tag?;
        xw.set_window_desktop(window, &tag);
    }
    None
}

fn from_configure_xlib_window(xw: &mut XWrap, window: &Window) -> Option<DisplayEvent> {
    xw.configure_window(window);
    None
}

fn from_window_take_focus(
    xw: &mut XWrap,
    window: &Window,
    previous_window: &Option<Window>,
) -> Option<DisplayEvent> {
    xw.window_take_focus(window, previous_window.as_ref());
    None
}

fn from_focus_window_under_cursor(xw: &mut XWrap) -> Option<DisplayEvent> {
    if let Ok(mut window) = xw.get_cursor_window() {
        if window == WindowHandle::XlibHandle(0) {
            window = xw.get_default_root_handle();
        }
        return Some(DisplayEvent::WindowTakeFocus(window));
    }
    let point = xw.get_cursor_point().ok()?;
    let evt = DisplayEvent::MoveFocusTo(point.0, point.1);
    Some(evt)
}

fn from_normal_mode(xw: &mut XWrap) -> Option<DisplayEvent> {
    xw.set_mode(Mode::Normal);
    None
}
