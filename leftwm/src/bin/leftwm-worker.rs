use leftwm_core::{Manager, XlibDisplayServer};
use slog::{o, Drain};
use std::panic;

fn main() {
    let _log_guard = setup_logfile();
    // let _log_guard = setup_logging();
    log::info!("leftwm-worker booted!");

    let exit_status = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        let config = leftwm::load();

        let manager = Manager::<leftwm::Config, XlibDisplayServer>::new(config);
        manager.register_child_hook();
        rt.block_on(manager.event_loop())
    });

    match exit_status {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}

// Very basic logging used when developing.
// outputs to /tmp/leftwm/leftwm-XXXXXXXXXXXX.log
#[allow(dead_code)]
fn setup_logfile() -> slog_scope::GlobalLoggerGuard {
    use std::fs;
    use std::fs::OpenOptions;
    use time_leftwm::{format_description, OffsetDateTime};
    let date = OffsetDateTime::now_local();
    let path = "/tmp/leftwm";
    let _droppable = fs::create_dir_all(path);
    let format_string =
        format_description::parse("[year][month][day][hour][minute]").expect("Error with Time");
    let date_formatted: String = if let Ok(df) = date {
        df.format(&format_string)
            .unwrap_or_else(|_| String::from("time-parse-error"))
    } else {
        let mut d = OffsetDateTime::now_utc()
            .format(&format_string)
            .unwrap_or_else(|_| String::from("time-parse-error"));
        d.push_str("UTC");
        d
    };
    let log_path = format!("{}/leftwm-{}.log", path, date_formatted);
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
