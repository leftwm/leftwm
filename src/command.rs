use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Derivative)]
#[derivative(Debug)]
pub enum Command<CMD> {
    Execute,
    CloseWindow,
    SwapTags,
    SoftReload,
    HardReload,
    ToggleScratchPad,
    ToggleFullScreen,
    GotoTag(usize),
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
    MoveToTag(usize),
    MoveToLastWorkspace,
    MouseMoveWindow,
    NextLayout,
    PreviousLayout,
    SetLayout,
    RotateTag,
    IncreaseMainWidth,
    DecreaseMainWidth,
    SetMarginMultiplier,
    Other(#[derivative(Debug = "ignore")] CMD),
}
