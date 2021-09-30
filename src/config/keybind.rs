use crate::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: Command,
    pub modifier: Vec<String>,
    pub key: String,
}
