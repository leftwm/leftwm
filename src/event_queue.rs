use super::utils::command::Command;
use super::utils::screen::Screen;
use super::utils::window;

pub enum EventQueueItem {
    Command(Command, Option<String>),
    WindowCreate(window::Window),
    WindowDestroy(window::WindowHandle),
    ScreenCreate(Screen),
}

//pub type EventQueue = Arc<Mutex<VecDeque<EventQueueItem>>>;
//
//pub fn new() -> EventQueue {
//    let q: VecDeque<EventQueueItem> = VecDeque::new();
//    Arc::new(Mutex::new(q))
//}
