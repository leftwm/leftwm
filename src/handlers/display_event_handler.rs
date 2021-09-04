use super::{CommandBuilder, Config, DisplayEvent, Manager, Mode};
use crate::state::State;
use crate::utils;
use crate::{display_action::DisplayAction, models::FocusBehaviour};

/// Configuration container for processing `DisplayEvents`.
pub struct DisplayEventHandler<C> {
    pub config: C,
}

impl<C: Config> DisplayEventHandler<C> {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn process<CMD>(
        &self,
        manager: &mut Manager<CMD>,
        state: &impl State,
        event: DisplayEvent,
    ) -> bool {
        let update_needed = match event {
            DisplayEvent::ScreenCreate(s) => manager.screen_create_handler(s),
            DisplayEvent::WindowCreate(w, x, y) => manager.window_created_handler(w, x, y),
            DisplayEvent::WindowChange(w) => manager.window_changed_handler(w),

            //The window has been focused, do we want to do anything about it?
            DisplayEvent::MouseEnteredWindow(handle) => match manager.focus_manager.behaviour {
                FocusBehaviour::Sloppy => return manager.focus_window(&handle),
                _ => return false,
            },

            DisplayEvent::MoveFocusTo(x, y) => manager.move_focus_to_point(x, y),

            //This is a request to validate focus. Double check that we are focused the correct
            //thing under this point.
            DisplayEvent::VerifyFocusedAt(x, y) => match manager.focus_manager.behaviour {
                FocusBehaviour::Sloppy => return manager.validate_focus_at(x, y),
                _ => return false,
            },

            DisplayEvent::WindowDestroy(handle) => manager.window_destroyed_handler(&handle),

            DisplayEvent::KeyCombo(mod_mask, xkeysym) => {
                //look through the config and build a command if its defined in the config
                let build = CommandBuilder::new(&self.config);
                let command = build.xkeyevent(mod_mask, xkeysym);
                if let Some((cmd, val)) = command {
                    manager.command_handler(state, &self.config, &cmd, &val)
                } else {
                    false
                }
            }

            DisplayEvent::SendCommand(command, value) => {
                manager.command_handler(state, &self.config, &command, &value)
            }

            DisplayEvent::MouseCombo(mod_mask, button, handle) => {
                let mouse_key = utils::xkeysym_lookup::into_mod(self.config.mousekey());
                manager.mouse_combo_handler(mod_mask, button, handle, mouse_key)
            }

            DisplayEvent::ChangeToNormalMode => {
                manager.mode = Mode::Normal;
                //look through the config and build a command if its defined in the config
                let act = DisplayAction::NormalMode;
                manager.actions.push_back(act);
                true
            }

            DisplayEvent::Movement(handle, x, y) => {
                if manager.screens.iter().any(|s| s.root == handle)
                    && manager.focus_manager.behaviour == FocusBehaviour::Sloppy
                {
                    return manager.focus_workspace_under_cursor(x, y);
                }
                false
            }

            DisplayEvent::MoveWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - manager.frame_rate_limitor) > (1000 / 60) {
                    refresh = manager.window_move_handler(&handle, x, y);
                    manager.frame_rate_limitor = time;
                }
                refresh
            }
            DisplayEvent::ResizeWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - manager.frame_rate_limitor) > (1000 / 60) {
                    refresh = manager.window_resize_handler(&handle, x, y);
                    manager.frame_rate_limitor = time;
                }
                refresh
            }
        };

        if update_needed {
            manager.update_windows();
        }

        update_needed
    }
}
