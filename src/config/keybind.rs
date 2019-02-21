use super::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: Command,
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}
