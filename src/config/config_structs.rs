use super::utils::command::Command;

use std::default::Default;
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    modkey: String,
    keybind: Vec<Keybind>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keybind {
    command: Command,
    value: Option<String>,
    modifier: Vec<String>,
    key: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands: Vec<Keybind> = vec![];

        //add goto workspace
        for i in 1..10 {
            commands.push(Keybind {
                command: Command::GotoWorkspace,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned()],
                key: i.to_string(),
            });
        }

        //add move to workspace
        for i in 1..10 {
            commands.push(Keybind {
                command: Command::MovetoWorkspace,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: i.to_string(),
            });
        }

        Config {
            modkey: "Mod4".to_owned(),
            keybind: commands,
        }
    }
}
