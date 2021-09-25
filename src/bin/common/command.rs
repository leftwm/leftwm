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
        _value: Option<&str>,
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

// TODO this code is temporary. Due to the limitations of TOML we cannot serialize leftwm::Command
//      easily. If we replace TOML by JSON/JSON5/YAML we will be able to remove this code and a
//      bunch of validation in leftwm-check.rs. This requires to deprecate the TOML config file,
//      thus making a breaking change.
//
//      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=59232ae3a6f902fc3a3a7a09d1d48c80
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BaseCommand {
    Execute,
    CloseWindow,
    SwapTags,
    SoftReload,
    HardReload,
    ToggleScratchPad,
    ToggleFullScreen,
    GotoTag,
    FloatingToTile,
    MoveWindowUp,
    MoveWindowDown,
    MoveWindowTop,
    FocusNextTag,
    FocusPreviousTag,
    FocusWindowUp,
    FocusWindowDown,
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    MoveToTag,
    MoveToLastWorkspace,
    MouseMoveWindow,
    NextLayout,
    PreviousLayout,
    SetLayout,
    RotateTag,
    IncreaseMainWidth,
    DecreaseMainWidth,
    SetMarginMultiplier,
    // Custom commands
    UnloadTheme,
    LoadTheme,
}
