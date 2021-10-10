use slog::{o, Drain};

// Very basic logging used when developing.
// outputs to /tmp/leftwm/leftwm-XXXXXXXXXXXX.log
#[allow(dead_code, clippy::module_name_repetitions)]
pub fn setup_logfile() -> slog_scope::GlobalLoggerGuard {
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
#[allow(dead_code, clippy::module_name_repetitions)]
pub fn setup_logging() -> slog_scope::GlobalLoggerGuard {
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
