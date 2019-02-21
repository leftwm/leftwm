use super::DisplayEvent;
use super::Manager;

pub struct ScreenCreateHandler {}

impl ScreenCreateHandler {
    pub fn new() -> ScreenCreateHandler {
        ScreenCreateHandler {}
    }

    /*
     * process a collection of events, and apply them changes to a manager
     * returns true if changes need to be rendered
     */
    pub fn process(&self, manager: &mut Manager, screen: Screen) -> bool {
        //let tag_index = manager.workspaces.len();
        //let mut workspace = Workspace::from_screen(&screen);
        //workspace.name = tag_index.to_string();
        //let next_tag = self.tags[tag_index].clone();
        //workspace.show_tag(next_tag);
        //self.workspaces.push(workspace);
        //self.screens.push(screen);
        false
    }
}
