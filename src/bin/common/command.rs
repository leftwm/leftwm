use super::Config;
use leftwm::{DisplayServer, Manager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Command {
    UnloadTheme,
    LoadTheme(PathBuf),
}

impl Command {
    pub fn execute<SERVER: DisplayServer<Self>>(
        &self,
        manager: &mut Manager<Config, Self, SERVER>,
    ) -> Option<bool> {
        match self {
            Command::UnloadTheme => {
                manager.state.config.theme_setting = Default::default();
                Some(manager.update_for_theme())
            }
            Command::LoadTheme(path) => {
                manager.state.config.theme_setting.load(&path);
                Some(manager.update_for_theme())
            }
        }
    }
}
