extern crate whata;

use whata::*;

fn get_events<T: DisplayServer>(ds: &T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

fn main() {
    let mut manager = Manager::default();
    let config = config::load();
    let display_server: XlibDisplayServer = DisplayServer::new(&config);
    let handler = DisplayEventHandler {
        config: config
    };

    //main event loop
    loop {
        let events = get_events(&display_server);
        for event in events {
            let needs_update = handler.process(&mut manager, event);
            //println!("state: {:?}", manager);
            if needs_update {
                let windows: Vec<&Window> = (&manager.windows).iter().map(|w| w).collect();
                display_server.update_windows(windows);
            }
        }
    }
}
