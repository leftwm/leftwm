use std::{process::Command, sync::atomic::Ordering, time::Duration};

use event_channel::EventChannelReceiver;
use internal_action::InternalAction;
use leftwm_config::{BorderConfig, LeftwmConfig};
use leftwm_core::{
    models::{Handle, Window, WindowHandle},
    DisplayAction, DisplayEvent, DisplayServer,
};
use serde::{Deserialize, Serialize};
use smithay::{
    backend::{
        input::{Event, InputEvent, KeyState, KeyboardKeyEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::UdevBackend,
        SwapBuffersError,
    },
    input::keyboard::{xkb, FilterResult},
    reexports::{
        calloop::{
            channel::{self, Sender as CalloopSender},
            EventLoop,
        },
        input::{Led, Libinput},
        wayland_server::Display,
    },
    utils::SERIAL_COUNTER,
};
use tokio::sync::oneshot;
use tracing::{error, info, warn};

use crate::state::{CalloopData, SmithayState};
mod drawing;
mod event_channel;
mod handlers;
mod input_handler;
mod internal_action;
mod leftwm_config;
mod managed_window;
mod pointer;
mod protocols;
mod state;
mod udev;
mod window_registry;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SmithayWindowHandle(usize);

impl Handle for SmithayWindowHandle {}

pub struct SmithayHandle {
    event_receiver: EventChannelReceiver,
    action_sender: CalloopSender<InternalAction>,
}

impl DisplayServer<SmithayWindowHandle> for SmithayHandle {
    fn new(config: &impl leftwm_core::Config) -> Self {
        let (event_sender, event_receiver) = event_channel::event_channel();
        let (init_notify_sender, init_notify_receiver) = oneshot::channel::<()>();
        let (action_sender, action_receiver) = channel::channel::<InternalAction>();

        let config = LeftwmConfig {
            focus_behavior: config.focus_behaviour(),
            sloppy_mouse_follows_focus: config.sloppy_mouse_follows_focus(),

            borders: BorderConfig {
                border_width: config.border_width(),
                default_border_color: read_color::rgb(&mut config.default_border_color().chars())
                    .map_or([0, 0, 0].into(), Into::into),
                floating_border_color: read_color::rgb(&mut config.floating_border_color().chars())
                    .map_or([0, 0, 0].into(), Into::into),
                focused_border_color: read_color::rgb(&mut config.focused_border_color().chars())
                    .map_or([255, 0, 0].into(), Into::into),
            },
        };

        std::thread::spawn(move || {
            let mut event_loop = EventLoop::<CalloopData>::try_new().unwrap();
            let mut display = Display::<SmithayState>::new().unwrap();

            let (session, notifier) = LibSeatSession::new().unwrap();

            let mut state = SmithayState::init(
                event_sender,
                &mut display,
                udev::init_udev_stage_1(session),
                config,
                event_loop.handle(),
                event_loop.get_signal(),
            );

            let udev_backend = match UdevBackend::new(&state.seat_name) {
                Ok(ret) => ret,
                Err(err) => {
                    panic!("Failed to initialize udev backend: {}", err);
                }
            };

            let mut libinput_context = Libinput::new_with_udev::<
                LibinputSessionInterface<LibSeatSession>,
            >(state.udev_data.session.clone().into());
            libinput_context
                .udev_assign_seat(&state.udev_data.session.seat())
                .unwrap();

            let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

            // TODO: Proper key handling
            event_loop
                .handle()
                .insert_source(libinput_backend, move |event, _, data| {
                    match event {
                        InputEvent::Keyboard { event, .. } => {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = Event::time_msec(&event);
                            if let Some(Some(vt)) = data.state.seat.get_keyboard().unwrap().input(
                                &mut data.state,
                                event.key_code(),
                                event.state(),
                                serial,
                                time,
                                |state, modifiers, handle| {
                                    if event.state() == KeyState::Pressed {
                                        let mut leds = Led::empty();
                                        if modifiers.caps_lock {
                                            leds.insert(Led::CAPSLOCK);
                                        }
                                        if modifiers.num_lock {
                                            leds.insert(Led::NUMLOCK);
                                        }
                                        event.device().led_update(leds);
                                        if modifiers.logo
                                            && modifiers.shift
                                            && handle.modified_sym() == xkb::KEY_Return
                                        {
                                            Command::new("kitty").spawn().unwrap();
                                            FilterResult::Intercept(None)
                                        } else if modifiers.logo
                                            && modifiers.shift
                                            && handle.modified_sym() == xkb::KEY_Q
                                        {
                                            info!("Exiting");
                                            state.running.store(false, Ordering::SeqCst);
                                            FilterResult::Intercept(None)
                                        } else if (xkb::KEY_XF86Switch_VT_1
                                            ..=xkb::KEY_XF86Switch_VT_12)
                                            .contains(&handle.modified_sym())
                                        {
                                            // VTSwitch
                                            let vt = (handle.modified_sym()
                                                - xkb::KEY_XF86Switch_VT_1
                                                + 1)
                                                as i32;
                                            FilterResult::Intercept(Some(vt))
                                        } else {
                                            FilterResult::Forward
                                        }
                                    } else if event.state() == KeyState::Released {
                                        let mut leds = Led::empty();
                                        if modifiers.caps_lock {
                                            leds.insert(Led::CAPSLOCK);
                                        }
                                        if modifiers.num_lock {
                                            leds.insert(Led::NUMLOCK);
                                        }
                                        event.device().led_update(leds);
                                        FilterResult::Forward
                                    } else {
                                        FilterResult::Forward
                                    }
                                },
                            ) {
                                data.state.udev_data.session.change_vt(vt).unwrap();
                            };
                        }
                        InputEvent::PointerMotion { event } => {
                            data.state.on_pointer_move::<LibinputInputBackend>(event);
                        }
                        InputEvent::PointerMotionAbsolute { event } => {
                            data.state
                                .on_pointer_move_absolute::<LibinputInputBackend>(event)
                        }
                        InputEvent::DeviceAdded { mut device } => {
                            device.config_tap_set_enabled(true).ok();
                            device.config_tap_set_drag_enabled(true).ok();
                        }
                        _ => {}
                    };
                })
                .unwrap();

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
                            for (crtc, surface) in backend
                                .surfaces
                                .iter_mut()
                                .map(|(handle, backend)| (*handle, backend))
                            {
                                if let Err(err) = surface.compositor.surface().reset_state() {
                                    warn!("Failed to reset crtc state: {:?}", err);
                                }
                                surface.compositor.reset_buffers();
                                data.state.loop_handle.insert_idle(move |data| {
                                    if let Some(SwapBuffersError::ContextLost(_)) =
                                        data.state.render(node, crtc, None).err()
                                    {
                                        panic!("Device context lost ({})", node);
                                    }
                                });
                            }
                        }
                    }
                })
                .unwrap();

            state.init_udev_stage_2(udev_backend, &display);

            event_loop
                .handle()
                .insert_source(action_receiver, |event, _, data| match event {
                    channel::Event::Msg(act) => {
                        data.state.handle_action(act, &mut data.display);
                    }
                    channel::Event::Closed => {
                        info!("LeftWM closed the channel, assuming we're exiting.");
                        data.state.running.store(false, Ordering::SeqCst);
                    }
                })
                .unwrap();

            init_notify_sender.send(()).unwrap();

            while state.running.load(Ordering::SeqCst) {
                let mut calloop_data = CalloopData { state, display };
                let result =
                    event_loop.dispatch(Some(Duration::from_millis(16)), &mut calloop_data);
                CalloopData { state, display } = calloop_data;

                if result.is_err() {
                    state.running.store(false, Ordering::SeqCst);
                } else {
                    state.window_registry.clean();
                    // state.popups.cleanup();
                    display.flush_clients().unwrap();
                }
            }
        });

        std::env::set_var("XDG_SESSION_TYPE", "wayland");

        init_notify_receiver.blocking_recv().unwrap();

        Self {
            event_receiver,
            action_sender,
        }
    }

    fn get_next_events(&mut self) -> Vec<DisplayEvent<SmithayWindowHandle>> {
        // info!("LeftWM is collecting events");
        self.event_receiver.collect_events()
    }

    //NOTE: Adding the `'_` lifetime here requires the `DisplayServer` trait to be modified to add
    //the lifetime there too.
    fn wait_readable(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + '_>> {
        Box::pin(self.event_receiver.wait_readable())
    }

    fn flush(&self) {
        self.action_sender.send(InternalAction::Flush).unwrap();
    }

    fn generate_verify_focus_event(&self) -> Option<DisplayEvent<SmithayWindowHandle>> {
        self.action_sender
            .send(InternalAction::GenerateVerifyFocusEvent)
            .unwrap();
        None
    }

    fn reload_config(
        &mut self,
        config: &impl leftwm_core::Config,
        _focused: Option<WindowHandle<SmithayWindowHandle>>,
        _windows: &[leftwm_core::Window<SmithayWindowHandle>],
    ) {
        let config = LeftwmConfig {
            focus_behavior: config.focus_behaviour(),
            sloppy_mouse_follows_focus: config.sloppy_mouse_follows_focus(),

            borders: BorderConfig {
                border_width: config.border_width(),
                default_border_color: read_color::rgb(&mut config.default_border_color().chars())
                    .map_or([0, 0, 0].into(), Into::into),
                floating_border_color: read_color::rgb(&mut config.floating_border_color().chars())
                    .map_or([0, 0, 0].into(), Into::into),
                focused_border_color: read_color::rgb(&mut config.focused_border_color().chars())
                    .map_or([255, 0, 0].into(), Into::into),
            },
        };
        self.action_sender
            .send(InternalAction::UpdateConfig(config))
            .unwrap();
    }

    fn update_windows(&self, windows: Vec<&Window<SmithayWindowHandle>>) {
        let windows = windows.into_iter().map(|w| w.clone()).collect();
        self.action_sender
            .send(InternalAction::UpdateWindows(windows))
            .unwrap()
    }

    fn update_workspaces(&self, _focused: Option<&leftwm_core::Workspace>) {}

    fn execute_action(
        &mut self,
        act: DisplayAction<SmithayWindowHandle>,
    ) -> Option<DisplayEvent<SmithayWindowHandle>> {
        self.action_sender
            .send(InternalAction::DisplayAction(act))
            .unwrap();
        None
    }
}
