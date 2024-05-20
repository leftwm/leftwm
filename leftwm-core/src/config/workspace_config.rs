use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct Workspace {
    pub x: i32,
    pub y: i32,
    pub height: i32,
    pub width: i32,
    pub output: String,
    pub relative: Option<bool>,
    pub layouts: Option<Vec<String>>,
}
