use super::utils::window::Window;
use super::utils::screen::Screen;
use super::utils::workspace::Workspace;

pub struct Manager{
    pub windows: Vec<Window>,
    pub screens: Vec<Screen>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<String>,
    pub focused_workspace: i32,
}

impl Manager{

    pub fn new() -> Manager{
        Manager{
            windows: Vec::new(),
            screens: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            focused_workspace: 0,
        }
    }

    // load all the defaults values and the things from config 
    pub fn load_config(&mut self, screens: Vec<Screen>){
        self.screens = screens;

        // defaults is to build one workspace per screen
        if self.workspaces.len() == 0 {
            for s in self.screens.clone() {
                self.workspaces.push( Workspace::from_screen(s) )
            }
        }

        // default to tags 1 to 9
        for i in 1..10 {
            self.tags.push( i.to_string() );
        }

    }


    pub fn on_new_window(&mut self, window: Window){
        for w in self.windows.clone() {
            if w.handle == window.handle {
                return;
            }
        }
        self.windows.push(window);
    }


}




#[test]
fn default_config_should_create_tags_1_to_9(){
    let mut subject = Manager::new();
    let screens: Vec<Screen> = Vec::new();
    subject.load_config(screens);
    let tags = subject.tags.clone();
    assert!( tags[0] == "1", "default tag {1} did not load");
    assert!( tags[1] == "2", "default tag {2} did not load");
    assert!( tags[2] == "3", "default tag {3} did not load");
    assert!( tags[3] == "4", "default tag {4} did not load");
    assert!( tags[4] == "5", "default tag {5} did not load");
    assert!( tags[5] == "6", "default tag {6} did not load");
    assert!( tags[6] == "7", "default tag {6} did not load");
    assert!( tags[7] == "8", "default tag {7} did not load");
    assert!( tags[8] == "9", "default tag {8} did not load");
}

#[test]
fn default_config_should_be_one_workspace_per_screen(){
    let mut subject = Manager::new();
    let mut screens = Vec::new();
    let x: Screen = unsafe{ std::mem::zeroed() };
    screens.push(x);
    subject.load_config(screens);
    assert!( subject.screens.len() == subject.workspaces.len(), "default workspaces did not load");
}

#[test]
fn after_loading_config_it_should_know_about_all_screens(){
    let mut subject = Manager::new();
    let mut screens = Vec::new();
    let x: Screen = unsafe{ std::mem::zeroed() };
    screens.push(x);
    subject.load_config(screens);
    assert!( subject.screens.len() == 1, "Was unable to manage the screen");
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

