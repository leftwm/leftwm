use x11_dl::xlib;

mod xwrap;
use xwrap::WaWindow;
use xwrap::XWrap;
mod event_handler;

fn main() {

    let xw = XWrap::new();
    xw.init();

    let mut windows = WaWindow::find_all(&xw);

    loop {
        let raw_event = xw.get_next_event();
        event_handler::handle_event(raw_event);
    }

}

