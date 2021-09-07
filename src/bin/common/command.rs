use super::Config;
use leftwm::Manager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Command {
    UnloadTheme,
    LoadTheme(PathBuf),
}

impl Command {
    pub fn execute(&self, manager: &mut Manager<Config, Self>) -> Option<bool> {
        match self {
            Command::UnloadTheme => {
                manager.config.theme_setting = Default::default();
                Some(manager.update_for_theme())
            }
            Command::LoadTheme(path) => {
                manager.config.theme_setting.load(&path);
                Some(manager.update_for_theme())
            }
        }
    }
}
