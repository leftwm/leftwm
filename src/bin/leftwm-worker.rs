use leftwm::child_process::Nanny;

use leftwm::*;
use std::panic;
use std::sync::Once;

fn get_events<T: DisplayServer>(ds: &mut T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

use slog::{o, Drain};

fn main() {
    let _log_guard = setup_logging();
    log::info!("leftwm-worker booted!");

    let completed = panic::catch_unwind(|| {
        let config = config::load();

        let mut manager = Manager::default();
        manager.tags = config.get_list_of_tags();

        let mut display_server = XlibDisplayServer::new(&config);
        let handler = DisplayEventHandler { config };

        event_loop(&mut manager, &mut display_server, &handler);
    });

    match completed {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}

fn event_loop(
    manager: &mut Manager,
    display_server: &mut XlibDisplayServer,
    handler: &DisplayEventHandler,
) {
    let mut state_socket = StateSocket::new();
    let mut command_pipe = CommandPipe::new();

    //start the current theme
    let after_first_loop: Once = Once::new();

    //main event loop
    let mut events_remainder = vec![];
    loop {
        if manager.mode == Mode::NormalMode {
            let _ = state_socket.write_manager_state(manager);
        }
        let mut events = get_events(display_server);
        events.append(&mut events_remainder);

        let mut needs_update = false;
        for event in events {
            needs_update = handler.process(manager, event) || needs_update;
        }

        if let Some(cmd) = command_pipe.read_command() {
            needs_update = external_command_handler::process(manager, cmd) || needs_update;
            display_server.update_theme_settings(manager.theme_setting.clone());
        }

        //if we need to update the displayed state
        if needs_update {
            match &manager.mode {
                Mode::NormalMode => {
                    let windows: Vec<&Window> = (&manager.windows).iter().collect();
                    let focused = manager.focused_window();
                    display_server.update_windows(windows, focused);
                    let workspaces: Vec<&Workspace> =
                        (&manager.workspaces).iter().map(|w| w).collect();
                    let focused = manager.focused_workspace();
                    display_server.update_workspaces(workspaces, focused);
                }
                //when (resizing / moving) only deal with the single window
                Mode::ResizingWindow(h) | Mode::MovingWindow(h) => {
                    let focused = manager.focused_window();
                    let windows: Vec<&Window> = (&manager.windows)
                        .iter()
                        .filter(|w| &w.handle == h)
                        .collect();
                    display_server.update_windows(windows, focused);
                }
            }
        }

        //preform any actions requested by the handler
        while !manager.actions.is_empty() {
            if let Some(act) = manager.actions.pop_front() {
                if let Some(event) = display_server.execute_action(act) {
                    events_remainder.push(event);
                }
            }
        }

        //after the very first loop boot the theme. we need the unix socket to already exist
        after_first_loop.call_once(|| {
            if let Err(err) = Nanny::new().boot_current_theme() {
                log::error!("Theme loading failed: {}", err);
            }

            leftwm::state::load(manager);
        });

        if manager.reload_requested {
            break;
        }
    }
}

/// Log to both stdout and journald.
fn setup_logging() -> slog_scope::GlobalLoggerGuard {
    #[cfg(feature = "slog-journald")]
    let journald = slog_journald::JournaldDrain.ignore_res();

    #[cfg(feature = "slog-term")]
    let stdout = slog_term::CompactFormat::new(slog_term::TermDecorator::new().stdout().build())
        .build()
        .ignore_res();

    #[cfg(all(feature = "slog-journald", feature = "slog-term"))]
    let drain = slog::Duplicate(journald, stdout).ignore_res();
    #[cfg(all(feature = "slog-journald", not(feature = "slog-term")))]
    let drain = journald;
    #[cfg(all(not(feature = "slog-journald"), feature = "slog-term"))]
    let drain = stdout;

    // Set level filters from RUST_LOG. Defaults to `info`.
    let envlogger = slog_envlogger::LogBuilder::new(drain)
        .parse(&std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .build()
        .ignore_res();

    let logger = slog::Logger::root(slog_async::Async::default(envlogger).ignore_res(), o!());

    slog_stdlog::init().unwrap_or_else(|err| {
        eprintln!("failed to setup logging: {}", err);
    });

    slog_scope::set_global_logger(logger)
}
