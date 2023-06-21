use tracing::{metadata::LevelFilter, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

#[cfg(feature = "journald-log")]
mod journald;

#[cfg(feature = "file-log")]
pub mod file;

#[cfg(feature = "sys-log")]
mod sys;

pub fn setup_logging() {
    let subscribers = get_subscribers();

    tracing::subscriber::set_global_default(subscribers)
        .expect("Couldn't setup global subscriber (logger)");
}

#[allow(clippy::let_and_return)]
fn get_subscribers() -> impl Subscriber {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    let subscriber = tracing_subscriber::registry().with(env_filter);

    #[cfg(feature = "journald-log")]
    let subscriber = journald::add_layer(subscriber);

    #[cfg(feature = "file-log")]
    let subscriber = file::add_layer(subscriber);

    #[cfg(feature = "sys-log")]
    let subscriber = sys::add_layer(subscriber);

    subscriber
}
