use super::utils::window::Window;
use super::utils::screen::Screen;

pub struct Manager{
    pub windows :Vec<Window>,
    pub screens :Vec<Screen>
}

impl Manager{

    pub fn new() -> Manager{
        Manager{
            windows: Vec::new(),
            screens: Vec::new()
        }
    }


    pub fn add_screen(&mut self, screen: Screen){
        self.screens.push(screen);
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
fn it_should_be_able_to_manage_a_screen(){
    let mut subject = Manager::new();
    let x: Screen = unsafe{ std::mem::zeroed() };
    subject.add_screen(x);
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
    assert!( subject.windows.len() == 1, "multiable windows with the same handle in the managed list");
}

