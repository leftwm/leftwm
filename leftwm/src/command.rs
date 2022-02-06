use serde::{Deserialize, Serialize};

/*  TODO this code is temporary. Due to the limitations of TOML we cannot serialize leftwm_core::Command
*      easily. If we replace TOML by JSON/JSON5/YAML we will be able to remove this code and a
*      bunch of validation in leftwm-check.rs. This requires to deprecate the TOML config file,
*      thus making a breaking change.
*
*      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=59232ae3a6f902fc3a3a7a09d1d48c80
*/

// Because this is temporary, we will allow this clippy lint to be bypassed
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BaseCommand {
    Execute,
    CloseWindow,
    SwapTags,
    SoftReload,
    HardReload,
    ToggleScratchPad,
    ToggleFullScreen,
    ToggleSticky,
    GotoTag,
    FloatingToTile,
    TileToFloating,
    ToggleFloating,
    MoveWindowUp,
    MoveWindowDown,
    MoveWindowTop,
    FocusNextTag,
    FocusPreviousTag,
    FocusWindowUp,
    FocusWindowDown,
    FocusWindowTop,
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    MoveToTag,
    MoveToLastWorkspace,
    MoveWindowToNextWorkspace,
    MoveWindowToPreviousWorkspace,
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

impl std::convert::From<BaseCommand> for String {
    fn from(command: BaseCommand) -> Self {
        let r#str = match command {
            BaseCommand::CloseWindow => "CloseWindow",
            BaseCommand::SwapTags => "SwapScreens",
            BaseCommand::SoftReload => "SoftReload",
            BaseCommand::ToggleScratchPad => "ToggleScratchPad",
            BaseCommand::ToggleFullScreen => "ToggleFullScreen",
            BaseCommand::ToggleSticky => "ToggleSticky",
            BaseCommand::FloatingToTile => "FloatingToTile",
            BaseCommand::TileToFloating => "TileToFloating",
            BaseCommand::ToggleFloating => "ToggleFloating",
            BaseCommand::MoveWindowUp => "MoveWindowUp",
            BaseCommand::MoveWindowDown => "MoveWindowDown",
            BaseCommand::MoveWindowTop => "MoveWindowTop",
            BaseCommand::FocusNextTag => "FocusNextTag",
            BaseCommand::FocusPreviousTag => "FocusPreviousTag",
            BaseCommand::FocusWindowUp => "FocusWindowUp",
            BaseCommand::FocusWindowDown => "FocusWindowDown",
            BaseCommand::FocusWindowTop => "FocusWindowTop",
            BaseCommand::FocusWorkspaceNext => "FocusWorkspaceNext",
            BaseCommand::FocusWorkspacePrevious => "FocusWorkspacePrevious",
            BaseCommand::MoveToTag => "SendWindowToTag",
            BaseCommand::MoveToLastWorkspace => "MoveWindowToLastWorkspace",
            BaseCommand::MoveWindowToNextWorkspace => "MoveWindowToNextWorkspace",
            BaseCommand::MoveWindowToPreviousWorkspace => "MoveWindowToPreviousWorkspace",
            BaseCommand::NextLayout => "NextLayout",
            BaseCommand::PreviousLayout => "PreviousLayout",
            BaseCommand::SetLayout => "SetLayout",
            BaseCommand::RotateTag => "RotateTag",
            BaseCommand::IncreaseMainWidth => "IncreaseMainWidth",
            BaseCommand::DecreaseMainWidth => "DecreaseMainWidth",
            BaseCommand::SetMarginMultiplier => "SetMarginMultiplier",
            // Custom commands
            BaseCommand::UnloadTheme => "UnloadTheme",
            BaseCommand::LoadTheme => "LoadTheme",
            _ => "",
        };
        r#str.to_owned()
    }
}
