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
    CloseAllOtherWindows,
    SwapTags,
    SoftReload,
    HardReload,
    AttachScratchPad,
    ReleaseScratchPad,
    NextScratchPadWindow,
    PrevScratchPadWindow,
    ToggleScratchPad,
    ToggleFullScreen,
    ToggleSticky,
    GotoTag,
    ReturnToLastTag,
    FloatingToTile,
    TileToFloating,
    ToggleFloating,
    MoveWindowUp,
    MoveWindowDown,
    MoveWindowTop,
    FocusNextTag,
    FocusPreviousTag,
    FocusWindow,
    FocusWindowUp,
    FocusWindowDown,
    FocusWindowTop,
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    MoveToTag,
    MoveWindowToNextTag,
    MoveWindowToPreviousTag,
    MoveToLastWorkspace,
    MoveWindowToNextWorkspace,
    MoveWindowToPreviousWorkspace,
    NextLayout,
    PreviousLayout,
    SetLayout,
    RotateTag,
    IncreaseMainWidth, //deprecated
    DecreaseMainWidth, //deprecated
    IncreaseMainSize,
    DecreaseMainSize,
    IncreaseMainCount,
    DecreaseMainCount,
    SetMarginMultiplier,
    // Custom commands
    UnloadTheme,
    LoadTheme,
}

impl std::convert::From<BaseCommand> for String {
    fn from(command: BaseCommand) -> Self {
        match command {
            // Special cases that have different names.
            BaseCommand::SwapTags => "SwapScreens".to_owned(),
            BaseCommand::GotoTag => "GoToTag".to_owned(),
            BaseCommand::MoveToTag => "SendWindowToTag".to_owned(),
            BaseCommand::MoveToLastWorkspace => "MoveWindowToLastWorkspace".to_owned(),
            BaseCommand::Execute => String::new(),
            _ => format!("{command:?}"),
        }
    }
}
