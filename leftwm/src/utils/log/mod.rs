use tracing::Subscriber;

mod journald;

pub fn setup_logging() {
    let subscriber = get_subscriber();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Couldn't setup global subscriber (logger)");
}

fn get_subscriber() -> impl Subscriber {
    let subscriber = tracing_subscriber::registry();

    #[cfg(feature = "journald")]
    let subscriber = journald::add_layer(subscriber);

    subscriber
}
