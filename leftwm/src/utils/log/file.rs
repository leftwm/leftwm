use std::path::Path;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;

const LOG_DIR: &str = "~/.cache/leftwm";
const LOG_FILE_NAME: &str = "log.log";

pub fn add_layer<S>(subscriber: S) -> impl Subscriber + for<'span> LookupSpan<'span>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    prepare_path(&Path::new(LOG_DIR));

    let log_writer = get_log_writer();
    let layer = tracing_subscriber::fmt::layer();

    subscriber.with(layer.with_writer(log_writer))
}

fn prepare_path(path: &Path) {
    std::fs::create_dir_all(path).expect(&format!("Couldn't create log directory: {}", LOG_DIR));
}

fn get_log_writer() -> NonBlocking {
    let writer = tracing_appender::rolling::never(LOG_DIR, LOG_FILE_NAME);
    let (non_blocking, _guard) = tracing_appender::non_blocking(writer);

    non_blocking

}
