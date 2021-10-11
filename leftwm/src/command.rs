use serde::{Deserialize, Serialize};

// TODO this code is temporary. Due to the limitations of TOML we cannot serialize leftwm_core::Command
//      easily. If we replace TOML by JSON/JSON5/YAML we will be able to remove this code and a
//      bunch of validation in leftwm-check.rs. This requires to deprecate the TOML config file,
//      thus making a breaking change.
//
//      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=59232ae3a6f902fc3a3a7a09d1d48c80
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    MoveWindowToNextWorkspace,
    MoveWindowToPreviousWorkspace,
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
