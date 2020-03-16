use leftwm::child_process::Nanny;
use std::env;
use std::process::Command;

fn main() {
    if let Ok(current_exe) = std::env::current_exe() {
        //boot everything in ~/.config/autostart
        Nanny::new().autostart();

        //Fix for JAVA apps so they repaint correctly
        env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");

        let worker_path = current_exe.with_file_name("leftwm-worker");

        loop {
            Command::new(&worker_path)
                .status()
                .expect("failed to start leftwm");

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
