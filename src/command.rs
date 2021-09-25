use crate::layouts::Layout;
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
    ToggleScratchPad(String),
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
    SetLayout(Layout),
    RotateTag,
    IncreaseMainWidth,
    DecreaseMainWidth,
    SetMarginMultiplier(f32),
    SendWorkspaceToTag(usize, usize),
    Other(#[derivative(Debug = "ignore")] CMD),
}
