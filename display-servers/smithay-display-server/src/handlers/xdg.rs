use smithay::{
    delegate_xdg_shell,
    desktop::Window,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::State,
        wayland_server::protocol::wl_seat::WlSeat,
    },
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

use crate::{managed_window::ManagedWindow, state::SmithayState};
use leftwm_core::{models::WindowHandle, DisplayEvent, Window as WMWindow};

impl XdgShellHandler for SmithayState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = ManagedWindow::new(Window::new(surface));
        let id = self.window_registry.insert(window.clone());

        self.send_event(DisplayEvent::WindowCreate(
            WMWindow::new(WindowHandle::SmithayHandle(id), None, None),
            self.pointer_location.x as i32,
            self.pointer_location.y as i32,
        ))
        .unwrap();

        window.window.toplevel().with_pending_state(|state| {
            state.states.set(State::TiledTop);
            state.states.set(State::TiledBottom);
            state.states.set(State::TiledRight);
            state.states.set(State::TiledLeft);
        });
        window.window.toplevel().send_configure();

        // self.space.map_element(window, (0, 0), true);
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        todo!()
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        todo!()
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        for (h, w) in self.window_registry.windows() {
            if *w.toplevel() == surface {
                self.send_event(DisplayEvent::WindowDestroy(WindowHandle::SmithayHandle(*h)))
                    .unwrap();
                return;
            }
        }
    }

    fn popup_destroyed(&mut self, _surface: PopupSurface) {}
}

delegate_xdg_shell!(SmithayState);
