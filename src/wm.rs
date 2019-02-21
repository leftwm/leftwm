extern crate whata;

fn get_events<T: whata::DisplayServer>(ds: &T) -> Vec<whata::DisplayEvent>{
    ds.get_next_events()
}

fn main() {
    let mut manager = whata::Manager::default();
    let event_handler = whata::DisplayEventHandler::new();
    let display_server: whata::XlibDisplayServer = whata::DisplayServer::new(&manager.config);

    //main event loop
    loop {
        let events = get_events(&display_server);
        for event in events {
            let needs_update = event_handler.process(&mut manager, event);
        }
    }
}
