use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    pub x: i32,
    pub y: i32,
    pub height: i32,
    pub width: i32,
}
