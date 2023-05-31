use std::{
    ffi::OsString,
    os::fd::AsRawFd,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
    time::Instant,
};

use leftwm_core::DisplayEvent;
use smithay::{
    desktop::{Space, Window},
    input::{pointer::CursorImageStatus, Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{backend::ClientData, Display, DisplayHandle},
    },
    utils::{Clock, Logical, Monotonic, Point},
    wayland::{compositor::CompositorState, shm::ShmState, socket::ListeningSocketSource},
};

use crate::udev::UdevData;

pub struct SmithayState {
    pub display_handle: DisplayHandle,
    pub udev_data: UdevData,
    pub start_time: Instant,
    pub loop_handle: LoopHandle<'static, CalloopData>,
    pub loop_signal: LoopSignal,
    pub space: Space<Window>,
    pub clock: Clock<Monotonic>,
    pub running: Arc<AtomicBool>,

    pub pointer_location: Point<f64, Logical>,
    pub cursor_status: Arc<Mutex<CursorImageStatus>>,

    // Protocols
    pub compositor_state: CompositorState,
    // xdg_shell_state
    // xdg_decoration_state
    pub shm_state: ShmState,
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
        let space = Space::default();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let seat_name = udev_data.seat_name();
        let seat = seat_state.new_wl_seat(&dh, seat_name.clone());
        let cursor_status = Arc::new(Mutex::new(CursorImageStatus::Default));

        let clock = Clock::new().unwrap();

        let socket_name = Self::init_wayland_listener(&mut loop_handle, display);

        Self {
            display_handle: dh,
            udev_data,
            start_time: Instant::now(),
            loop_handle,
            loop_signal,
            space,
            clock,
            running: Arc::new(AtomicBool::new(true)),

            pointer_location: (0f64, 0f64).into(),
            cursor_status,

            compositor_state,
            shm_state,
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
