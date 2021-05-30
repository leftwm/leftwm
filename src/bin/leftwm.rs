use leftwm::child_process::{self, Nanny};
use std::env;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Starts leftwm programs.
///
/// If no arguments are passed, starts `leftwm-worker`. If arguments are passed, starts
/// `leftwm-{check, command, state, theme}` as specified, and passes along any extra arguments.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        execute_subcommand(args);
        return;
    }

    // If not invoked with a subcommand, start leftwm.
    if let Ok(current_exe) = std::env::current_exe() {
        //boot everything in ~/.config/autostart
        let mut children = Nanny::autostart();

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
            while worker
                .try_wait()
                .expect("failed to wait on worker")
                .is_none()
            {
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
            // Left is too fast for some logging managers. We need to
            // wait to give the logging manager a second to boot.
            #[cfg(feature = "slow-dm-fix")]
            {
                let delay = std::time::Duration::from_millis(2000);
                std::thread::sleep(delay);
            }
        }
    }
}

/// Executes a subcommand.
///
/// If a valid subcommand is supplied, executes that subcommand, passing `args` to the program.
/// Valid subcommands are `check`, `command`, `state` and `theme`.
/// Prints an error to STDERR and returns `false` if an invalid subcommand is supplied.
///
/// # Arguments
///
/// + `args` - The command line arguments leftwm was called with.
///
/// # Panics
///
/// Panics if `args` has length < 2.
fn execute_subcommand(args: Vec<String>) -> bool {
    if ["check", "command", "state", "theme"]
        .iter()
        .any(|x| x == &args[1])
    {
        println!("Running leftwm-{} with args {:?}", &args[1], &args[2..]);
        true
    } else {
        eprintln!("Invalid command '{}'.", &args[1]);
        false
    }
}
