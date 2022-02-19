mod insert_behavior;
mod keybind;
mod scratchpad;
mod workspace_config;

use crate::display_servers::DisplayServer;
use crate::layouts::Layout;
pub use crate::models::{FocusBehaviour, Gutter, Margins, Size};
use crate::models::{LayoutMode, Manager, Window, WindowType};
use crate::state::State;
pub use insert_behavior::InsertBehavior;
pub use keybind::Keybind;
pub use scratchpad::ScratchPad;
pub use workspace_config::Workspace;

pub trait Config {
    /// Returns a collection of bindings with the mod key mapped.
    fn mapped_bindings(&self) -> Vec<Keybind>;

    fn create_list_of_tag_labels(&self) -> Vec<String>;

    fn workspaces(&self) -> Option<Vec<Workspace>>;

    fn focus_behaviour(&self) -> FocusBehaviour;

    fn mousekey(&self) -> Vec<String>;

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad>;

    fn layouts(&self) -> Vec<Layout>;

    fn layout_mode(&self) -> LayoutMode;

    fn insert_behavior(&self) -> InsertBehavior;

    fn focus_new_windows(&self) -> bool;

    fn command_handler<SERVER>(command: &str, manager: &mut Manager<Self, SERVER>) -> bool
    where
        SERVER: DisplayServer,
        Self: Sized;

    fn always_float(&self) -> bool;
    fn default_width(&self) -> i32;
    fn default_height(&self) -> i32;
    fn border_width(&self) -> i32;
    fn margin(&self) -> Margins;
    fn workspace_margin(&self) -> Option<Margins>;
    fn gutter(&self) -> Option<Vec<Gutter>>;
    fn default_border_color(&self) -> String;
    fn floating_border_color(&self) -> String;
    fn focused_border_color(&self) -> String;
    fn on_new_window_cmd(&self) -> Option<String>;
    fn get_list_of_gutters(&self) -> Vec<Gutter>;
    fn max_window_width(&self) -> Option<Size>;
    fn disable_tile_drag(&self) -> bool;

    /// Attempt to write current state to a file.
    ///
    /// It will be used to restore the state after soft reload.
    ///
    /// **Note:** this function cannot fail.
    fn save_state(&self, state: &State);

    /// Load saved state if it exists.
    fn load_state(&self, state: &mut State);

    /// Handle window placement based on `WM_CLASS`
    fn setup_predefined_window(&self, window: &mut Window) -> bool;

    fn load_window(&self, window: &mut Window) {
        if window.r#type == WindowType::Normal {
            window.margin = self.margin();
            window.border = self.border_width();
            window.must_float = self.always_float();
        } else {
            window.margin = Margins::new(0);
            window.border = 0;
        }
    }
}

#[cfg(test)]
#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct TestConfig {
    pub tags: Vec<String>,
    pub layouts: Vec<Layout>,
    pub workspaces: Option<Vec<Workspace>>,
    pub insert_behavior: InsertBehavior,
}

#[cfg(test)]
impl Config for TestConfig {
    fn mapped_bindings(&self) -> Vec<Keybind> {
        unimplemented!()
    }
    fn create_list_of_tag_labels(&self) -> Vec<String> {
        self.tags.clone()
    }
    fn workspaces(&self) -> Option<Vec<Workspace>> {
        self.workspaces.clone()
    }
    fn focus_behaviour(&self) -> FocusBehaviour {
        FocusBehaviour::ClickTo
    }
    fn mousekey(&self) -> Vec<String> {
        vec!["Mod4".to_owned()]
    }
    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        vec![]
    }
    fn layouts(&self) -> Vec<Layout> {
        self.layouts.clone()
    }
    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Workspace
    }

    fn insert_behavior(&self) -> InsertBehavior {
        self.insert_behavior
    }

    fn focus_new_windows(&self) -> bool {
        false
    }
    fn command_handler<SERVER>(command: &str, manager: &mut Manager<Self, SERVER>) -> bool
    where
        SERVER: DisplayServer,
    {
        match command {
            "GoToTag2" => manager.command_handler(&crate::Command::GoToTag {
                tag: 2,
                swap: false,
            }),
            _ => unimplemented!("custom command handler: {:?}", command),
        }
    }
    fn always_float(&self) -> bool {
        false
    }
    fn default_width(&self) -> i32 {
        1000
    }
    fn default_height(&self) -> i32 {
        800
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
    fn default_border_color(&self) -> String {
        unimplemented!()
    }
    fn floating_border_color(&self) -> String {
        unimplemented!()
    }
    fn focused_border_color(&self) -> String {
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
    fn disable_tile_drag(&self) -> bool {
        false
    }
    fn save_state(&self, _state: &State) {
        unimplemented!()
    }
    fn load_state(&self, _state: &mut State) {
        unimplemented!()
    }
    fn setup_predefined_window(&self, window: &mut Window) -> bool {
        if window.res_class == Some("ShouldGoToTag2".to_string()) {
            window.tags = vec![2];
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Screen;
    use crate::models::Window;
    use crate::models::WindowHandle;

    #[test]
    fn ensure_command_handler_trait_boundary() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        assert!(TestConfig::command_handler("GoToTag2", &mut manager));
        assert_eq!(manager.state.focus_manager.tag_history, &[2, 1]);
    }

    #[test]
    fn check_wm_class_is_associated_with_predefined_tag() {
        let mut manager = Manager::new_test(vec!["1".to_string(), "2".to_string()]);
        manager.screen_create_handler(Screen::default());
        let mut subject = Window::new(WindowHandle::MockHandle(1), None, None);
        subject.res_class = Some("ShouldGoToTag2".to_string());
        manager.window_created_handler(subject, 0, 0);
        assert!(manager.state.windows.iter().all(|w| w.has_tag(&2)));
    }
}
