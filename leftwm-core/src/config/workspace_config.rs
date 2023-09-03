use serde::{Deserialize, Serialize};

use crate::models::Size;

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct Workspace {
    pub output: Option<WorkspaceOutput>,
    pub options: Option<WorkspaceOptions>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkspaceOptions {
    pub max_window_width: Option<Size>,
    pub layouts: Option<Vec<String>>,
    pub pinned_tags: Option<Vec<String>>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkspaceOutput {
    pub output: String,
    pub relative: Option<bool>,
    pub x: i32,
    pub y: i32,
    pub height: i32,
    pub width: i32,
}
