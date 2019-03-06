extern crate leftwm;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use leftwm::child_process::Nanny;
use leftwm::*;

fn get_events<T: DisplayServer>(ds: &T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

fn main() -> Result<(), Error> {
    {
        let mut manager = Box::new(Manager::default());
        let mut process_nanny = Box::new(Nanny::new());
        let config = config::load();
        let display_server: XlibDisplayServer = DisplayServer::new(&config);
        let handler = DisplayEventHandler { config: config };

        let stopit = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&stopit))?;
        signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&stopit))?;
        while !stopit.load(Ordering::Relaxed) {
            event_loop(&mut manager, &mut process_nanny, &display_server, &handler);
        }
    }

    //NOTE: at this point all the things from the last block are cleaned up :)
    println!("EXITING!!!");
    //let mut file = std::fs::File::create("/home/lex/foo.txt")?;
    //file.write_all(b"Hello, world!")?;

    Ok(())
}

fn event_loop(
    manager: &mut Manager,
    process_nanny: &mut Nanny,
    display_server: &XlibDisplayServer,
    handler: &DisplayEventHandler,
) {
    println!("BOOT:");

    //main event loop
    loop {
        let events = get_events(display_server);
        for event in events {
            let needs_update = handler.process(manager, event);

            while manager.actions.len() > 0 {
                if let Some(act) = manager.actions.pop_front() {
                    display_server.execute_action(act);
                }
            }

            //if we need to update the displayed state
            if needs_update {
                let windows: Vec<&Window> = (&manager.windows).iter().map(|w| w).collect();
                display_server.update_windows(windows);
            }

            //inform all child processes of the new state
            process_nanny.update_children(&manager);
        }
    }
}
