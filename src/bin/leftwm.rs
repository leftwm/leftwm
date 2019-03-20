extern crate leftwm;
use leftwm::child_process::Nanny;
use std::process::Command;

fn main() {
    if let Ok(booter) = std::env::current_exe() {
        //boot everything in ~/.config/autostart
        Nanny::new().autostart();

        let mut worker = booter.clone();
        worker.pop();
        worker.push("leftwm-worker");

        loop {
            Command::new(&worker)
                .status()
                .expect("failed to start leftwm");
        }
    }
}
