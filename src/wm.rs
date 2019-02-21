extern crate whata;

use whata::*;

fn get_events<T: DisplayServer>(ds: &T) -> Vec<DisplayEvent>{
    ds.get_next_events()
}

fn main() {
    let mut manager = Manager::default();
    let event_handler = DisplayEventHandler::new();
    let display_server: XlibDisplayServer = DisplayServer::new(&manager.config);

    //main event loop
    loop {
        let events = get_events(&display_server);
        for event in events {
            let needs_update = event_handler.process(&mut manager, event);
            if needs_update {
                let windows: Vec<&Window> = (&manager.windows).iter().map(|w| w).collect();
                display_server.update_windows( windows );
            }
        }
    }

}
