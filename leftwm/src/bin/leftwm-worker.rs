use leftwm_core::{Manager, XlibDisplayServer};
use std::panic;

fn main() {
    leftwm::utils::log::setup_logging();

    log::info!("leftwm-worker booted!");

    let exit_status = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        let config = leftwm::load();

        let manager = Manager::<leftwm::Config, XlibDisplayServer>::new(config);
        manager.register_child_hook();
        rt.block_on(manager.start_event_loop())
    });

    match exit_status {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}
