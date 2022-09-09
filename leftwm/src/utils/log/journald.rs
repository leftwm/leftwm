use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;

pub fn add_layer<S>(subscriber: S) -> impl Subscriber
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    subscriber.with(tracing_journald::layer().unwrap())
}
