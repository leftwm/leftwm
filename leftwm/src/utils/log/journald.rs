use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;

pub fn add_layer<S>(subscriber: S) -> impl Subscriber + for<'span> LookupSpan<'span>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let layer = tracing_journald::layer()
        .expect("Couldn't setup journald-logger. Are you sure journald is running?");
    subscriber.with(layer)
}
