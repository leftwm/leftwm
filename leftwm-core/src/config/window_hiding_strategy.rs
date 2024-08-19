use serde::{Deserialize, Serialize};

/// The stategy used to hide windows when switching tags in the backend
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WindowHidingStrategy {
    /// The common behaviour for a window manager, but it prevents hidden windows from being
    /// captured by other applications
    Unmap,
    /// Move the windows out of the visible area, so it can still be captured by some applications.
    /// We still inform the window that it is in a "minimized"-like state, so it can probably
    /// decide to not render its content as if it was focused.
    MoveMinimize,
    /// Move the windows out of the visible area and don't minilize them.
    /// This should allow all applications to be captured by any other applications.
    /// This could result in higher resource usage, since windows will render their content like
    /// normal even if hidden.
    #[default]
    MoveOnly,
}
