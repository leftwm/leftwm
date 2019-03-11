use super::*;
use crate::display_action::DisplayAction;
use crate::utils::logging::*;

pub struct DisplayEventHandler {
    pub config: Config,
}

impl DisplayEventHandler {
    /*
     * process a collection of events, and apply them changes to a manager
     * returns true if changes need to be rendered
     */
    pub fn process(&self, manager: &mut Manager, event: DisplayEvent) -> bool {
        log_info("DISPLAY_EVENT", &(format!("{:?}", event)));
        let update_needed = match event {
            DisplayEvent::ScreenCreate(s) => screen_create_handler::process(manager, s),
            DisplayEvent::WindowCreate(w) => window_handler::created(manager, w),
            DisplayEvent::WindowChange(w) => window_handler::changed(manager, w),
            DisplayEvent::FocusedWindow(handle, x, y) => {
                focus_handler::focus_window_by_handle(manager, &handle, x, y)
            }
            DisplayEvent::WindowDestroy(handle) => window_handler::destroyed(manager, &handle),

            DisplayEvent::KeyCombo(mod_mask, xkeysym) => {
                //look through the config and build a command if its defined in the config
                let build = CommandBuilder::new(&self.config);
                let command = build.from_xkeyevent(mod_mask, xkeysym);
                if let Some((cmd, val)) = command {
                    command_handler::process(manager, cmd, val)
                } else {
                    false
                }
            }

            DisplayEvent::MouseCombo(mod_mask, button, handle) => {
                mouse_combo_handler::process(manager, mod_mask, button, handle)
            }

            DisplayEvent::ChangeToNormalMode => {
                //look through the config and build a command if its defined in the config
                let act = DisplayAction::NormalMode;
                manager.actions.push_back(act);
                false
            }

            DisplayEvent::Movement(handle, x, y) => {
                if manager.screens.iter().any(|s| s.root == handle) {
                    focus_handler::focus_workspace_under_cursor(manager, x, y)
                } else {
                    false
                }
            }

            DisplayEvent::MoveWindow(handle, x, y) => {
                window_move_handler::process(manager, &handle, x, y)
            }
            DisplayEvent::ResizeWindow(handle, x, y) => {
                window_resize_handler::process(manager, &handle, x, y)
            }
            //_ => false,
        };

        if update_needed {
            self.update_windows(manager);
        }

        //println!("WORKSPACES: {}", manager.workspaces_display());

        //println!("state: {:?}", manager);
        //println!("state: {:?}", manager.windows);

        update_needed
    }

    /*
     * step over all the windows for each workspace and updates all the things
     * based on the new state of the WM
     */
    fn update_windows(&self, manager: &mut Manager) {
        log_info("WINDOWS", &manager.windows_display());
        log_info("TAGS", &manager.tags_display());
        log_info("WORKSPACES", &manager.workspaces_display());
        let state_str = format!("{:?}", manager);
        log_info("FULL_STATE", &state_str);
        let all_windows = &mut manager.windows;
        let all: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
        for w in all {
            w.visable = w.tags.is_empty() || w.floating;
        } // if not tagged force it to display
        for ws in &mut manager.workspaces {
            let windows: Vec<&mut Window> = all_windows.iter_mut().map(|w| w).collect();
            ws.update_windows(windows);
        }
    }
}
