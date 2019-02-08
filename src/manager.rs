use super::display_servers::DisplayServer;
use super::event_queue::EventQueueItem;
use super::utils::screen::Screen;
use super::utils::window::Window;
use super::utils::window::WindowHandle;
use super::utils::window;
use super::utils::workspace::Workspace;

pub trait DisplayEventHandler {
    fn on_new_window(&mut self, window: Window);
    fn on_new_screen(&mut self, screen: Screen);
}

#[derive(Clone)]
pub struct Manager<DM: DisplayServer> {
    pub windows: Vec<Window>,
    pub screens: Vec<Screen>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>, //list of all known tags
    pub ds: DM,
    active_wp_index: usize,
}

impl<DM: DisplayServer> Manager<DM> {
    pub fn new() -> Manager<DM> {
        Manager {
            windows: Vec::new(),
            ds: DM::new(),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            active_wp_index: 0,
        }
    }

    fn active_workspace(&self) -> Option<&Workspace> {
        if self.active_wp_index < self.workspaces.len() {
            return Some(&self.workspaces[self.active_wp_index]);
        }
        None
    }

    pub fn update_windows(&mut self) {
        {
            let all_windows = &mut self.windows;
            let all: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
            for w in all {
                w.visable = w.tags.is_empty();
            } // if not tagged force it to display
            for ws in &mut self.workspaces {
                let windows: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
                ws.update_windows(windows);
            }
        }
        let to_update: Vec<&Window> = (&self.windows).iter().map(|w| w).collect();
        self.ds.update_windows(to_update);
    }

    fn on_new_window(&mut self, a_window: Window) {
        println!("on_new_window");
        //don't add the window if the manager already knows about it
        for w in &self.windows {
            if w.handle == a_window.handle {
                return;
            }
        }

        let mut window = a_window;

        if let Some(ws) = self.active_workspace() {
            window.tags = ws.tags.clone();
        }
        self.windows.push(window);
        self.update_windows();
    }

    fn on_new_screen(&mut self, screen: Screen) {
        println!("on_new_screen");
        let tag_index = self.workspaces.len();
        let mut workspace = Workspace::from_screen(&screen);
        let next_tag = self.tags[tag_index].clone();
        workspace.show_tag(next_tag);
        self.workspaces.push(workspace);
        self.screens.push(screen);
    }

    fn on_destroy_window(&mut self, handle: WindowHandle) {
        println!("on_destroy_window");
        let index = self.windows.iter().position(|w| window::handles_equal(&w.handle, &handle) );
        if let Some(i) = index {
            println!("REMOVED WINDOW: {}", i);
            self.windows.remove(i);
        }
        let to_update: Vec<&Window> = (&self.windows).iter().map(|w| w).collect();
        self.ds.update_windows(to_update);
    }

    pub fn on_event(&mut self, event: EventQueueItem) {
        match event {
            EventQueueItem::WindowCreate(w) => self.on_new_window(w),
            EventQueueItem::WindowDestroy(window_handle) => self.on_destroy_window(window_handle),
            EventQueueItem::ScreenCreate(s) => self.on_new_screen(s),
        }
    }
}
