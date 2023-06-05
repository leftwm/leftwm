mod xdg;

use std::cell::RefCell;

use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::{utils::on_commit_buffer_handler, ImportDma},
    },
    delegate_compositor, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm,
    desktop::{layer_map_for_output, Space, Window, WindowSurfaceType},
    input::{Seat, SeatHandler, SeatState},
    reexports::{
        calloop::Interest,
        wayland_server::{
            protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
            Client, Resource,
        },
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            add_blocker, add_pre_commit_hook, get_parent, is_sync_subsurface, with_states,
            with_surface_tree_upward, BufferAssignment, CompositorClientState, CompositorHandler,
            CompositorState, SurfaceAttributes, TraversalAction,
        },
        dmabuf::{get_dmabuf, DmabufGlobal, DmabufHandler, DmabufState, ImportError},
        seat::WaylandFocus,
        shell::{wlr_layer::LayerSurfaceData, xdg::XdgToplevelSurfaceData},
        shm::{ShmHandler, ShmState},
    },
    xwayland::xwm::ResizeEdge,
};

use crate::{
    managed_window::ManagedWindow,
    state::{ClientState, SmithayState},
};

impl CompositorHandler for SmithayState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        if let Some(state) = client.get_data::<ClientState>() {
            return &state.compositor_state;
        }
        panic!("No valid data found on client");
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        self.udev_data.early_import(surface);

        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self.window_for_surface(&root) {
                window.on_commit();
            }
        }

        // self.popups.commit(surface);

        ensure_initial_configure(surface, &self.space)
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        add_pre_commit_hook::<Self, _>(surface, move |state, _dh, surface| {
            let maybe_dmabuf = with_states(surface, |surface_data| {
                surface_data
                    .cached_state
                    .pending::<SurfaceAttributes>()
                    .buffer
                    .as_ref()
                    .and_then(|assignment| match assignment {
                        BufferAssignment::NewBuffer(buffer) => get_dmabuf(buffer).ok(),
                        _ => None,
                    })
            });
            if let Some(dmabuf) = maybe_dmabuf {
                if let Ok((blocker, source)) = dmabuf.generate_blocker(Interest::READ) {
                    let client = surface.client().unwrap();
                    let res = state.loop_handle.insert_source(source, move |_, _, data| {
                        data.state
                            .client_compositor_state(&client)
                            .blocker_cleared(&mut data.state, &data.display.handle());
                        Ok(())
                    });
                    if res.is_ok() {
                        add_blocker(surface, blocker);
                    }
                }
            }
        })
    }

    fn destroyed(&mut self, _surface: &WlSurface) {}
}

delegate_compositor!(SmithayState);

impl SeatHandler for SmithayState {
    type KeyboardFocus = ManagedWindow;

    type PointerFocus = ManagedWindow;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

delegate_seat!(SmithayState);

impl BufferHandler for SmithayState {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl DmabufHandler for SmithayState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.udev_data.dmabuf_state.as_mut().unwrap().0
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
    ) -> Result<(), ImportError> {
        self.udev_data
            .gpu_manager
            .single_renderer(&self.udev_data.primary_gpu)
            .and_then(|mut renderer| renderer.import_dmabuf(&dmabuf, None))
            .map(|_| ())
            .map_err(|_| ImportError::Failed)
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

impl SmithayState {
    pub fn window_for_surface(&self, surface: &WlSurface) -> Option<ManagedWindow> {
        self.space
            .elements()
            .find(|window| window.wl_surface().map(|s| s == *surface).unwrap_or(false))
            .cloned()
    }
}

/// Information about the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ResizeData {
    /// The edges the surface is being resized with.
    pub edges: ResizeEdge,
    /// The initial window location.
    pub initial_window_location: Point<i32, Logical>,
    /// The initial window size (geometry width and height).
    pub initial_window_size: Size<i32, Logical>,
}

//TODO: Move Out
// State of the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResizeState {
    /// The surface is not being resized.
    NotResizing,
    /// The surface is currently being resized.
    Resizing(ResizeData),
    /// The resize has finished, and the surface needs to ack the final configure.
    WaitingForFinalAck(ResizeData, Serial),
    /// The resize has finished, and the surface needs to commit its final state.
    WaitingForCommit(ResizeData),
}

impl Default for ResizeState {
    fn default() -> Self {
        ResizeState::NotResizing
    }
}

#[derive(Default)]
pub struct SurfaceData {
    pub geometry: Option<Rectangle<i32, Logical>>,
    pub resize_state: ResizeState,
}

fn ensure_initial_configure(
    surface: &WlSurface,
    space: &Space<ManagedWindow>,
    // popups: &mut PopupManager,
) {
    with_surface_tree_upward(
        surface,
        (),
        |_, _, _| TraversalAction::DoChildren(()),
        |_, states, _| {
            states
                .data_map
                .insert_if_missing(|| RefCell::new(SurfaceData::default()));
        },
        |_, _, _| true,
    );

    if let Some(window) = space
        .elements()
        .find(|window| window.wl_surface().map(|s| s == *surface).unwrap_or(false))
        .cloned()
    {
        // send the initial configure if relevant

        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if !initial_configure_sent {
            window.toplevel().send_configure();
        }

        with_states(surface, |states| {
            let mut data = states
                .data_map
                .get::<RefCell<SurfaceData>>()
                .unwrap()
                .borrow_mut();

            // Finish resizing.
            if let ResizeState::WaitingForCommit(_) = data.resize_state {
                data.resize_state = ResizeState::NotResizing;
            }
        });

        return;
    }

    // if let Some(popup) = popups.find_popup(surface) {
    //     let PopupKind::Xdg(ref popup) = popup;
    //     let initial_configure_sent = with_states(surface, |states| {
    //         states
    //             .data_map
    //             .get::<XdgPopupSurfaceData>()
    //             .unwrap()
    //             .lock()
    //             .unwrap()
    //             .initial_configure_sent
    //     });
    //     if !initial_configure_sent {
    //         // NOTE: This should never fail as the initial configure is always
    //         // allowed.
    //         popup.send_configure().expect("initial configure failed");
    //     }
    //
    //     return;
    // };

    if let Some(output) = space.outputs().find(|o| {
        let map = layer_map_for_output(o);
        map.layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
            .is_some()
    }) {
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });

        let mut map = layer_map_for_output(output);

        // arrange the layers before sending the initial configure
        // to respect any size the client may have sent
        map.arrange();
        // send the initial configure if relevant
        if !initial_configure_sent {
            let layer = map
                .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
                .unwrap();

            layer.layer_surface().send_configure();
        }
    };
}
