mod keybind;
mod scratchpad;
mod theme_setting;
mod workspace_config;

use crate::layouts::Layout;
pub use crate::models::FocusBehaviour;
use crate::Manager;
pub use keybind::Keybind;
pub use scratchpad::ScratchPad;
use std::sync::Arc;
pub use theme_setting::{ThemeLoader, ThemeSetting};
pub use workspace_config::Workspace;

pub trait Config<CMD> {
    /// Returns a collection of bindings with the mod key mapped.
    fn mapped_bindings(&self) -> Vec<Keybind<CMD>>;

    fn create_list_of_tags(&self) -> Vec<String>;

    fn workspaces(&self) -> Option<&[Workspace]>;

    fn focus_behaviour(&self) -> FocusBehaviour;

    fn mousekey(&self) -> &str;

    //of you are on tag "1" and you goto tag "1" this takes you to the previous tag
    fn disable_current_tag_swap(&self) -> bool;

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad>;

    fn layouts(&self) -> Vec<Layout>;

    fn focus_new_windows(&self) -> bool;

    fn command_handler(&self, command: &CMD, manager: &mut Manager<CMD>) -> Option<bool>;
}

impl<C, CMD> Config<CMD> for Arc<C>
where
    C: Config<CMD>,
{
    fn mapped_bindings(&self) -> Vec<Keybind<CMD>> {
        C::mapped_bindings(self)
    }

    fn create_list_of_tags(&self) -> Vec<String> {
        C::create_list_of_tags(self)
    }

    fn workspaces(&self) -> Option<&[Workspace]> {
        C::workspaces(self)
    }

    fn focus_behaviour(&self) -> FocusBehaviour {
        C::focus_behaviour(self)
    }

    fn mousekey(&self) -> &str {
        C::mousekey(self)
    }

    fn disable_current_tag_swap(&self) -> bool {
        C::disable_current_tag_swap(self)
    }

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        C::create_list_of_scratchpads(self)
    }

    fn layouts(&self) -> Vec<Layout> {
        C::layouts(self)
    }

    fn focus_new_windows(&self) -> bool {
        C::focus_new_windows(self)
    }

    fn command_handler(&self, command: &CMD, manager: &mut Manager<CMD>) -> Option<bool> {
        C::command_handler(self, command, manager)
    }
}

#[cfg(test)]
pub struct TestConfig {
    pub tags: Vec<String>,
}

#[cfg(test)]
impl<CMD> Config<CMD> for TestConfig {
    fn mapped_bindings(&self) -> Vec<Keybind<CMD>> {
        unimplemented!()
    }
    fn create_list_of_tags(&self) -> Vec<String> {
        self.tags.clone()
    }
    fn workspaces(&self) -> Option<&[Workspace]> {
        unimplemented!()
    }
    fn focus_behaviour(&self) -> FocusBehaviour {
        FocusBehaviour::Sloppy
    }
    fn mousekey(&self) -> &str {
        unimplemented!()
    }
    fn disable_current_tag_swap(&self) -> bool {
        false
    }
    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        vec![]
    }
    fn layouts(&self) -> Vec<Layout> {
        vec![]
    }
    fn focus_new_windows(&self) -> bool {
        false
    }
}
