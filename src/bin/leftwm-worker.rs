mod common;

use leftwm::{
    child_process::{self, Nanny},
    config::Config as _,
    config::ThemeLoader as _,
    models::{FocusBehaviour, FocusManager, Tag},
};

use leftwm::{
    external_command_handler, CommandPipe, DisplayEvent, DisplayEventHandler, DisplayServer,
    Manager, Mode, State, StateSocket, Window, Workspace, XlibDisplayServer,
};
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Arc, Once};

use common::config::Config;

fn get_events<T: DisplayServer<C>, C: leftwm::config::Config>(ds: &mut T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

use slog::{o, Drain};

fn main() {
    //let _log_guard = setup_logfile();
    let _log_guard = setup_logging();
    log::info!("leftwm-worker booted!");

    let completed = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        let config = common::config::load();
        let state = common::state::State;
        let theme_loader = common::theme_setting::ThemeLoader;
        let default_theme = Arc::new(theme_loader.default());

        let focus_manager = FocusManager {
            behaviour: config.focus_behaviour,
            focus_new_windows: config.focus_new_windows,
            ..FocusManager::default()
        };

        let mut tags: Vec<Tag> = config
            .create_list_of_tags()
            .iter()
            .map(|s| Tag::new(s))
            .collect();
        tags.push(Tag {
            id: "NSP".to_owned(),
            hidden: true,
            ..Tag::default()
        });
        // TODO use constructor pattern
        let mut manager = Manager {
            focus_manager,
            tags,
            scratchpads: config.create_list_of_scratchpads(),
            layouts: config.layouts.clone(),
            theme_setting: default_theme.clone(),
            screens: std::vec::Vec::default(),
            windows: std::vec::Vec::default(),
            workspaces: std::vec::Vec::default(),
            mode: leftwm::Mode::default(),
            active_scratchpads: std::collections::HashMap::default(),
            actions: std::collections::VecDeque::default(),
            frame_rate_limitor: Default::default(),
            children: leftwm::child_process::Children::default(),
            reap_requested: std::sync::Arc::default(),
            reload_requested: Default::default(),
            max_window_width: config.max_window_width,
        };

        child_process::register_child_hook(manager.reap_requested.clone());

        let config = Arc::new(config);
        let mut display_server = XlibDisplayServer::new(config.clone(), default_theme);
        let handler = DisplayEventHandler {
            config: config.clone(),
        };

        rt.block_on(event_loop(
            &mut manager,
            &mut display_server,
            &handler,
            config,
            state,
            theme_loader,
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

async fn timeout(mills: u64) {
    use tokio::time::{sleep, Duration};
    sleep(Duration::from_millis(mills)).await;
}

async fn event_loop(
    manager: &mut Manager,
    display_server: &mut XlibDisplayServer<Arc<Config>>,
    handler: &DisplayEventHandler<Arc<Config>>,
    config: Arc<Config>,
    state: common::state::State,
    theme_loader: common::theme_setting::ThemeLoader,
) {
    let socket_file = place_runtime_file("current_state.sock")
        .expect("ERROR: couldn't create current_state.sock");
    let mut state_socket = StateSocket::default();
    state_socket
        .listen(socket_file)
        .await
        .expect("ERROR: couldn't connect to current_state.sock");

    let pipe_file =
        place_runtime_file("commands.pipe").expect("ERROR: couldn't create commands.pipe");
    let mut command_pipe = CommandPipe::new(pipe_file)
        .await
        .expect("ERROR: couldn't connect to commands.pipe");

    //start the current theme
    let after_first_loop: Once = Once::new();

    //main event loop
    let mut event_buffer = vec![];
    loop {
        if manager.mode == Mode::Normal {
            state_socket.write_manager_state(manager).await.ok();
        }
        display_server.flush();

        let mut needs_update = false;
        tokio::select! {
            _ = display_server.wait_readable(), if event_buffer.is_empty() => {
                event_buffer.append(&mut get_events(display_server));
                continue;
            }
            //Once in a blue moon we miss the focus event,
            //This is to double check that we know which window is currently focused
            _ = timeout(100), if event_buffer.is_empty() && manager.focus_manager.behaviour == FocusBehaviour::Sloppy => {
                let mut focus_event = display_server.verify_focused_window();
                event_buffer.append(&mut focus_event);
                continue;
            }
            Some(cmd) = command_pipe.read_command(), if event_buffer.is_empty() => {
                needs_update = external_command_handler::process(manager, &state, &config, &theme_loader, cmd) || needs_update;
                display_server.update_theme_settings(manager.theme_setting.clone());
            }
            else => {
                event_buffer.drain(..).for_each(|event| needs_update = handler.process(manager, &state, event) || needs_update);
            }
        }

        //if we need to update the displayed state
        if needs_update {
            match &manager.mode {
                Mode::Normal => {
                    let windows: Vec<&Window> = manager.windows.iter().collect();
                    let focused = manager.focused_window();
                    display_server.update_windows(windows, focused, manager);
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
                    display_server.update_windows(windows, focused, manager);
                }
            }
        }

        //preform any actions requested by the handler
        while !manager.actions.is_empty() {
            if let Some(act) = manager.actions.pop_front() {
                if let Some(event) = display_server.execute_action(act) {
                    event_buffer.push(event);
                }
            }
        }

        //after the very first loop run the 'up' scripts (global and theme). we need the unix
        //socket to already exist.
        after_first_loop.call_once(|| {
            match Nanny::run_global_up_script() {
                Ok(child) => {
                    child.map(|child| manager.children.insert(child));
                }
                Err(err) => log::error!("Global up script faild: {}", err),
            }
            match Nanny::boot_current_theme() {
                Ok(child) => {
                    child.map(|child| manager.children.insert(child));
                }
                Err(err) => log::error!("Theme loading failed: {}", err),
            }

            state.load(manager);
        });

        if manager.reap_requested.swap(false, Ordering::SeqCst) {
            manager.children.reap();
        }

        if manager.reload_requested {
            state_socket.shutdown().await;
            break;
        }
    }
}

// Very basic logging used when developing.
// outputs to /tmp/leftwm/leftwm-XXXXXXXXXXXX.log
#[allow(dead_code)]
fn setup_logfile() -> slog_scope::GlobalLoggerGuard {
    use chrono::Local;
    use std::fs;
    use std::fs::OpenOptions;
    let date = Local::now();
    let path = "/tmp/leftwm";
    let _droppable = fs::create_dir_all(path);
    let log_path = format!("{}/leftwm-{}.log", path, date.format("%Y%m%d%H%M"));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("ERROR: couldn't open log file");
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let envlogger = slog_envlogger::LogBuilder::new(drain)
        .parse(&std::env::var("RUST_LOG").unwrap_or_else(|_| "trace".into()))
        .build()
        .ignore_res();
    let logger = slog::Logger::root(slog_async::Async::default(envlogger).ignore_res(), o!());
    slog_stdlog::init().unwrap_or_else(|err| {
        eprintln!("failed to setup logging: {}", err);
    });
    slog_scope::set_global_logger(logger)
}

/// Log to both stdout and journald.
#[allow(dead_code)]
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
