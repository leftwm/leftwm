use leftwm::child_process::{self, Nanny};
use std::env;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn main() {
    if let Ok(current_exe) = std::env::current_exe() {
        //boot everything in ~/.config/autostart
        let mut children = Nanny::new().autostart();

        let flag = Arc::new(AtomicBool::new(false));
        child_process::register_child_hook(flag.clone());

        //Fix for JAVA apps so they repaint correctly
        env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");

        let worker_path = current_exe.with_file_name("leftwm-worker");

        loop {
            let mut worker = Command::new(&worker_path)
                .spawn()
                .expect("failed to start leftwm");

            // Wait until worker exits.
            while let None = worker.try_wait().expect("failed to wait on worker") {
                // Not worker, then it might be autostart programs.
                children.reap();
                // Wait for SIGCHLD signal flag to be set.
                while !flag.swap(false, Ordering::SeqCst) {
                    nix::unistd::pause();
                }
                // Either worker or autostart program exited.
            }

            // TODO: either add more details or find a better workaround.
            //
            // Left is to fast for some logging managers. We need to
            // wait to give the logging manager a second to boot.
            #[cfg(feature = "slow-dm-fix")]
            {
                let delay = std::time::Duration::from_millis(2000);
                std::thread::sleep(delay);
            }
        }
    }
}
