use leftwm_core::Manager;
use x11rb_display_server::X11rbWindowHandle;
use xlib_display_server::XlibWindowHandle;
use std::{env, panic, process::exit};

#[cfg(feature = "x11rb")]
use x11rb_display_server::X11rbDisplayServer;
#[cfg(feature = "xlib")]
use xlib_display_server::XlibDisplayServer;

fn main() {
    // INFO: This is used when attaching to leftwm-worker with lldb using `--waitfor` to ensure
    //       the process don't run further.
    //       Should probably be removed in the future if it is not needed
    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_secs(1));

    leftwm::utils::log::setup_logging();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        tracing::error!("You need to specify a backend, as argument.");
        tracing::error!("Backends must be one of the following: xlib, x11rb");
        exit(1);
    }

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

        match args.get(1) {
            #[cfg(feature = "xlib")]
            Some(name) if name == "xlib" => {
                let manager = Manager::<XlibWindowHandle, leftwm::Config, XlibDisplayServer>::new(config);

                manager.register_child_hook();
                //TODO: Error handling
                rt.block_on(manager.start_event_loop())
            }

            #[cfg(feature = "x11rb")]
            Some(name) if name == "x11rb" => {
                let manager = Manager::<X11rbWindowHandle, leftwm::Config, X11rbDisplayServer>::new(config);

                manager.register_child_hook();
                //TODO: Error handling
                rt.block_on(manager.start_event_loop())
            }
            _ => {
                tracing::error!("Invalid backend.");
                tracing::error!("Backends must be one of the following: xlib, x11rb");
                exit(1)
            }
        }
    });

    match exit_status {
        Ok(Ok(_)) => tracing::info!("Completed"),
        Ok(Err(err)) => tracing::info!("Completed with event loop error: {}", err),
        Err(err) => tracing::info!("Completed with error: {:?}", err),
    }
}
