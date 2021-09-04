mod keybind;
mod scratchpad;
mod theme_setting;
mod workspace_config;

use crate::layouts::Layout;
pub use crate::models::FocusBehaviour;
pub use keybind::Keybind;
pub use scratchpad::ScratchPad;
pub use theme_setting::{ThemeLoader, ThemeSetting};
pub use workspace_config::Workspace;

pub trait Config {
    /// Returns a collection of bindings with the mod key mapped.
    fn mapped_bindings(&self) -> Vec<Keybind>;

    fn create_list_of_tags(&self) -> Vec<String>;

    fn workspaces(&self) -> Option<&[Workspace]>;

    fn focus_behaviour(&self) -> FocusBehaviour;

    fn mousekey(&self) -> &str;

    //of you are on tag "1" and you goto tag "1" this takes you to the previous tag
    fn disable_current_tag_swap(&self) -> bool;

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad>;

    fn layouts(&self) -> Vec<Layout>;

    fn focus_new_windows(&self) -> bool;
}

use std::sync::Arc;
impl<C> Config for Arc<C>
where
    C: Config,
{
    fn mapped_bindings(&self) -> Vec<Keybind> {
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
}

#[cfg(test)]
pub struct TestConfig {
    pub tags: Vec<String>,
}

#[cfg(test)]
impl Config for TestConfig {
    fn mapped_bindings(&self) -> Vec<Keybind> {
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
