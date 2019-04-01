use super::*;
use crate::config::ThemeSetting;
use crate::utils::command_pipe::ExternalCommand;

pub fn process(manager: &mut Manager, command: ExternalCommand) -> bool {
    match command {
        ExternalCommand::UnloadTheme => {
            let theme = ThemeSetting::default();
            for win in &mut manager.windows {
                win.update_for_theme(&theme);
            }
            manager.theme_setting = theme;
            return true;
        }

        ExternalCommand::LoadTheme(path) => {
            let theme = ThemeSetting::load(&path);
            for win in &mut manager.windows {
                win.update_for_theme(&theme);
            }
            manager.theme_setting = theme;
            return true;
        }

        _ => {}
    }

    false
}
