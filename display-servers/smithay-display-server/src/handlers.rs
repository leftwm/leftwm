use smithay::{
    delegate_compositor, delegate_seat,
    input::{SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Client},
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
};

use crate::state::SmithayState;

impl CompositorHandler for SmithayState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        todo!()
    }

    fn commit(&mut self, surface: &WlSurface) {
        todo!()
    }

    fn new_surface(&mut self, surface: &WlSurface) {}

    fn destroyed(&mut self, _surface: &WlSurface) {}
}

delegate_compositor!(SmithayState);

impl SeatHandler for SmithayState {
    type KeyboardFocus = todo!();

    type PointerFocus = todo!();

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

delegate_seat!(SmithayState);
