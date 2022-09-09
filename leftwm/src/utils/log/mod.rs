use tracing::Subscriber;

mod journald;
mod file;

pub fn setup_logging() {
    let subscribers = get_subscribers();

    tracing::subscriber::set_global_default(subscribers)
        .expect("Couldn't setup global subscriber (logger)");
}

fn get_subscribers() -> impl Subscriber {
    let subscriber = tracing_subscriber::registry();

    #[cfg(feature = "journald-log")]
    let subscriber = journald::add_layer(subscriber);

    #[cfg(feature = "file-log")]
    let subscriber = file::add_layer(subscriber);

    subscriber
}
