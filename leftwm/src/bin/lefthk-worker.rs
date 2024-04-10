use lefthk_core::{config::Config, worker::Worker};
use leftwm::utils;
use tracing_subscriber::EnvFilter;
use xdg::BaseDirectories;

fn main() {
    tracing::subscriber::set_global_default(utils::log::get_subscribers(
        EnvFilter::from_default_env(),
    ))
    .expect("Couldn't setup global subscriber (logger)");

    tracing::info!("lefthk-worker booted!");

    let exit_status = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();
        let config = leftwm::load();
        let path = BaseDirectories::with_prefix("leftwm-lefthk")
            .expect("ERROR: could not find base directory");

        rt.block_on(Worker::new(config.mapped_bindings(), path).event_loop());
    });

    match exit_status {
        Ok(()) => tracing::info!("Completed"),
        Err(err) => tracing::error!("Completed with error: {:?}", err),
    }
}
