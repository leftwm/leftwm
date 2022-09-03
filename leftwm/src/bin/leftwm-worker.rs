use env_logger::Env;
use leftwm::CACHER;
use leftwm_core::{Manager, XlibDisplayServer};
use std::{panic, path::PathBuf};

const LOGGING_FILE: &str = "leftwm.log";

fn main() {
    setup_logger();

    log::info!("starting leftwm-worker");

    let completed = panic::catch_unwind(|| {
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

fn setup_logger() {
    use env_logger::{Builder, Target};

    let log_file = CACHER.get_file(PathBuf::from(LOGGING_FILE)).unwrap();

    Builder::from_env(Env::default().default_filter_or("debug"))
        .target(Target::Pipe(Box::new(log_file)))
        .init();
}
