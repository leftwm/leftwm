//! x11rb backend for leftwm
//! TODO: Error Handling
//! TODO: Refactoring

use leftwm_core::{
    models::{Handle, Screen, TagId, WindowHandle, WindowState},
    Config, DisplayAction, DisplayEvent, DisplayServer, Mode, Window, Workspace,
};
use serde::{Deserialize, Serialize};
use x11rb::protocol::xproto;

use crate::xwrap::XWrap;
use error::Result;

mod error;
mod event_translate;
mod xatom;
mod xwrap;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct X11rbWindowHandle(xproto::Window);
impl Handle for X11rbWindowHandle {}

pub struct X11rbDisplayServer {
    xw: XWrap,
    root: xproto::Window,
    initial_events: Vec<DisplayEvent<X11rbWindowHandle>>,
}

impl DisplayServer<X11rbWindowHandle> for X11rbDisplayServer {
    fn new(config: &impl Config) -> Self {
        let mut xwrap = XWrap::new();

        xwrap.init(config).expect("XWrap initialisation failed.");

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
        focused: Option<&Option<WindowHandle<X11rbWindowHandle>>>,
        windows: &[leftwm_core::Window<X11rbWindowHandle>],
    ) {
        if let Err(e) = self.xw.load_config(config, focused, windows) {
            tracing::error!("Error when loading config: {}", e);
        }
    }

    fn update_windows(&self, windows: Vec<&Window<X11rbWindowHandle>>) {
        for window in &windows {
            if let Err(e) = self.xw.update_window(window) {
                tracing::error!("Error when updating window {:?}: {}", window, e);
            }
        }
    }

    fn update_workspaces(&self, focused: Option<&Workspace>) {
        if let Some(focused) = focused {
            if let Err(e) = self.xw.set_current_desktop(focused.tag) {
                tracing::error!("Error when setting current desktop to {:?}: {}", focused, e);
            }
        }
    }

