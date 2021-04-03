use super::{
    command_handler, focus_handler, mouse_combo_handler, screen_create_handler, window_handler,
    window_move_handler, window_resize_handler, CommandBuilder, Config, DisplayEvent, Manager,
    Mode,
};
use crate::display_action::DisplayAction;
use crate::utils::window_updater::update_windows;

/// Configuration container for processing DisplayEvents.
pub struct DisplayEventHandler {
    pub config: Config,
}

impl DisplayEventHandler {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn process(&self, manager: &mut Manager, event: DisplayEvent) -> bool {
        let update_needed = match event {
            DisplayEvent::ScreenCreate(s) => screen_create_handler::process(manager, s),
            DisplayEvent::WindowCreate(w, x, y) => window_handler::created(manager, w, x, y),
            DisplayEvent::WindowChange(w) => window_handler::changed(manager, w),

            //The window has been focused, do we want to do anything about it?
            DisplayEvent::MouseEnteredWindow(handle) => {
                return focus_handler::focus_window(manager, &handle)
            }

            DisplayEvent::MoveFocusTo(x, y) => focus_handler::move_focus_to_point(manager, x, y),

            //This is a request to validate focus. Double check that we are focused the correct
            //thing under this point.
            DisplayEvent::VerifyFocusedAt(x, y) => focus_handler::validate_focus_at(manager, x, y),

            DisplayEvent::WindowDestroy(handle) => {
                window_handler::destroyed(manager, &handle);
                true
            }

            DisplayEvent::KeyCombo(mod_mask, xkeysym) => {
                //look through the config and build a command if its defined in the config
                let build = CommandBuilder::new(&self.config);
                let command = build.xkeyevent(mod_mask, xkeysym);
                if let Some((cmd, val)) = command {
                    command_handler::process(manager, &self.config, cmd, val)
                } else {
                    false
                }
            }

            DisplayEvent::SendCommand(command, value) => {
                command_handler::process(manager, &self.config, command, value)
            }

            DisplayEvent::MouseCombo(mod_mask, button, handle) => {
                mouse_combo_handler::process(manager, mod_mask, button, handle)
            }

            DisplayEvent::ChangeToNormalMode => {
                manager.mode = Mode::NormalMode;
                //look through the config and build a command if its defined in the config
                let act = DisplayAction::NormalMode;
                manager.actions.push_back(act);
                true
            }

            DisplayEvent::Movement(handle, x, y) => {
                if manager.screens.iter().any(|s| s.root == handle) {
                    focus_handler::focus_workspace_under_cursor(manager, x, y)
                } else {
                    false
                }
            }

            DisplayEvent::MoveWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - manager.frame_rate_limitor) > (1000 / 60) {
                    refresh = window_move_handler::process(manager, &handle, x, y);
                    manager.frame_rate_limitor = time;
                }
                refresh
            }
            DisplayEvent::ResizeWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - manager.frame_rate_limitor) > (1000 / 60) {
                    refresh = window_resize_handler::process(manager, &handle, x, y);
                    manager.frame_rate_limitor = time;
                }
                refresh
            }
        };

        if update_needed {
            update_windows(manager);
        }

        update_needed
    }
}
