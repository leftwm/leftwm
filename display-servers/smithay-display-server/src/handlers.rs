use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    delegate_compositor, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm,
    desktop::Window,
    input::{SeatHandler, SeatState},
    reexports::wayland_server::{
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
        Client,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportError},
        shm::{ShmHandler, ShmState},
    },
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
    type KeyboardFocus = Window;

    type PointerFocus = Window;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

delegate_seat!(SmithayState);

impl BufferHandler for SmithayState {
    fn buffer_destroyed(&mut self, buffer: &WlBuffer) {
        todo!()
    }
}

impl DmabufHandler for SmithayState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.udev_data.dmabuf_state.as_mut().unwrap().0
    }

    fn dmabuf_imported(
        &mut self,
        global: &DmabufGlobal,
        dmabuf: Dmabuf,
    ) -> Result<(), ImportError> {
        todo!()
    }
}

delegate_dmabuf!(SmithayState);

impl ShmHandler for SmithayState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_shm!(SmithayState);

delegate_output!(SmithayState);
