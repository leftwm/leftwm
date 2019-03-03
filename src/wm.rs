extern crate leftwm;

use leftwm::*;

fn get_events<T: DisplayServer>(ds: &T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

fn main() {
    println!("BOOT:");
    let mut manager = Manager::default();
    let config = config::load();
    let display_server: XlibDisplayServer = DisplayServer::new(&config);
    let handler = DisplayEventHandler { config: config };

    //main event loop
    loop {
        let events = get_events(&display_server);
        for event in events {
            let needs_update = handler.process(&mut manager, event);

            while manager.actions.len() > 0 {
                if let Some(act) = manager.actions.pop_front(){
                    display_server.execute_action(act);
                }
            }

            //if we need to update the displayed state
            if needs_update {
                let windows: Vec<&Window> = (&manager.windows).iter().map(|w| w).collect();
                display_server.update_windows(windows);
            }
        }
    }
}
