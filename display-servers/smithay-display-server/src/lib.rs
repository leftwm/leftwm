use std::sync::mpsc::{self, Receiver};

use leftwm_core::{DisplayAction, DisplayEvent, DisplayServer};
use smithay::{
    backend::{
        input::{Event, InputEvent, KeyboardKeyEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        SwapBuffersError,
    },
    input::keyboard::{xkb, FilterResult},
    reexports::{
        calloop::{
            channel::{self, Channel as CalloopReciever, Sender as CalloopSender},
            EventLoop,
        },
        input::{Led, Libinput},
        wayland_server::Display,
    },
    utils::SERIAL_COUNTER,
};
use tracing::{error, info, warn};

use crate::state::{CalloopData, SmithayState};
mod handlers;
mod rendering;
mod state;
mod udev;

struct SmithayHandle {
    event_receiver: Receiver<DisplayEvent>,
    action_sender: CalloopSender<DisplayAction>,
}

type WMHandle = CalloopReciever<DisplayAction>;

impl DisplayServer for SmithayHandle {
    fn new(config: &impl leftwm_core::Config) -> Self {
        let mut event_loop = EventLoop::<CalloopData>::try_new().unwrap();
        let mut display = Display::<SmithayState>::new().unwrap();

        let (event_sender, event_receiver) = mpsc::channel();
        let (action_sender, action_reciever) = channel::channel::<DisplayAction>();

        let (session, notifier) = LibSeatSession::new().unwrap();

        let mut state = SmithayState::init(
            event_sender,
            &mut display,
            udev::init_udev(session),
            event_loop.handle(),
            event_loop.get_signal(),
        );

        let mut libinput_context = Libinput::new_with_udev::<
            LibinputSessionInterface<LibSeatSession>,
        >(state.udev_data.session.clone().into());
        libinput_context
            .udev_assign_seat(&state.udev_data.session.seat())
            .unwrap();

        let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

        event_loop
            .handle()
            .insert_source(libinput_backend, move |event, _, calloopdata| {
                match event {
                    InputEvent::Keyboard { event, .. } => {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Event::time_msec(&event);
                        if let Some(vt) = calloopdata.state.seat.get_keyboard().unwrap().input(
                            &mut calloopdata.state,
                            event.key_code(),
                            event.state(),
                            serial,
                            time,
                            |_, modifiers, handle| {
                                let mut leds = Led::empty();
                                if modifiers.caps_lock {
                                    leds.insert(Led::CAPSLOCK);
                                }
                                if modifiers.num_lock {
                                    leds.insert(Led::NUMLOCK);
                                }
                                event.device().led_update(leds);
                                if (xkb::KEY_XF86Switch_VT_1..=xkb::KEY_XF86Switch_VT_12)
                                    .contains(&handle.modified_sym())
                                {
                                    // VTSwitch
                                    let vt = (handle.modified_sym() - xkb::KEY_XF86Switch_VT_1 + 1)
                                        as i32;
                                    return FilterResult::Intercept(vt);
                                }
                                FilterResult::Forward
                            },
                        ) {
                            calloopdata.state.udev_data.session.change_vt(vt);
                        };
                    }
                    InputEvent::DeviceAdded { mut device } => {
                        device.config_tap_set_enabled(true).ok();
                        device.config_tap_set_drag_enabled(true).ok();
                    }
                    _ => {}
                };
                ()
            })
            .unwrap();

        let handle = event_loop.handle();
        event_loop
            .handle()
            .insert_source(notifier, move |event, &mut (), data| match event {
                SessionEvent::PauseSession => {
                    libinput_context.suspend();
                    info!("pausing session");

                    for backend in data.state.udev_data.devices.values() {
                        backend.drm.pause();
                    }
                }
                SessionEvent::ActivateSession => {
                    info!("resuming session");

                    if let Err(err) = libinput_context.resume() {
                        error!("Failed to resume libinput context: {:?}", err);
                    }
                    for (node, backend) in data
                        .state
                        .udev_data
                        .devices
                        .iter_mut()
                        .map(|(handle, backend)| (*handle, backend))
                    {
                        backend.drm.activate();
                        for surface in backend.surfaces.values_mut() {
                            if let Err(err) = surface.compositor.surface().reset_state() {
                                warn!("Failed to reset drm surface state: {}", err);
                            }
                            // reset the buffers after resume to trigger a full redraw
                            // this is important after a vt switch as the primary plane
                            // has no content and damage tracking may prevent a redraw
                            // otherwise
                            surface.compositor.reset_buffers();
                        }
                        handle.insert_idle(move |data| data.state.render(node, None));
                    }
                }
            })
            .unwrap();

        todo!()
    }

    fn get_next_events(&mut self) -> Vec<DisplayEvent> {
        self.event_receiver.try_iter().collect()
    }

    fn wait_readable(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn generate_verify_focus_event(&self) -> Option<DisplayEvent> {
        todo!()
    }

    fn load_config(
        &mut self,
        _config: &impl leftwm_core::Config,
        _focused: Option<&Option<leftwm_core::models::WindowHandle>>,
        _windows: &[leftwm_core::Window],
    ) {
    }

    fn update_windows(&self, _windows: Vec<&leftwm_core::Window>) {}

    fn update_workspaces(&self, _focused: Option<&leftwm_core::Workspace>) {}

    fn execute_action(&mut self, _act: DisplayAction) -> Option<DisplayEvent> {
        None
    }
}
