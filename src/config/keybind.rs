use crate::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: Command,
    // TODO DELETE
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}
