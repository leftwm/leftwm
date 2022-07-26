use crate::{
    layouts::Layout,
    models::{TagId, WindowHandle},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Command {
    Execute(String),
    CloseWindow,
    SwapScreens,
    SoftReload,
    HardReload,
    AttachScratchPad {
        window: Option<WindowHandle>,
        scratchpad: String,
    },
    ReleaseScratchPad {
        window: ReleaseScratchPadOption,
        tag: Option<TagId>,
    },
    PrevScratchPadWindow {
        scratchpad: String,
    },
    NextScratchPadWindow {
        scratchpad: String,
    },
    ToggleScratchPad(String),
    ToggleFullScreen,
    ToggleSticky,
    GoToTag {
        tag: TagId,
        swap: bool,
    },
    ReturnToLastTag,
    FloatingToTile,
    TileToFloating,
    ToggleFloating,
    MoveWindowUp,
    MoveWindowDown,
    MoveWindowTop {
        swap: bool,
    },
    FocusNextTag,
    FocusPreviousTag,
    FocusWindow(String),
    FocusWindowUp,
    FocusWindowDown,
    FocusWindowTop {
        swap: bool,
    },
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    SendWindowToTag {
        window: Option<WindowHandle>,
        tag: TagId,
    },
    MoveWindowToNextTag {
        follow: bool,
    },
    MoveWindowToPreviousTag {
        follow: bool,
    },
    MoveWindowToLastWorkspace,
    MoveWindowToNextWorkspace,
    MoveWindowToPreviousWorkspace,
    MouseMoveWindow,
    NextLayout,
    PreviousLayout,
    SetLayout(Layout),
    RotateTag,
    IncreaseMainWidth(i8),
    DecreaseMainWidth(i8),
    SetMarginMultiplier(f32),
    SendWorkspaceToTag(usize, usize),
    CloseAllOtherWindows,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ReleaseScratchPadOption {
    Handle(WindowHandle),
    ScrathpadName(String),
    None,
}
