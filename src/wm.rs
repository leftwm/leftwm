
mod utils;
mod display_servers;
mod xwrap;
use display_servers::*;

fn main() {


    let ds:MockDisplayServer = DisplayServer::new();
    //let ds:XlibDisplayServer = DisplayServer::new();

    let windows = ds.find_all_windows();

    for window in windows {
        println!("window: {:#?} ", window);
    }



    //let xw = XWrap::new();
    //xw.init();

    //let mut windows = WaWindow::find_all(&xw);

    //loop {
    //    let raw_event = xw.get_next_event();
    //    //event_handler::handle_event(raw_event);
    //}

}

