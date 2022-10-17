use syslog_tracing::{Facility, Options, Syslog};
use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan};

const IDENTITY: &[u8] = b"leftwm\0";

pub fn add_layer<S>(subscriber: S) -> impl Subscriber + for<'span> LookupSpan<'span>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let log_writer = get_log_writer();
    let layer = tracing_subscriber::fmt::layer().with_writer(log_writer);
    subscriber.with(layer)
}

fn get_log_writer() -> Syslog {
    let identity = std::ffi::CStr::from_bytes_with_nul(IDENTITY).unwrap();
    let options = Options::default();
    let facility = Facility::default();
    Syslog::new(identity, options, facility).unwrap()
}
