#[cfg(feature = "lefthk")]
use lefthk_core::{config::Config, worker::Worker};
#[cfg(feature = "logging")]
use slog::{o, Drain};
#[cfg(feature = "lefthk")]
use xdg::BaseDirectories;

#[cfg(feature = "lefthk")]
fn main() {
    #[cfg(feature = "logging")]
    let _log_guard = setup_logging();
    log::info!("lefthk-worker booted!");
    let completed = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();
        let config = leftwm::load();
        let path = BaseDirectories::with_prefix("leftwm-lefthk")
            .expect("ERROR: could not find base directory");
        let mut worker = Worker::new(config.mapped_bindings(), path);

        rt.block_on(worker.event_loop());
    });

    match completed {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}

#[cfg(not(feature = "lefthk"))]
fn main() {}

/// Log to both stdout and journald.
#[allow(dead_code)]
#[cfg(feature = "logging")]
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
