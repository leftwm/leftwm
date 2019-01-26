use super::utils::window::Window;

pub struct Manager{
    pub windows :Vec<Window>
}

impl Manager{

    pub fn new() -> Manager{
        Manager{
            windows: Vec::new()
        }
    }

    pub fn on_new_window(&mut self, w: Window){
        println!("NEW WINDOW! {:#?}", w);
        self.windows.push(w);
    }

}

