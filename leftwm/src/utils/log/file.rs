use std::path::Path;

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
    prepare_path(&Path::new(LOG_PREFIX));

    let log_writer = get_log_writer();
    let layer = tracing_subscriber::fmt::layer().with_writer(log_writer);
    subscriber.with(layer)
}

fn prepare_path(path: &Path) {
    std::fs::create_dir_all(path).expect(&format!("Couldn't create log directory: {}", LOG_PREFIX));
}

fn get_log_writer() -> RollingFileAppender {
    let cache_dir = BaseDirectories::with_prefix(LOG_PREFIX).unwrap();
    let log_path = cache_dir.place_state_file(LOG_FILE_NAME).unwrap();
    tracing_appender::rolling::never(LOG_PREFIX, log_path)
}
