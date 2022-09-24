pub use crate::handlers::command_handler::ReleaseScratchPadOption;
use crate::{
    layouts::Layout,
    models::{ScratchPadName, TagId, WindowHandle},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Command {
    CloseWindow,
    SwapScreens,
    SoftReload,
    HardReload,
    AttachScratchPad {
        window: Option<WindowHandle>,
        scratchpad: ScratchPadName,
    },
    ReleaseScratchPad {
        window: ReleaseScratchPadOption,
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
