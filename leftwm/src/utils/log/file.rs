use std::fmt::Debug;
use std::path::{Path, PathBuf};

use tracing::Subscriber;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan};
use xdg::BaseDirectories;

const LOG_PREFIX: &str = "leftwm";
const LOG_FILE_NAME: &str = "log.log";

pub fn add_layer<S>(subscriber: S) -> impl Subscriber + for<'span> LookupSpan<'span>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let log_path = get_log_path();

    let log_dir_path = get_log_dir(&log_path);
    let log_file_name = get_log(&log_path);

    create_dirs(&log_dir_path);

    let log_writer = get_log_writer(log_dir_path, log_file_name);
    let layer = tracing_subscriber::fmt::layer().with_writer(log_writer);
    subscriber.with(layer)
}

#[must_use]
/// Gets the path to the log
///
/// # Panics
/// - If HOME is not set
/// - If path permissions are not at least 0700
pub fn get_log_path() -> Box<Path> {
    let cache_dir = BaseDirectories::with_prefix(LOG_PREFIX).unwrap();
    cache_dir
        .place_state_file(LOG_FILE_NAME)
        .unwrap()
        .into_boxed_path()
}

fn create_dirs<P: AsRef<Path> + Debug>(path: P) {
    std::fs::create_dir_all(&path)
        .unwrap_or_else(|_| panic!("Couldn't create directory-path: {path:?}"));
}

fn get_log_writer<P: AsRef<Path>>(log_dir: P, log_file: P) -> RollingFileAppender {
    tracing_appender::rolling::never(log_dir, log_file)
}

fn get_log_dir<P: AsRef<Path> + Clone>(path: P) -> Box<Path> {
    let mut log_dir = path.as_ref().to_path_buf();
    log_dir.pop();
    log_dir.into_boxed_path()
}

/// Gets the log file's filename
///
/// # Panics
/// - If HOME is not set
/// - If path permissions are not at least 0700
pub fn get_log<P: AsRef<Path>>(path: P) -> Box<Path> {
    let file_name = path.as_ref().file_name().unwrap().to_owned();

    let file_name = PathBuf::from(file_name);
    file_name.into_boxed_path()
}
