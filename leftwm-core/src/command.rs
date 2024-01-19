pub use crate::handlers::command_handler::ReleaseScratchPadOption;
use crate::models::{Handle, ScratchPadName, TagId, WindowHandle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Command<H: Handle> {
    CloseWindow,
    SwapScreens,
    SoftReload,
    HardReload,
    AttachScratchPad {
        #[serde(bound = "")]
        window: Option<WindowHandle<H>>,
        scratchpad: ScratchPadName,
    },
    ReleaseScratchPad {
        #[serde(bound = "")]
        window: ReleaseScratchPadOption<H>,
        tag: Option<TagId>,
    },
    PrevScratchPadWindow {
        scratchpad: ScratchPadName,
    },
    NextScratchPadWindow {
        scratchpad: ScratchPadName,
    },
    ToggleScratchPad(ScratchPadName),
    ToggleFullScreen,
    ToggleMaximized,
    ToggleSticky,
    ToggleAbove,
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
    SwapWindowTop {
        swap: bool,
    },
    FocusNextTag {
        behavior: FocusDeltaBehavior,
    },
    FocusPreviousTag {
        behavior: FocusDeltaBehavior,
    },
    FocusWindow(String),
    FocusWindowUp,
    FocusWindowDown,
    FocusWindowTop {
        swap: bool,
    },
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    SendWindowToTag {
        #[serde(bound = "")]
        window: Option<WindowHandle<H>>,
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
    NextLayout,
    PreviousLayout,
    SetLayout(String),
    RotateTag,
    IncreaseMainWidth(i32), // deprecated: use IncreaseMainSize instead
    DecreaseMainWidth(i32), // deprecated: use DecreaseMainSize instead
    IncreaseMainSize(i32),
    DecreaseMainSize(i32),
    IncreaseMainCount(),
    DecreaseMainCount(),
    SetMarginMultiplier(f32),
    SendWorkspaceToTag(usize, usize),
    CloseAllOtherWindows,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum FocusDeltaBehavior {
    Default,
    IgnoreUsed,
    IgnoreEmpty,
}
