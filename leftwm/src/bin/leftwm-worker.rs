use leftwm_core::Manager;
use std::panic;
use xlib_display_server::XlibDisplayServer;

fn main() {
    leftwm::utils::log::setup_logging();

    tracing::info!("leftwm-worker booted!");

    let exit_status = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        #[cfg(feature = "lefthk")]
        let mut config = leftwm::load();
        // Clear the keybinds so leftwm is not storing them.
        // TODO: Make this more elegant.
        #[cfg(feature = "lefthk")]
        config.clear_keybinds();

        #[cfg(not(feature = "lefthk"))]
        let config = leftwm::load();

        let manager = Manager::<leftwm::Config, XlibDisplayServer>::new(config);
        manager.register_child_hook();
        rt.block_on(manager.start_event_loop())
    });

    match exit_status {
        Ok(_) => tracing::info!("Completed"),
        Err(err) => tracing::info!("Completed with error: {:?}", err),
    }
}
