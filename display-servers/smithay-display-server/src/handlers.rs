pub mod display_action;
mod screencopy;
mod xdg;

use std::cell::RefCell;

use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::{utils::on_commit_buffer_handler, ImportDma},
    },
    delegate_compositor, delegate_data_device, delegate_dmabuf, delegate_layer_shell,
    delegate_output, delegate_seat, delegate_shm,
    desktop::{layer_map_for_output, LayerSurface, WindowSurfaceType},
    input::{SeatHandler, SeatState},
    output::Output,
    reexports::{
        calloop::Interest,
        wayland_server::{
            protocol::{wl_buffer::WlBuffer, wl_output::WlOutput, wl_surface::WlSurface},
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
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        dmabuf::{get_dmabuf, DmabufGlobal, DmabufHandler, DmabufState, ImportError},
        seat::WaylandFocus,
        shell::{
            wlr_layer::{
                Layer, LayerSurface as WlrLayerSurface, LayerSurfaceData, WlrLayerShellHandler,
                WlrLayerShellState,
            },
            xdg::XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
    },
    xwayland::xwm::ResizeEdge,
};

use crate::{
    managed_window::ManagedWindow,
    state::{ClientState, SmithayState},
    window_registry::WindowRegisty,
    SmithayWindowHandle,
};
use leftwm_core::{
    models::{WindowHandle, WindowType},
    DisplayEvent, Window as WMWindow,
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

        if let Some((output, _)) = self.outputs.iter().find(|(o, _)| {
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
            let mut map = layer_map_for_output(&output);
            map.arrange();

            if !initial_configure_sent {
                let layer = map
                    .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
                    .unwrap();

                layer.layer_surface().send_configure();
            }
        }

        // self.popups.commit(surface);

        ensure_initial_configure(
            surface,
            &self.window_registry,
            &self.outputs.iter().map(|(o, _)| o.clone()).collect(),
        )
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

impl DataDeviceHandler for SmithayState {
    type SelectionUserData = ();

    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for SmithayState {}
impl ServerDndGrabHandler for SmithayState {}

delegate_data_device!(SmithayState);

impl WlrLayerShellHandler for SmithayState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = output
            .as_ref()
            .and_then(Output::from_resource)
            .unwrap_or_else(|| self.outputs.iter().next().map(|(o, _)| o).unwrap().clone());
        let mut map = layer_map_for_output(&output);
        let layer_surface = LayerSurface::new(surface, namespace);
        map.map_layer(&layer_surface).unwrap();

        let window = ManagedWindow::from_surface(layer_surface);
        let id = self.window_registry.insert(window.clone());

        let mut wm_window = WMWindow::new(WindowHandle(SmithayWindowHandle(id)), None, None);
        wm_window.r#type = WindowType::WlrSurface;
        self.send_event(DisplayEvent::WindowCreate(
            wm_window,
            self.pointer_location.x as i32,
            self.pointer_location.y as i32,
        ))
        .unwrap();
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        if let Some((mut map, layer)) = self.outputs.iter().find_map(|(o, _)| {
            let map = layer_map_for_output(o);
            let layer = map
                .layers()
                .find(|&layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (map, layer))
        }) {
            map.unmap_layer(&layer);
        }
    }
}

delegate_layer_shell!(SmithayState);

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
        self.window_registry
            .windows()
            .find(|(_, w)| w.wl_surface().map(|s| s == *surface).unwrap_or(false))
            .map(|(_, w)| w)
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
#[allow(dead_code)]
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
    windows: &WindowRegisty,
    outputs: &Vec<Output>,
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

    if let Some(window) = windows
        .windows()
        .map(|(_, w)| w)
        .find(|window| window.wl_surface().map(|s| s == *surface).unwrap_or(false))
        .cloned()
    {
        if !window.is_wlr_surface() {
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
                //Unwrapping is safe since this window is never a wlr layer surface
                window.toplevel().unwrap().send_configure();
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

    if let Some(output) = outputs.iter().find(|o| {
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
