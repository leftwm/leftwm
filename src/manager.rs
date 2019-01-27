use super::utils::window::Window;
use super::utils::screen::Screen;
use super::utils::workspace::Workspace;

pub struct Manager{
    pub windows: Vec<Window>,
    pub screens: Vec<Screen>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>,
    pub active_tag: String,
}

impl Manager{

    pub fn new() -> Manager{
        Manager{
            windows: Vec::new(),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            active_tag: "".to_string(),
        }
    }

    pub fn on_new_window(&mut self, a_window: Window){
        for w in &self.windows {
            if w.handle == a_window.handle {
                return;
            }
        }
        let mut window = a_window.clone();
        window.tag( self.active_tag.clone() );
        self.windows.push(window);
    }

}


#[test]
fn adding_a_window_should_tag_it(){
    let mut subject = Manager::new();
    subject.active_tag = "test".to_string();
    let window: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(window);
    let ww = subject.windows[0].clone();
    assert!( ww.has_tag( "test".to_string() ), "adding a window didn't auto tag it");
}

#[test]
fn on_new_window_should_add_items_window_to_the_managed_list(){
    let mut subject = Manager::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(w);
    assert!( subject.windows.len() == 1, "Window was not added to managed list");
}

#[test]
fn two_windows_with_the_same_handle_should_not_be_added(){
    let mut subject = Manager::new();
    let w: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(w);
    let w2: Window = unsafe{ std::mem::zeroed() };
    subject.on_new_window(w2);
    assert!( subject.windows.len() == 1, "multiple windows with the same handle in the managed list");
}

