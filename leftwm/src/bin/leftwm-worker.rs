use leftwm::utils;
use leftwm_core::Manager;
use std::panic;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "x11rb")]
use x11rb_display_server::X11rbDisplayServer;
#[cfg(feature = "x11rb")]
use x11rb_display_server::X11rbWindowHandle;

#[cfg(feature = "xlib")]
use xlib_display_server::XlibDisplayServer;
#[cfg(feature = "xlib")]
use xlib_display_server::XlibWindowHandle;

fn main() {
    // INFO: This is used when attaching to leftwm-worker with lldb using `--waitfor` to ensure
    //       the process don't run further.
    //       Should probably be removed in the future if it is not needed
    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Set temporary debug logger until config is parsed
    let log_guard = tracing::subscriber::set_default(utils::log::get_subscribers(
        EnvFilter::builder().parse("debug").unwrap(),
    ));
    tracing::info!("leftwm-worker booting...");

    #[cfg(feature = "lefthk")]
    let mut config = leftwm::load();
    // Clear the keybinds so leftwm is not storing them.
    // TODO: Make this more elegant.
    #[cfg(feature = "lefthk")]
    config.clear_keybinds();

    #[cfg(not(feature = "lefthk"))]
    let config = leftwm::load();

    let (subscribers, log_parse_err) = utils::log::parse_log_level(&config.log_level);
    tracing::subscriber::set_global_default(subscribers)
        .expect("Couldn't setup global subscriber (logger)");
    // Drop init log config as the config files have been read and applied to the global default.
    drop(log_guard);
    if let Some(err) = log_parse_err {
        tracing::warn!("Error parsing log_level config: {err}");
    }

    let exit_status = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().expect("ERROR: couldn't init Tokio runtime");
        let _rt_guard = rt.enter();

        match config.backend {
            #[cfg(feature = "xlib")]
            leftwm::Backend::XLib => {
                tracing::info!("Loading XLib backend");
                let manager =
                    Manager::<XlibWindowHandle, leftwm::Config, XlibDisplayServer>::new(config);

                manager.register_child_hook();
                //TODO: Error handling
                rt.block_on(manager.start_event_loop())
            }

            #[cfg(feature = "x11rb")]
            leftwm::Backend::X11rb => {
                tracing::info!("Loading X11rb backend");
                let manager =
                    Manager::<X11rbWindowHandle, leftwm::Config, X11rbDisplayServer>::new(config);

                manager.register_child_hook();
                //TODO: Error handling
                rt.block_on(manager.start_event_loop())
            }
        }
    });

    match exit_status {
        Ok(Ok(())) => tracing::info!("Completed"),
        Ok(Err(err)) => tracing::info!("Completed with event loop error: {}", err),
        Err(err) => tracing::info!("Completed with error: {:?}", err),
    }
}
