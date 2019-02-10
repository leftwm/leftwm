use super::utils::command::Command;

use std::default::Default;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub modkey: String,
    pub keybind: Vec<Keybind>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: Command,
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands: Vec<Keybind> = vec![];

        //Alt + Shift + Enter => Open A Shell
        commands.push(Keybind {
            command: Command::Execute,
            value: Some("termite".to_owned()),
            modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
            key: "Enter".to_owned(),
        });

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
            modkey: "Mod1".to_owned(),
            //modkey: "Mod4".to_owned(),
            keybind: commands,
        }
    }
}
