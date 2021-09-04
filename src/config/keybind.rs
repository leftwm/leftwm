use crate::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Keybind<CMD> {
    pub command: Command<CMD>,
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}
