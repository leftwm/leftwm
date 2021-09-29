use super::{CommandBuilder, Config, DisplayEvent, Manager, Mode};
use crate::display_servers::DisplayServer;
use crate::utils;
use crate::{display_action::DisplayAction, models::FocusBehaviour};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn display_event_handler(&mut self, event: DisplayEvent) -> bool {
        let update_needed = match event {
            DisplayEvent::ScreenCreate(s) => self.screen_create_handler(s),
            DisplayEvent::WindowCreate(w, x, y) => self.window_created_handler(w, x, y),
            DisplayEvent::WindowChange(w) => self.window_changed_handler(w),

            //The window has been focused, do we want to do anything about it?
            DisplayEvent::MouseEnteredWindow(handle) => match self.state.focus_manager.behaviour {
                FocusBehaviour::Sloppy => return self.state.focus_window(&handle),
                _ => return false,
            },

            DisplayEvent::KeyGrabReload => {
                self.state
                    .actions
                    .push_back(DisplayAction::ReloadKeyGrabs(self.config.mapped_bindings()));
                false
            }

            DisplayEvent::MoveFocusTo(x, y) => self.state.move_focus_to_point(x, y),

            //This is a request to validate focus. Double check that we are focused the correct
            //thing under this point.
            DisplayEvent::VerifyFocusedAt(x, y) => match self.state.focus_manager.behaviour {
                FocusBehaviour::Sloppy => return self.state.validate_focus_at(x, y),
                _ => return false,
            },

            DisplayEvent::WindowDestroy(handle) => self.window_destroyed_handler(&handle),

            DisplayEvent::KeyCombo(mod_mask, xkeysym) => {
                //look through the config and build a command if its defined in the config
                let build = CommandBuilder::<C>::new(&self.config);
                let command = build.xkeyevent(mod_mask, xkeysym);
                command.map_or(false, |cmd| self.command_handler(cmd))
            }

            DisplayEvent::SendCommand(command) => self.command_handler(&command),

            DisplayEvent::MouseCombo(mod_mask, button, handle) => {
                // TODO looks like this should be entirely in state
                let mouse_key = utils::xkeysym_lookup::into_mod(&self.state.mousekey);
                self.state
                    .mouse_combo_handler(mod_mask, button, handle, mouse_key)
            }

            DisplayEvent::ChangeToNormalMode => {
                self.state.mode = Mode::Normal;
                //look through the config and build a command if its defined in the config
                let act = DisplayAction::NormalMode;
                self.state.actions.push_back(act);
                true
            }

            DisplayEvent::Movement(handle, x, y) => {
                if self.state.screens.iter().any(|s| s.root == handle)
                    && self.state.focus_manager.behaviour == FocusBehaviour::Sloppy
                {
                    return self.state.focus_workspace_under_cursor(x, y);
                }
                false
            }

            DisplayEvent::MoveWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - self.state.frame_rate_limitor) > (1000 / 60) {
                    refresh = self.window_move_handler(&handle, x, y);
                    self.state.frame_rate_limitor = time;
                }
                refresh
            }
            DisplayEvent::ResizeWindow(handle, time, x, y) => {
                //limit the frame rate to 60f/sec. otherwise you get lag
                let mut refresh = false;
                if (time - self.state.frame_rate_limitor) > (1000 / 60) {
                    refresh = self.window_resize_handler(&handle, x, y);
                    self.state.frame_rate_limitor = time;
                }
                refresh
            }
        };

        if update_needed {
            self.update_windows();
        }

        update_needed
    }
}
