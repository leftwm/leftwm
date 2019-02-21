use super::DisplayEvent;
use super::Manager;

pub struct DisplayEventHandler {}

impl DisplayEventHandler {
    pub fn new() -> DisplayEventHandler {
        DisplayEventHandler {}
    }

    /*
     * process a collection of events, and apply them changes to a manager
     * returns true if changes need to be rendered
     */
    pub fn process(&self, manager: &mut Manager, event: DisplayEvent) -> bool {
        match event {
            _ => {}
            //DisplayEvent::WindowCreate(w) => self.on_new_window(w),
            //DisplayEvent::WindowDestroy(window_handle) => self.on_destroy_window(window_handle),
            //DisplayEvent::ScreenCreate(s) => self.on_new_screen(s),
            //DisplayEvent::FocusedWindow(window_handle) => {
            //    self.update_focused_window(window_handle)
            //}
            //EventQueueItem::Command(command, value) => self.on_command(command, value),
        }
        false
    }
}

//fn active_workspace(&mut self) -> Option<&mut Workspace> {
//    let index = self.focused_workspace_history[0];
//    if index < self.workspaces.len() {
//        return Some(&mut self.workspaces[index]);
//    }
//    None
//}

//fn focused_window(&mut self) -> Option<&mut Window> {
//    if self.focused_window_history.len() == 0 {
//        return None;
//    }
//    let handle = self.focused_window_history[0];
//    for w in &mut self.windows {
//        if handle == w.handle {
//            return Some(w);
//        }
//    }
//    None
//}

//pub fn update_windows(&mut self) {
//    {
//        let all_windows = &mut self.windows;
//        let all: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
//        for w in all {
//            w.visable = w.tags.is_empty();
//        } // if not tagged force it to display
//        for ws in &mut self.workspaces {
//            let windows: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
//            ws.update_windows(windows);
//        }
//    }
//    let to_update: Vec<&Window> = (&self.windows).iter().map(|w| w).collect();
//    self.ds.update_windows(to_update);
//}

//fn on_new_window(&mut self, a_window: Window) {
//    //don't add the window if the manager already knows about it
//    for w in &self.windows {
//        if w.handle == a_window.handle {
//            return;
//        }
//    }
//    let mut window = a_window;
//    if let Some(ws) = self.active_workspace() {
//        window.tags = ws.tags.clone();
//    }
//    self.windows.push(window);
//    self.update_windows();
//}

//fn on_new_screen(&mut self, screen: Screen) {
//    let tag_index = self.workspaces.len();
//    let mut workspace = Workspace::from_screen(&screen);
//    workspace.name = tag_index.to_string();
//    let next_tag = self.tags[tag_index].clone();
//    workspace.show_tag(next_tag);
//    self.workspaces.push(workspace);
//    self.screens.push(screen);
//}

//fn on_destroy_window(&mut self, handle: WindowHandle) {
//    let index = self.windows.iter().position(|w| w.handle == handle);
//    if let Some(i) = index {
//        self.windows.remove(i);
//    }
//    self.update_windows();
//}

//pub fn on_event(&mut self, event: EventQueueItem) {
//    match event {
//        EventQueueItem::WindowCreate(w) => self.on_new_window(w),
//        EventQueueItem::WindowDestroy(window_handle) => self.on_destroy_window(window_handle),
//        EventQueueItem::ScreenCreate(s) => self.on_new_screen(s),
//        EventQueueItem::FocusedWindow(window_handle) => {
//            self.update_focused_window(window_handle)
//        }
//        EventQueueItem::Command(command, value) => self.on_command(command, value),
//    }
//}

// /*
// * set the focused window if we know about the handle
// */
//fn update_focused_window(&mut self, handle: WindowHandle) {
//    while self.focused_window_history.len() > 10 {
//        self.focused_window_history.pop_back();
//    }
//    //self.focused_window_handle = None;
//    for w in &self.windows {
//        if w.handle == handle {
//            if let WindowHandle::XlibHandle(xlibh) = &handle {
//                println!("FOCUSED: {}", xlibh);
//            }
//            self.focused_window_history.push_front(handle);
//            return;
//        }
//    }
//}

// /*
// * change the active workspace to view a given set of tags
// */
//fn goto_tags(&mut self, tags: Vec<&String>) {
//    if let Some(workspace) = self.active_workspace() {
//        if tags.len() == 1 {
//            workspace.show_tag(tags[0].clone());
//        }
//        self.update_windows();
//    }
//}

// /*
// * move the current focused window to a given tag
// */
//fn move_to_tags(&mut self, tags: Vec<&String>) {
//    if let Some(window) = self.focused_window() {
//        window.clear_tags();
//        for s in tags {
//            window.tag(s.clone());
//        }
//        self.update_windows();
//    }
//}

// /*
// * route a command to its correct handler
// */
//pub fn on_command(&mut self, command: Command, value: Option<String>) {
//    match command {
//        Command::Execute => {}
//        //CloseWindow => {},
//        //SwapWorkspaces => {},
//        Command::GotoTag => {
//            if let Some(val) = &value {
//                self.goto_tags(vec![val]);
//            }
//        }
//        Command::MoveToTag => {
//            if let Some(val) = &value {
//                self.move_to_tags(vec![val]);
//            }
//        }

//        //MovetoWorkspace => {},
//        _ => {}
//    }
//}
//}
