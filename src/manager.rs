use super::utils::window::Window;
use super::utils::screen::Screen;
use super::utils::workspace::Workspace;
use super::display_servers::DisplayServer;
use super::event_queue::EventQueueItem;

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
        {
            let all_windows = &mut self.windows;
            let all: Vec<&mut Window> = all_windows.into_iter().map(|w| w ).collect();
            for w in all { w.visable = w.tags.len() == 0; } // if not tagged force it to display
            for ws in &mut self.workspaces {
                let windows: Vec<&mut Window> = all_windows.into_iter().map(|w| w ).collect();
                ws.update_windows( windows );
            }
        }
        let to_update :Vec<&Window>= (&self.windows).into_iter().map(|w| w ).collect();
        self.ds.update_windows(to_update);
    }

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


    pub fn on_event(&mut self, event: EventQueueItem){
        match event {
            EventQueueItem::WindowCreate(w) => { self.on_new_window(w) }
            EventQueueItem::ScreenCreate(s) => { self.on_new_screen(s) }
            _ => {}
        }
    }


}








