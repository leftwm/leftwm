use smithay::{
    delegate_xdg_decoration, delegate_xdg_shell,
    desktop::Window,
    output::Output,
    reexports::{
        wayland_protocols::xdg::{
            decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
            shell::server::xdg_toplevel::State,
        },
        wayland_server::protocol::{wl_output::WlOutput, wl_seat::WlSeat},
    },
    utils::{Logical, Rectangle, Serial},
    wayland::{
        compositor::with_states,
        shell::xdg::{
            decoration::XdgDecorationHandler, PopupSurface, PositionerState, ToplevelSurface,
            XdgShellHandler, XdgShellState, XdgToplevelSurfaceData,
        },
    },
};

use crate::{
    delegate_xdg_output_handler, managed_window::ManagedWindow,
    protocols::xdg_output_manager::XdgOutputHandler, state::SmithayState, SmithayWindowHandle,
};
use leftwm_core::{models::WindowHandle, DisplayEvent, Window as WMWindow};

impl XdgShellHandler for SmithayState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = ManagedWindow::from_window(Window::new(surface));
        let id = self.window_registry.insert(window.clone());

        let (name, class) = with_states(window.toplevel().unwrap().wl_surface(), |states| {
            let data = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap();

            (data.title.clone(), data.app_id.clone())
        });

        let mut wm_window = WMWindow::new(WindowHandle(SmithayWindowHandle(id)), name, None);
        wm_window.res_class = class;
        self.send_event(DisplayEvent::WindowCreate(
            wm_window,
            self.pointer_location.x as i32,
            self.pointer_location.y as i32,
        ))
        .unwrap();

        window.toplevel().unwrap().with_pending_state(|state| {
            state.states.set(State::TiledTop);
            state.states.set(State::TiledBottom);
            state.states.set(State::TiledRight);
            state.states.set(State::TiledLeft);
        });
        window.toplevel().unwrap().send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        todo!()
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        todo!()
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let mut handle = None;
        for (h, w) in self.window_registry.windows() {
            if *w.toplevel().unwrap() == surface {
                handle = Some(*h);
                self.send_event(DisplayEvent::WindowDestroy(WindowHandle(
                    SmithayWindowHandle(*h),
                )))
                .unwrap();
                break;
            }
        }
        if let Some(h) = handle {
            self.window_registry.remove(h);
        }
    }

    fn popup_destroyed(&mut self, _surface: PopupSurface) {}
}

delegate_xdg_shell!(SmithayState);

impl XdgDecorationHandler for SmithayState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(Mode::ServerSide));
        toplevel.send_configure();
    }

    fn request_mode(&mut self, _toplevel: ToplevelSurface, _mode: Mode) {}

    fn unset_mode(&mut self, _toplevel: ToplevelSurface) {}
}

delegate_xdg_decoration!(SmithayState);

impl XdgOutputHandler for SmithayState {
    fn output(&mut self, output: &WlOutput) -> &(Output, Rectangle<i32, Logical>) {
        self.outputs.iter().find(|(o, _)| o.owns(output)).unwrap()
    }
}

delegate_xdg_output_handler!(SmithayState);
