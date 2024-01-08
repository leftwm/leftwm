use serde::{Deserialize, Serialize};

/// Leftwm has 2 layout modes, Workspace and Tag. These determine how layouts are remembered.
/// When in Workspace mode, layouts will be remembered per workspace.
/// When in Tag mode, layouts are remembered per tag.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Tag,
    Workspace,
}

impl Default for LayoutMode {
    fn default() -> Self {
        Self::Tag
    }
}
