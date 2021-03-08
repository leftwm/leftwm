use super::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Keybind {
    pub command: Command,
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}
