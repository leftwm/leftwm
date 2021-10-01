mod keybind;
mod scratchpad;
mod workspace_config;

use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
pub use crate::models::{FocusBehaviour, Gutter, Margins, Size};
use crate::Manager;
pub use keybind::Keybind;
pub use scratchpad::ScratchPad;
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

    fn command_handler<SERVER>(command: &str, manager: &mut Manager<Self, SERVER>) -> bool
    where
        Self: Sized,
        SERVER: DisplayServer;

    fn border_width(&self) -> i32;
    fn margin(&self) -> Margins;
    fn workspace_margin(&self) -> Option<Margins>;
    fn gutter(&self) -> Option<Vec<Gutter>>;
    fn default_border_color(&self) -> &str;
    fn floating_border_color(&self) -> &str;
    fn focused_border_color(&self) -> &str;
    fn on_new_window_cmd(&self) -> Option<String>;
    fn get_list_of_gutters(&self) -> Vec<Gutter>;
    fn max_window_width(&self) -> Option<Size>;

    /// Write current state to a file.
    /// It will be used to restore the state after soft reload.
    ///
    /// # Errors
    ///
    /// Will return error if unable to create `state_file` or
    /// if unable to serialize the text.
    /// May be caused by inadequate permissions, not enough
    /// space on drive, or other typical filesystem issues.
    fn save_state<SERVER>(manager: &Manager<Self, SERVER>)
    where
        Self: Sized,
        SERVER: DisplayServer;

    /// Load saved state if it exists.
    fn load_state<SERVER>(manager: &mut Manager<Self, SERVER>)
    where
        Self: Sized,
        SERVER: DisplayServer;
}

#[cfg(test)]
#[allow(clippy::module_name_repetitions)]
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
    fn command_handler<SERVER>(_command: &str, _manager: &mut Manager<Self, SERVER>) -> bool
    where
        Self: Sized,
        SERVER: DisplayServer,
    {
        unimplemented!()
    }
    fn border_width(&self) -> i32 {
        0
    }
    fn margin(&self) -> Margins {
        Margins::new(0)
    }
    fn workspace_margin(&self) -> Option<Margins> {
        None
    }
    fn gutter(&self) -> Option<Vec<Gutter>> {
        unimplemented!()
    }
    fn default_border_color(&self) -> &str {
        unimplemented!()
    }
    fn floating_border_color(&self) -> &str {
        unimplemented!()
    }
    fn focused_border_color(&self) -> &str {
        unimplemented!()
    }
    fn on_new_window_cmd(&self) -> Option<String> {
        None
    }
    fn get_list_of_gutters(&self) -> Vec<Gutter> {
        Default::default()
    }
    fn max_window_width(&self) -> Option<Size> {
        None
    }
    fn save_state<SERVER>(_manager: &Manager<Self, SERVER>)
    where
        Self: Sized,
        SERVER: DisplayServer,
    {
        unimplemented!()
    }
    /// Load saved state if it exists.
    fn load_state<SERVER>(_manager: &mut Manager<Self, SERVER>)
    where
        Self: Sized,
        SERVER: DisplayServer,
    {
        unimplemented!()
    }
}
