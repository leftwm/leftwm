mod insert_behavior;
mod workspace_config;

use crate::display_servers::DisplayServer;
use crate::layouts::LayoutMode;
pub use crate::models::ScratchPad;
pub use crate::models::{FocusBehaviour, Gutter, Margins, Size};
use crate::models::{Manager, Window, WindowType, Handle};
use crate::state::State;
pub use insert_behavior::InsertBehavior;
use leftwm_layouts::Layout;
pub use workspace_config::Workspace;

pub trait Config {
    fn create_list_of_tag_labels(&self) -> Vec<String>;

    fn workspaces(&self) -> Option<Vec<Workspace>>;

    fn focus_behaviour(&self) -> FocusBehaviour;

    fn mousekey(&self) -> Vec<String>;

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad>;

    fn layouts(&self) -> Vec<String>;

    fn layout_definitions(&self) -> Vec<Layout>;

    fn layout_mode(&self) -> LayoutMode;

    fn insert_behavior(&self) -> InsertBehavior;

    fn single_window_border(&self) -> bool;

    fn focus_new_windows(&self) -> bool;

    fn command_handler<H: Handle, SERVER>(command: &str, manager: &mut Manager<H, Self, SERVER>) -> bool
    where
        SERVER: DisplayServer<H>,
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
    fn background_color(&self) -> String;
    fn on_new_window_cmd(&self) -> Option<String>;
    fn get_list_of_gutters(&self) -> Vec<Gutter>;
    fn auto_derive_workspaces(&self) -> bool;
    fn disable_tile_drag(&self) -> bool;
    fn disable_window_snap(&self) -> bool;
    fn sloppy_mouse_follows_focus(&self) -> bool;
    fn create_follows_cursor(&self) -> bool;
    fn reposition_cursor_on_resize(&self) -> bool;

    /// Attempt to write current state to a file.
    ///
    /// It will be used to restore the state after soft reload.
    ///
    /// **Note:** this function cannot fail.
    fn save_state<H: Handle>(&self, state: &State<H>);

    /// Load saved state if it exists.
    fn load_state<H: Handle>(&self, state: &mut State<H>);

    /// Handle window placement based on `WM_CLASS`
    fn setup_predefined_window<H: Handle>(&self, state: &mut State<H>, window: &mut Window<H>) -> bool;

    fn load_window<H: Handle>(&self, window: &mut Window<H>) {
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
pub(crate) mod tests {
    use super::*;
    use crate::models::MockHandle;
    use crate::models::Screen;
    use crate::models::Window;
    use crate::models::WindowHandle;

    #[allow(clippy::module_name_repetitions)]
    #[derive(Default)]
    pub struct TestConfig {
        pub tags: Vec<String>,
        pub layouts: Vec<String>,
        pub layout_definitions: Vec<Layout>,
        pub workspaces: Option<Vec<Workspace>>,
        pub insert_behavior: InsertBehavior,
        pub border_width: i32,
        pub single_window_border: bool,
    }

    impl Config for TestConfig {
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
        fn layouts(&self) -> Vec<String> {
            self.layouts.clone()
        }
        fn layout_definitions(&self) -> Vec<Layout> {
            self.layout_definitions.clone()
        }
        fn layout_mode(&self) -> LayoutMode {
            LayoutMode::Workspace
        }

        fn insert_behavior(&self) -> InsertBehavior {
            self.insert_behavior
        }

        fn single_window_border(&self) -> bool {
            self.single_window_border
        }

        fn focus_new_windows(&self) -> bool {
            false
        }
        fn command_handler<H: Handle, SERVER>(command: &str, manager: &mut Manager<H, Self, SERVER>) -> bool
        where
            SERVER: DisplayServer<H>,
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
            self.border_width
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
        fn background_color(&self) -> String {
            unimplemented!()
        }
        fn on_new_window_cmd(&self) -> Option<String> {
            None
        }
        fn get_list_of_gutters(&self) -> Vec<Gutter> {
            Default::default()
        }
        fn disable_tile_drag(&self) -> bool {
            false
        }
        fn disable_window_snap(&self) -> bool {
            false
        }
        fn save_state<H: Handle>(&self, _state: &State<H>) {
            unimplemented!()
        }
        fn load_state<H: Handle>(&self, _state: &mut State<H>) {
            unimplemented!()
        }
        fn setup_predefined_window<H: Handle>(&self, _: &mut State<H>, window: &mut Window<H>) -> bool {
            if window.res_class == Some("ShouldGoToTag2".to_string()) {
                window.tag = Some(2);
                true
            } else {
                false
            }
        }
        fn sloppy_mouse_follows_focus(&self) -> bool {
            true
        }

        fn auto_derive_workspaces(&self) -> bool {
            true
        }

        fn reposition_cursor_on_resize(&self) -> bool {
            true
        }

        fn create_follows_cursor(&self) -> bool {
            false
        }
    }

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
        let mut subject = Window::new(WindowHandle::<MockHandle>(1), None, None);
        subject.res_class = Some("ShouldGoToTag2".to_string());
        manager.window_created_handler(subject, 0, 0);
        assert!(manager.state.windows.iter().all(|w| w.has_tag(&2)));
    }
}
