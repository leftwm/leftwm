use tracing::{metadata::LevelFilter, Subscriber};
use tracing_subscriber::{filter::ParseError, layer::SubscriberExt, EnvFilter};

#[cfg(feature = "journald-log")]
mod journald;

#[cfg(feature = "file-log")]
pub mod file;

#[cfg(feature = "sys-log")]
mod sys;

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn parse_log_level(level_regex: &str) -> (impl Subscriber, Option<ParseError>) {
    let mut parse_err = None;
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .parse(level_regex)
        .unwrap_or_else(|err| {
            parse_err = Some(err);
            EnvFilter::builder().parse("debug").unwrap()
        });
    (get_subscribers(filter), parse_err)
}

#[allow(clippy::let_and_return)]
pub fn get_subscribers(filter: EnvFilter) -> impl Subscriber {
    let subscriber = tracing_subscriber::registry().with(filter);

    #[cfg(feature = "journald-log")]
    let subscriber = journald::add_layer(subscriber);

    #[cfg(feature = "file-log")]
    let subscriber = file::add_layer(subscriber);

    #[cfg(feature = "sys-log")]
    let subscriber = sys::add_layer(subscriber);

    subscriber
}
