use super::utils::window::Window;
use super::utils::screen::Screen;
use super::utils::workspace::Workspace;
use super::display_servers::DisplayServer;

#[derive(Clone)]
pub struct Manager{
    pub windows: Vec<Window>,
    pub screens: Vec<Screen>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>,
    active_wp_index: usize,
}

impl Manager{


    pub fn new() -> Manager{
        Manager{
            windows: Vec::new(),
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


    pub fn on_new_window<T: DisplayServer>(&mut self, _: &T, a_window: Window){
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





#[test]
fn adding_a_window_should_tag_it(){
    use super::display_servers::MockDisplayServer;
    let mut subject = Manager::new();
    subject.workspaces = vec![ Workspace::new() ];
    subject.workspaces[0].show_tag( "test".to_owned() );
    let ds:MockDisplayServer = DisplayServer::new();
    let window: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(&ds, window);
    let ww = subject.windows[0].clone();
    assert!( ww.has_tag( "test".to_string() ), "adding a window didn't auto tag it");
}

#[test]
fn on_new_window_should_add_items_window_to_the_managed_list(){
    use super::display_servers::MockDisplayServer;
    let mut subject = Manager::new();
    let ds:MockDisplayServer = DisplayServer::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(&ds, w);
    assert!( subject.windows.len() == 1, "Window was not added to managed list");
}

#[test]
fn two_windows_with_the_same_handle_should_not_be_added(){
    use super::display_servers::MockDisplayServer;
    let mut subject = Manager::new();
    let ds:MockDisplayServer = DisplayServer::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(&ds, w);
    let w2: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(&ds, w2);
    assert!( subject.windows.len() == 1, "multiple windows with the same handle in the managed list");
}

