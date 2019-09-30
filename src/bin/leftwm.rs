extern crate leftwm;
use leftwm::child_process::Nanny;
use std::env;
use std::process::Command;

use nix::sys::wait;
use nix::sys::signal::{self, SigHandler, Signal};

extern fn handle_sigchld(signal: libc::c_int) {
    // TODO: replaced with Signal::try_from() in to-be-released nix crate
    let signal = Signal::from_c_int(signal).unwrap();
    if signal == Signal::SIGCHLD {
        let _ = wait::wait();
    }
}

fn install_sigchld_handler() {
    let handler = SigHandler::Handler(handle_sigchld);
    unsafe { signal::signal(Signal::SIGCHLD, handler) }.unwrap();
}

fn main() {
    install_sigchld_handler();
    if let Ok(booter) = std::env::current_exe() {
        //boot everything in ~/.config/autostart
        Nanny::new().autostart();

        //Fix for JAVA apps so they repaint correctly
        env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");

        let mut worker = booter.clone();
        worker.pop();
        worker.push("leftwm-worker");

        loop {
            Command::new(&worker)
                .status()
                .expect("failed to start leftwm");

            //left it to fast for some logging managers. We need to wait to give the logging
            //manager a second to boot
            let delay = std::time::Duration::from_millis(2000);
            std::thread::sleep(delay);
        }
    }
}
