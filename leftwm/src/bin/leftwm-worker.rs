use leftwm_core::{Manager, XlibDisplayServer};
use xdg::BaseDirectories;
use std::{panic, fs::File};

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

        rt.block_on(manager.event_loop());
    });

    match completed {
        Ok(_) => log::info!("Completed"),
        Err(err) => log::error!("Completed with error: {:?}", err),
    }
}

fn setup_logger() {
    use env_logger::{Builder, Target};

    let base_dir = BaseDirectories::new().unwrap();
    let log_file_path = base_dir.place_cache_file(LOGGING_FILE)
        .expect("Couldn't create logging file.");

    let log_file = File::open(log_file_path)
        .expect("Couldn't open log file.");

    Builder::from_default_env()
        .target(Target::Pipe(Box::new(log_file)))
        .init();
}
