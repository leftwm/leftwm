use leftwm_core::{Manager, XlibDisplayServer};

use leftwm_core::logging::{setup_logfile, setup_logging};
use std::panic;

fn main() {
    // let _log_guard = setup_logfile();
    let _log_guard = setup_logging();
    log::info!("leftwm-worker booted!");

    let completed = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        let config = leftwm::load();

        let manager = Manager::<leftwm::Config, XlibDisplayServer>::new(config);
        manager.register_child_hook();

        rt.block_on(manager.event_loop());
    });

    match completed {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}
