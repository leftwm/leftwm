use slog::{o, Drain};
use std::result;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};

/// Custom Drain logic
struct RuntimeLevelFilter<D> {
    drain: D,
    on: Arc<atomic::AtomicBool>,
}

impl<D> Drain for RuntimeLevelFilter<D>
where
    D: Drain,
{
    type Ok = Option<D::Ok>;
    type Err = Option<D::Err>;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> result::Result<Self::Ok, Self::Err> {
        let current_level = if self.on.load(Ordering::Relaxed) {
            slog::Level::Trace
        } else {
            slog::Level::Info
        };

        if record.level().is_at_least(current_level) {
            self.drain.log(record, values).map(Some).map_err(Some)
        } else {
            Ok(None)
        }
    }
}

// Very basic logging used when developing.
// outputs to /tmp/leftwm/leftwm-XXXXXXXXXXXX.log
#[cfg(feature = "slog-term")]
#[allow(dead_code, clippy::module_name_repetitions)]
pub fn setup_logfile() -> slog_scope::GlobalLoggerGuard {
    // use chrono::Local;
    use std::fs;
    use std::fs::OpenOptions;
    // let date = Local::now();
    let date = time::OffsetDateTime::now_local().expect("Localtime: ");
    let path = "/tmp/leftwm";
    let _droppable = fs::create_dir_all(path);
    let log_path = format!(
        "{}/leftwm-{}.log",
        path,
        date.format(time::macros::format_description!(
            "[year][month][day][hour][minute]"
        ))
        .expect("Formated localtime: ")
    );
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

/// Log to both stdout and journald depending on build flags.
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

#[cfg(feature = "slog-journald")]
#[allow(dead_code)]
pub fn dyn_logger() {
    // atomic variable controlling logging level
    let on = Arc::new(atomic::AtomicBool::new(false));

    // let decorator = slog_term::TermDecorator::new().build();
    // let drain = slog_term::FullFormat::new(decorator).build();
    let drain = slog_journald::JournaldDrain.ignore_res();
    let drain = RuntimeLevelFilter {
        drain,
        on: on.clone(),
    }
    .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let _log = slog::Logger::root(drain, o!());

    // switch level in your code
    on.store(true, Ordering::Relaxed);
    log::info!("Logging with dyn-logger.");
}
