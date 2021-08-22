mod keybind;
mod scratchpad;
mod theme_setting;
mod workspace_config;

pub use crate::models::FocusBehaviour;
pub use keybind::Keybind;
pub use scratchpad::ScratchPad;
pub use theme_setting::ThemeSetting;
pub use workspace_config::Workspace;

pub trait Config {
    /// Returns a collection of bindings with the mod key mapped.
    #[must_use]
    fn mapped_bindings(&self) -> Vec<Keybind>;

    /// # Panics
    ///
    /// Will panic if the default tags cannot be unwrapped. Not likely to occur, as this is defined
    /// behaviour.
    #[must_use]
    fn get_list_of_tags(&self) -> Vec<String>;

    #[must_use]
    fn get_list_of_scratchpads(&self) -> Vec<ScratchPad>;

    // TODO why must_use everywhere?
    #[must_use]
    fn workspaces(&self) -> Option<&[Workspace]>;

    // TODO why must_use everywhere?
    #[must_use]
    fn focus_behaviour(&self) -> FocusBehaviour;

    fn mousekey(&self) -> &str;

    //of you are on tag "1" and you goto tag "1" this takes you to the previous tag
    fn disable_current_tag_swap(&self) -> bool;
}

use std::sync::Arc;
impl<C> Config for Arc<C>
where
    C: Config,
{
    fn mapped_bindings(&self) -> Vec<Keybind> {
        C::mapped_bindings(self)
    }

    fn get_list_of_tags(&self) -> Vec<String> {
        C::get_list_of_tags(self)
    }

    fn get_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        C::get_list_of_scratchpads(self)
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
}
