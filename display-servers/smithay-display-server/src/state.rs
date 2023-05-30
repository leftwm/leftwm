use std::{
    ffi::OsString,
    os::fd::AsRawFd,
    sync::{mpsc::Sender, Arc},
    time::Instant,
};

use leftwm_core::DisplayEvent;
use smithay::{
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{backend::ClientData, Display, DisplayHandle},
    },
    wayland::{compositor::CompositorState, socket::ListeningSocketSource},
};

use crate::udev::UdevData;

pub struct SmithayState {
    pub display_handle: DisplayHandle,
    pub udev_data: UdevData,
    pub start_time: Instant,
    pub loop_handle: LoopHandle<'static, CalloopData>,
    pub loop_signal: LoopSignal,

    // Protocols
    pub compositor_state: CompositorState,
    // xdg_shell_state
    // xdg_decoration_state
    // shm_state
    // output_manager_State
    // data_device_state
    // primary_selection_state
    pub seat_state: SeatState<Self>,
    // layer_shell_state
    // popup_manager
    //
    pub seat: Seat<Self>,
    pub seat_name: String,
    pub socket_name: OsString,

    event_sender: Sender<DisplayEvent>,
}

pub struct CalloopData {
    pub state: SmithayState,
    pub display: Display<SmithayState>,
}

pub struct ClientState;

impl ClientData for ClientState {}

impl SmithayState {
    pub fn init(
        event_sender: Sender<DisplayEvent>,
        display: &mut Display<SmithayState>,
        udev_data: UdevData,
        mut loop_handle: LoopHandle<'static, CalloopData>,
        loop_signal: LoopSignal,
    ) -> Self {
        let dh = display.handle();

        let compositor_state = CompositorState::new(&dh);
        let mut seat_state = SeatState::new();
        let seat_name = udev_data.seat_name();
        let mut seat = seat_state.new_wl_seat(&dh, seat_name.clone());

        let socket_name = Self::init_wayland_listener(&mut loop_handle, display);

        Self {
            display_handle: dh,
            udev_data,
            start_time: Instant::now(),
            loop_handle,
            loop_signal,

            compositor_state,
            seat_state,

            seat,
            seat_name,
            socket_name,

            event_sender,
        }
    }

    fn init_wayland_listener(
        handle: &mut LoopHandle<'static, CalloopData>,
        display: &mut Display<SmithayState>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket.socket_name().to_os_string();

        handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                // Inside the callback, you should insert the client into the display.
                //
                // You may also associate some data with the client when inserting the client.
                state
                    .display
                    .handle()
                    .insert_client(client_stream, Arc::new(ClientState))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");

        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.
        handle
            .insert_source(
                Generic::new(
                    display.backend().poll_fd().as_raw_fd(),
                    Interest::READ,
                    Mode::Level,
                ),
                |_, _, state| {
                    state.display.dispatch_clients(&mut state.state).unwrap();
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }
}