    fn get_next_events(&mut self) -> Vec<leftwm_core::DisplayEvent<X11rbWindowHandle>> {
        let mut events = std::mem::take(&mut self.initial_events);

        loop {
            match self.xw.poll_next_event() {
                Ok(Some(ev)) => {
                    if let Some(ev) = event_translate::translate(ev, &mut self.xw) {
                        events.push(ev);
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    tracing::error!("An error occurred when polling for events. {:?}", e);
                    break;
                }
            }
        }

        for event in &events {
            if let DisplayEvent::WindowDestroy(WindowHandle(X11rbWindowHandle(w))) = event {
                if let Err(e) = self.xw.force_unmapped(*w) {
                    tracing::error!(error = ?e, "Error when forcing unmapping of window {}", w);
                };
            }
        }

        events
    }

    fn execute_action(
        &mut self,
        act: DisplayAction<X11rbWindowHandle>,
    ) -> Option<DisplayEvent<X11rbWindowHandle>> {
        tracing::trace!("DisplayAction: {:?}", act);
        let xw = &mut self.xw;
        let event: Result<Option<DisplayEvent<X11rbWindowHandle>>> = match act.clone() {
            DisplayAction::KillWindow(h) => from_kill_window(xw, h),
            DisplayAction::AddedWindow(h, f, fm) => xw.setup_managed_window(h, f, fm),
            DisplayAction::MoveMouseOver(h, f) => from_move_mouse_over(xw, h, f),
            DisplayAction::MoveMouseOverPoint(p) => from_move_mouse_over_point(xw, p),
            DisplayAction::DestroyedWindow(h) => from_destroyed_window(xw, h),
            DisplayAction::Unfocus(h, f) => from_unfocus(xw, h, f),
            DisplayAction::ReplayClick(h, b) => from_replay_click(xw, h, b.bits()),
            DisplayAction::SetState(h, t, s) => from_set_state(xw, h, t, s),
            DisplayAction::SetWindowOrder(ws) => from_set_window_order(xw, ws),
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
        match event {
            Ok(ev) => {
                if ev.is_some() {
                    tracing::trace!("DisplayEvent: {:?}", ev);
                }
                ev
            }
            Err(e) => {
                tracing::error!(
                    "Error when processing a display action:\n\tAction: {:?}\n\tError: {}",
                    act,
                    e
                );
                None
            }
        }
    }

    fn wait_readable(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()>>> {
        let task_notify = self.xw.task_notify.clone();
        Box::pin(async move {
            task_notify.notified().await;
        })
    }

    fn flush(&self) {
        if let Err(e) = self.xw.flush() {
            tracing::error!(error = ?e, "Error when flushing the connection.");
        }
    }

    fn generate_verify_focus_event(&self) -> Option<leftwm_core::DisplayEvent<X11rbWindowHandle>> {
        let handle = self.xw.get_cursor_window().ok()?;
        Some(DisplayEvent::VerifyFocusedAt(handle))
    }
}

impl X11rbDisplayServer {
    fn initial_events(&self, config: &impl Config) -> Vec<DisplayEvent<X11rbWindowHandle>> {
        let mut events = vec![];
        if let Some(workspaces) = config.workspaces() {
            let screens = match self.xw.get_screens() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = ?e, "An error occurred when trying to get screens.");
                    return events;
                }
            };

            for (i, wsc) in workspaces.iter().enumerate() {
                let mut screen = Screen::from(wsc);
                screen.root = WindowHandle(X11rbWindowHandle(self.root));
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

    fn find_all_windows(&self) -> Vec<DisplayEvent<X11rbWindowHandle>> {
        let mut all: Vec<DisplayEvent<X11rbWindowHandle>> = Vec::new();
        match self.xw.get_all_windows() {
            Ok(handles) => handles.into_iter().for_each(|handle| {
                let attrs = match self.xw.get_window_attrs(handle) {
                    Ok(a) => a,
                    Err(e) => {
                        return tracing::error!(window_handle = handle, error = ?e, "Error when getting window attributes.");
                    }
                };
                let state = match self.xw.get_wm_state(handle) {
                    Ok((s, _)) => s,
                    Err(e) => {
                        return tracing::error!(window_handle = handle, error = ?e, "Error when getting WM_STATE atom.");
                    }
                };
                if attrs.map_state == xproto::MapState::VIEWABLE
                    || state == xatom::WMStateWindowState::Iconic
                {
                    match self.xw.setup_window(handle) {
                        Ok(Some(event)) => {
                            all.push(event);
                        }
                        Err(e) => tracing::error!(window_handle = handle, error = ?e, "Error when setting up window."),
                        _ => (),
                    }
                }
            }),
            Err(err) => {
                tracing::error!(error = ?err, "An error occurred.");
            }
        }
        all
    }
}

// Display actions.
fn from_kill_window(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.kill_window(&handle)?;
    Ok(None)
}

fn from_move_mouse_over(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
    force: bool,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    match (handle, xw.get_cursor_window()?) {
        (WindowHandle(X11rbWindowHandle(window)), WindowHandle(X11rbWindowHandle(cursor_window)))
            if force || cursor_window != window =>
        {
            _ = xw.move_cursor_to_window(window)?;
        }
        _ => {}
    }
    Ok(None)
}

fn from_move_mouse_over_point(
    xw: &mut XWrap,
    point: (i32, i32),
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.move_cursor_to_point(point)?;
    Ok(None)
}

fn from_destroyed_window(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.teardown_managed_window(&handle, true)?;
    Ok(None)
}

fn from_unfocus(
    xw: &mut XWrap,
    handle: Option<WindowHandle<X11rbWindowHandle>>,
    floating: bool,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.unfocus(handle, floating)?;
    Ok(None)
}

fn from_replay_click(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
    button: xproto::Button,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if let WindowHandle(X11rbWindowHandle(handle)) = handle {
        xw.replay_click(handle, button)?;
    }
    Ok(None)
}

fn from_set_state(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
    toggle_to: bool,
    window_state: WindowState,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
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
        WindowState::Maximized => {
            xw.set_state(handle, toggle_to, xw.atoms.NetWMStateMaximizedVert)?;
            xw.set_state(handle, toggle_to, xw.atoms.NetWMStateMaximizedVert)?;
            return Ok(None);
        }
    };
    xw.set_state(handle, toggle_to, state)?;
    Ok(None)
}

fn from_set_window_order(
    xw: &mut XWrap,
    windows: Vec<WindowHandle<X11rbWindowHandle>>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    // Unmanaged windows.
    let unmanaged: Vec<WindowHandle<X11rbWindowHandle>> = xw
        .get_all_windows()?
        .iter()
        .filter(|&w| *w != xw.get_default_root())
        .map(|&w| WindowHandle(X11rbWindowHandle(w)))
        .filter(|h| !windows.iter().any(|&w| w == *h))
        .collect();
    let all: Vec<WindowHandle<X11rbWindowHandle>> = [unmanaged, windows].concat();
    xw.restack(all)?;
    Ok(None)
}

fn from_move_to_top(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.move_to_top(&handle)?;
    Ok(None)
}

fn from_ready_to_move_window(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.set_mode(Mode::ReadyToMove(handle))?;
    Ok(None)
}

fn from_ready_to_resize_window(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.set_mode(Mode::ReadyToResize(handle))?;
    Ok(None)
}

fn from_set_current_tags(
    xw: &mut XWrap,
    tag: Option<TagId>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.set_current_desktop(tag)?;
    Ok(None)
}

fn from_set_window_tag(
    xw: &mut XWrap,
    handle: WindowHandle<X11rbWindowHandle>,
    tag: Option<TagId>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    if let WindowHandle(X11rbWindowHandle(window)) = handle {
        match tag {
            Some(tag) => xw.set_window_desktop(window, &tag)?,
            None => (),
        }
    }
    Ok(None)
}

fn from_configure_xlib_window(
    xw: &mut XWrap,
    window: &Window<X11rbWindowHandle>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.configure_window(window)?;
    Ok(None)
}

fn from_window_take_focus(
    xw: &mut XWrap,
    window: &Window<X11rbWindowHandle>,
    previous_window: &Option<Window<X11rbWindowHandle>>,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.window_take_focus(window, previous_window.as_ref())?;
    Ok(None)
}

fn from_focus_window_under_cursor(
    xw: &mut XWrap,
) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    let mut window = xw.get_cursor_window()?;
    if window == WindowHandle(X11rbWindowHandle(0)) {
        window = xw.get_default_root_handle();
        return Ok(Some(DisplayEvent::WindowTakeFocus(window)));
    }
    let point = xw.get_cursor_point()?;
    let evt = DisplayEvent::MoveFocusTo(point.0, point.1);
    Ok(Some(evt))
}

fn from_normal_mode(xw: &mut XWrap) -> Result<Option<DisplayEvent<X11rbWindowHandle>>> {
    xw.set_mode(Mode::Normal)?;
    Ok(None)
}
