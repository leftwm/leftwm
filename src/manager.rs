use super::utils::window::Window;
use super::utils::screen::Screen;
use super::utils::workspace::Workspace;
use super::display_servers::DisplayServer;

pub trait DisplayEventHandler {
    fn on_new_window(&mut self, window: Window);
    fn on_new_screen(&mut self, screen: Screen);
}


#[derive(Clone)]
pub struct Manager<DM: DisplayServer>{
    pub windows: Vec<Window>,
    pub screens: Vec<Screen>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>,
    pub ds: DM,
    active_wp_index: usize,
}


impl<DM: DisplayServer> Manager<DM>{


    pub fn new() -> Manager<DM>{
        Manager{
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
            return Some( &self.workspaces[ self.active_wp_index ] );
        }
        None
    }


    pub fn update_windows(&mut self){
        let all_windows = &mut self.windows;
        let all: Vec<&mut Window> = all_windows.into_iter().map(|w| w ).collect();
        for w in all { w.visable = w.tags.len() == 0; } // if not tagged force it to display
        for ws in &mut self.workspaces {
            let windows: Vec<&mut Window> = all_windows.into_iter().map(|w| w ).collect();
            ws.update_windows( windows );
        }
    }


}




impl<DM:DisplayServer> DisplayEventHandler for Manager<DM> {

    fn on_new_window(&mut self, a_window: Window){
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

    fn on_new_screen(&mut self, screen: Screen){
        self.workspaces.push( Workspace::from_screen(&screen) );
        self.screens.push(screen);
    }

}





#[test]
fn adding_a_window_should_tag_it(){
    use super::display_servers::MockDisplayServer;
    use std::sync::{Arc, Mutex};
    let mut subject: Manager<MockDisplayServer> = Manager::new();
    subject.workspaces = vec![ Workspace::new() ];
    subject.workspaces[0].show_tag( "test".to_owned() );
    let ds:MockDisplayServer = DisplayServer::new();
    let mutex = Arc::new(Mutex::new(subject));
    ds.watch_events(mutex.clone()); //NOTE: this add a mock window
    let ss = mutex.lock().unwrap();
    let ww = ss.windows[0].clone();
    assert!( ww.has_tag( "test".to_string() ), "adding a window didn't auto tag it");
}


#[test]
fn after_loading_config_it_should_know_about_all_screens(){
    use super::display_servers::MockDisplayServer;
    use std::sync::{Arc, Mutex};
    let subject: Manager<MockDisplayServer> = Manager::new();
    let ds:MockDisplayServer = DisplayServer::new();
    let mutex = Arc::new(Mutex::new(subject));
    ds.watch_events(mutex.clone()); //NOTE: this add a mock window
    let ss = mutex.lock().unwrap();
    assert!( ss.screens.len() == 1, "Was unable to build the screen");
}

#[test]
fn should_default_to_one_workspace_per_screen(){
    use super::display_servers::MockDisplayServer;
    use std::sync::{Arc, Mutex};
    let subject: Manager<MockDisplayServer> = Manager::new();
    let ds:MockDisplayServer = DisplayServer::new();
    let mutex = Arc::new(Mutex::new(subject));
    ds.watch_events(mutex.clone()); //NOTE: this add a mock window
    let ss = mutex.lock().unwrap();
    assert!( ss.screens.len() == ss.workspaces.len(), "default workspaces did not load");
}

#[test]
fn on_new_window_should_add_items_window_to_the_managed_list(){
    use super::display_servers::MockDisplayServer;
    let mut subject: Manager<MockDisplayServer> = Manager::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(w);
    assert!( subject.windows.len() == 1, "Window was not added to managed list");
}

#[test]
fn two_windows_with_the_same_handle_should_not_be_added(){
    use super::display_servers::MockDisplayServer;
    let mut subject: Manager<MockDisplayServer> = Manager::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    let w2: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(w);
    subject.on_new_window(w2);
    assert!( subject.windows.len() == 1, "Window was not added to managed list");
}
