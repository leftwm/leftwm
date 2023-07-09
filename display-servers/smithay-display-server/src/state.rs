use std::{
    ffi::OsString,
    os::fd::AsRawFd,
    sync::{atomic::AtomicBool, mpsc::SendError, Arc, Mutex},
    time::Instant,
};

use leftwm_core::{models::FocusBehaviour, DisplayEvent};
use smithay::{
    desktop::space::SpaceElement,
    input::{keyboard::XkbConfig, pointer::CursorImageStatus, Seat, SeatState},
    output::Output,
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{backend::ClientData, Display, DisplayHandle},
    },
    utils::{Clock, Logical, Monotonic, Point, Rectangle, SERIAL_COUNTER},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};
use tracing::{debug, warn};

use crate::{
    event_channel::EventChannelSender,
    leftwm_config::LeftwmConfig,
    udev::UdevData,
    window_registry::{WindowHandle, WindowRegisty},
};

pub struct SmithayState {
    pub display_handle: DisplayHandle,
    pub udev_data: UdevData,
    pub start_time: Instant,
    pub loop_handle: LoopHandle<'static, CalloopData>,
    pub loop_signal: LoopSignal,
    // pub space: Space<ManagedWindow>,
    pub outputs: Vec<(Output, Rectangle<i32, Logical>)>,
    pub clock: Clock<Monotonic>,
    pub running: Arc<AtomicBool>,

    pub pointer_location: Point<f64, Logical>,
    pub cursor_status: Arc<Mutex<CursorImageStatus>>,

    // Protocols
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
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

    pub window_registry: WindowRegisty,
    pub config: LeftwmConfig,
    pub focused_window: Option<WindowHandle>,

    event_sender: EventChannelSender,
}

pub struct CalloopData {
    pub state: SmithayState,
    pub display: Display<SmithayState>,
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}
impl ClientData for ClientState {}

impl SmithayState {
    pub fn init(
        event_sender: EventChannelSender,
        display: &mut Display<SmithayState>,
        udev_data: UdevData,
        config: LeftwmConfig,
        mut loop_handle: LoopHandle<'static, CalloopData>,
        loop_signal: LoopSignal,
    ) -> Self {
        let dh = display.handle();
        let outputs = Vec::new();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let shm_state = ShmState::new::<Self>(&dh, vec![]);

        let seat_name = udev_data.seat_name();
        let mut seat = seat_state.new_wl_seat(&dh, seat_name.clone());
        seat.add_keyboard(XkbConfig::default(), 0, 0).unwrap();
        seat.add_pointer();

        let window_registry = WindowRegisty::new();

        let cursor_status = Arc::new(Mutex::new(CursorImageStatus::Default));

        let clock = Clock::new().unwrap();

        let socket_name = Self::init_wayland_listener(&mut loop_handle, display);

        Self {
            display_handle: dh,
            udev_data,
            start_time: Instant::now(),
            loop_handle,
            loop_signal,
            // space,
            outputs,
            clock,
            running: Arc::new(AtomicBool::new(true)),

            pointer_location: (0f64, 0f64).into(),
            cursor_status,

            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,

            seat,
            seat_name,
            socket_name,

            window_registry,
            config,
            focused_window: None,

            event_sender,
        }
    }

    fn init_wayland_listener(
        handle: &mut LoopHandle<'static, CalloopData>,
        display: &mut Display<SmithayState>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::with_name("wayland-0").unwrap();

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
                    .insert_client(client_stream, Arc::new(ClientState::default()))
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

        println!("{:?}", socket_name);

        std::env::set_var("WAYLAND_DISPLAY", socket_name.clone());
        socket_name
    }

    pub fn send_event(&self, event: DisplayEvent) -> Result<(), SendError<()>> {
        debug!("Sending event: {:#?}", event);
        self.event_sender.send_event(event)
    }

    pub fn focus_window(&mut self, handle: WindowHandle, move_cursor: bool) {
        let serial = SERIAL_COUNTER.next_serial();
        let Some(window) = self.window_registry.get(handle).cloned() else {
            warn!("Trying to focus invalid window");
            return;
        };
        let bbox = window.bbox();
        self.seat
            .get_keyboard()
            .unwrap()
            .set_focus(self, Some(window), serial);
        if move_cursor {
            let x = bbox.loc.x as f64 + bbox.size.w as f64 / 2f64;
            let y = bbox.loc.y as f64 + bbox.size.h as f64 / 2f64;
            self.pointer_location = (x, y).into();
        }
        self.focused_window = Some(handle);
    }

    pub fn focus_window_under(&mut self) {
        let under = self.surface_under();
        if self.config.focus_behavior == FocusBehaviour::Sloppy {
            if let Some((window, _)) = under.clone() {
                if window.get_handle() != self.focused_window {
                    if let Some(h) = window.get_handle() {
                        self.focus_window(h, false);
                        // self.send_event(DisplayEvent::WindowTakeFocus(
                        //     WindowHandle::SmithayHandle(h),
                        // ))
                        // .unwrap();
                    }
                }
            }
        }
    }
}
