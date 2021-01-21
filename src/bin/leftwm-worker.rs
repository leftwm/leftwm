use leftwm::child_process::{self, Nanny};

use leftwm::*;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Once};

fn get_events<T: DisplayServer>(ds: &mut T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

use slog::{o, Drain};

fn main() {
    let _log_guard = setup_logging();
    log::info!("leftwm-worker booted!");

    let completed = panic::catch_unwind(|| {
        let config = config::load();

        let mut manager = Manager {
            tags: config.get_list_of_tags(),
            ..Default::default()
        };

        child_process::register_child_hook(manager.reap_requested.clone());

        let mut display_server = XlibDisplayServer::new(&config);
        let handler = DisplayEventHandler { config };

        tokio::runtime::Runtime::new().unwrap().block_on(event_loop(
            &mut manager,
            &mut display_server,
            &handler,
        ));
    });

    match completed {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}

fn place_runtime_file<P>(path: P) -> std::io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    xdg::BaseDirectories::with_prefix("leftwm")?.place_runtime_file(path)
}

async fn event_loop(
    manager: &mut Manager,
    display_server: &mut XlibDisplayServer,
    handler: &DisplayEventHandler,
) {
    let socket_file = place_runtime_file("current_state.sock").unwrap();
    let mut state_socket = StateSocket::default();
    state_socket.listen(socket_file).await.unwrap();

    let pipe_file = place_runtime_file("commands.pipe").unwrap();
    let mut command_pipe = CommandPipe::default();
    command_pipe.listen(pipe_file).await.unwrap();

    //start the current theme
    let after_first_loop: Once = Once::new();

    //main event loop
    let mut events_remainder = vec![];
    loop {
        if manager.mode == Mode::NormalMode {
            state_socket.write_manager_state(manager).await.ok();
        }
        let mut events = get_events(display_server);
        events.append(&mut events_remainder);

        if events.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }

        let mut needs_update = false;
        for event in events {
            needs_update = handler.process(manager, event) || needs_update;
        }

        if let Some(cmd) = command_pipe.read_command().await {
            needs_update = external_command_handler::process(manager, cmd) || needs_update;
            display_server.update_theme_settings(manager.theme_setting.clone());
        }

        //if we need to update the displayed state
        if needs_update {
            match &manager.mode {
                Mode::NormalMode => {
                    let windows: Vec<&Window> = manager.windows.iter().collect();
                    let focused = manager.focused_window();
                    display_server.update_windows(windows, focused);
                    let workspaces: Vec<&Workspace> = manager.workspaces.iter().collect();
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
            match Nanny::new().boot_current_theme() {
                Ok(child) => {
                    child.map(|child| manager.children.insert(child));
                }
                Err(err) => log::error!("Theme loading failed: {}", err),
            }

            leftwm::state::load(manager);
        });

        if manager.reap_requested.swap(false, Ordering::SeqCst) {
            manager.children.reap();
        }

        if manager.reload_requested {
            command_pipe.shutdown().await;
            state_socket.shutdown().await;
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
