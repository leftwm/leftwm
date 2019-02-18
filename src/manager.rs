use super::config;
use super::config::Config;
use super::display_servers::DisplayServer;
use super::display_servers::MockDisplayServer;
use super::event_queue::EventQueueItem;
use super::utils::command::Command;
use super::utils::screen::Screen;
use super::utils::window::Window;
use super::utils::window::WindowHandle;
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
    config: Config,
}

impl<DM: DisplayServer> Manager<DM> {
    pub fn new() -> Manager<DM> {
        let config = config::parse_config();
        let mut m = Manager {
            windows: Vec::new(),
            ds: DM::new(&config),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            active_wp_index: 0,
            config: config,
        };
        config::apply_config(&mut m);
        m
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
        let tag_index = self.workspaces.len();
        let mut workspace = Workspace::from_screen(&screen);
        let next_tag = self.tags[tag_index].clone();
        workspace.show_tag(next_tag);
        self.workspaces.push(workspace);
        self.screens.push(screen);
    }

    fn on_destroy_window(&mut self, handle: WindowHandle) {
        let index = self.windows.iter().position(|w| w.handle == handle);
        if let Some(i) = index {
            self.windows.remove(i);
        }
        self.update_windows();
    }

    pub fn on_event(&mut self, event: EventQueueItem) {
        match event {
            EventQueueItem::WindowCreate(w) => self.on_new_window(w),
            EventQueueItem::WindowDestroy(window_handle) => self.on_destroy_window(window_handle),
            EventQueueItem::ScreenCreate(s) => self.on_new_screen(s),
            EventQueueItem::Command(command, value) => self.on_command(command, value),
        }
    }

    /*
     * change the active workspace to view a given set of tags
     */
    fn goto_tags(&mut self, tags: Vec<&String>) {
        if let Some(workspace) = self.active_workspace() {
            //workspace.tags(tags)
        }
    }

    /*
     * route a command to its correct handler
     */
    pub fn on_command(&mut self, command: Command, value: Option<String>) {
        match command {
            Command::Execute => {}
            //CloseWindow => {},
            //SwapWorkspaces => {},
            Command::GotoTag => {
                if let Some(val) = &value {
                    self.goto_tags(vec![val]);
                    //println!("goto tag {:#?}", val);
                }
            }
            //MovetoWorkspace => {},
            _ => {}
        }
    }
}

#[allow(dead_code)]
fn mock_manager() -> Manager<MockDisplayServer> {
    let mut manager: Manager<MockDisplayServer> = Manager::new();
    for s in manager.ds.create_fake_screens(1) {
        manager.on_new_screen(s);
    }
    manager
}

#[test]
fn should_have_mock_workspaces() {
    let manager = mock_manager();
    assert!(manager.workspaces.len() == 1)
}

#[test]
fn adding_a_second_window_should_resize_the_first() {
    let mut manager = mock_manager();
    let w1 = Window::new(WindowHandle::MockHandle(1), None);
    let w2 = Window::new(WindowHandle::MockHandle(2), None);
    manager.on_new_window(w1);
    let w = manager.windows[0].width();
    manager.on_new_window(w2);
    assert!(
        manager.windows[0].width() != w,
        "Expected window to resize when other window was added"
    );
}

#[test]
fn removeing_a_window_should_remove_it_from_the_list_of_managed_windows() {
    let mut manager = mock_manager();
    let w1 = Window::new(WindowHandle::MockHandle(1), None);
    let w2 = Window::new(WindowHandle::MockHandle(2), None);
    manager.on_new_window(w1);
    manager.on_new_window(w2);
    manager.on_destroy_window(manager.windows[1].handle.clone());
    assert!(
        manager.windows.len() == 1,
        "Expected only one window to remain"
    );
}

#[test]
fn removeing_a_window_should_resize_the_windows_left_in_the_workspace() {
    let mut manager = mock_manager();
    let w1 = Window::new(WindowHandle::MockHandle(1), None);
    let w2 = Window::new(WindowHandle::MockHandle(2), None);
    manager.on_new_window(w1);
    manager.on_new_window(w2);
    let w = manager.windows[0].width();
    manager.on_destroy_window(manager.windows[1].handle.clone());
    assert!(
        manager.windows[0].width() != w,
        "Expected window to resize when other window was removed"
    );
}
