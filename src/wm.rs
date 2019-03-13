extern crate leftwm;

use leftwm::child_process::Nanny;
use leftwm::*;

fn get_events<T: DisplayServer>(ds: &T) -> Vec<DisplayEvent> {
    ds.get_next_events()
}

fn main() {
    let mut manager = Box::new(Manager::default());
    let mut process_nanny = Box::new(Nanny::new());
    let config = config::load();
    manager.tags = config.get_list_of_tags();
    let mut display_server: XlibDisplayServer = DisplayServer::new(&config);
    let handler = DisplayEventHandler { config };
    loop {
        event_loop(
            &mut manager,
            &mut process_nanny,
            &mut display_server,
            &handler,
        );
    }
}

fn event_loop(
    manager: &mut Manager,
    process_nanny: &mut Nanny,
    display_server: &mut XlibDisplayServer,
    handler: &DisplayEventHandler,
) {
    println!("BOOT:");

    //main event loop
    let mut events_remainder = vec![];
    loop {
        let mut events = get_events(display_server);
        events.append(&mut events_remainder);

        let mut needs_update = false;
        for event in events {
            needs_update = handler.process(manager, event) || needs_update;
        }

        //if we need to update the displayed state
        if needs_update {
            let windows: Vec<&Window> = (&manager.windows).iter().map(|w| w).collect();
            display_server.update_windows(windows);
        }

        //preform any actions requested by the handler
        while !manager.actions.is_empty() {
            if let Some(act) = manager.actions.pop_front() {
                if let Some(event) = display_server.execute_action(act) {
                    events_remainder.push(event);
                }
            }
        }

        //inform all child processes of the new state
        process_nanny.update_children(&manager);
    }
}
