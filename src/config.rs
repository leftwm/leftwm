mod keybind;
mod workspace_config;

use super::Command;
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use toml;
use xdg::BaseDirectories;

pub use keybind::Keybind;
pub use workspace_config::WorkspaceConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub modkey: String,
    pub workspace: Vec<WorkspaceConfig>,
    pub keybind: Vec<Keybind>,
}

pub fn load() -> Config {
    match load_from_file() {
        Ok(cfg) => cfg,
        Err(_) => Config::default(),
    }
}

fn load_from_file() -> Result<Config, Box<std::error::Error>> {
    let path = BaseDirectories::with_prefix("leftwm")?;
    let config_filename = path.place_config_file("config.toml")?;
    if Path::new(&config_filename).exists() {
        let contents = fs::read_to_string(config_filename)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let mut file = File::create(&config_filename)?;
        file.write_all(&toml.as_bytes())?;
        Ok(config)
    }
}

impl Config {
    /*
     * returns a collection of bindings with the mod key mapped
     */
    pub fn mapped_bindings(&self) -> Vec<Keybind> {
        let mod_key: &String = &self.modkey.clone();
        let old_binds: &Vec<Keybind> = &self.keybind;
        old_binds
            .iter()
            .map(|k| {
                let mut keymap = k.clone();
                let old_mods: &Vec<String> = &k.modifier;
                let mods = old_mods
                    .iter()
                    .map(|m| {
                        if m == "modkey" {
                            mod_key.clone()
                        } else {
                            m.clone()
                        }
                    })
                    .collect();
                keymap.modifier = mods;
                keymap
            })
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut commands: Vec<Keybind> = vec![];

        //Mod + Shift + Enter => Open dmenu
        commands.push(Keybind {
            command: Command::Execute,
            value: Some("dmenu_run".to_owned()),
            modifier: vec!["modkey".to_owned()],
            key: "p".to_owned(),
        });

        //Mod + Shift + Enter => Open A Shell
        commands.push(Keybind {
            command: Command::Execute,
            value: Some("termite".to_owned()),
            modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
            key: "Return".to_owned(),
        });

        //Mod + Shift + q => kill focused window
        commands.push(Keybind {
            command: Command::CloseWindow,
            value: None,
            modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
            key: "q".to_owned(),
        });

        //Mod + crtl + l => lock the screen
        commands.push(Keybind {
            command: Command::Execute,
            value: Some("i3lock --color 000000".to_owned()),
            modifier: vec!["modkey".to_owned(), "Control".to_owned()],
            key: "l".to_owned(),
        });

        //Mod + Shift + w => swap the tags on the last to active workspaces
        commands.push(Keybind {
            command: Command::SwapTags,
            value: None,
            modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
            key: "w".to_owned(),
        });

        //Mod + w => move the active window to the previous workspace
        commands.push(Keybind {
            command: Command::SwapTags,
            value: None,
            modifier: vec!["modkey".to_owned()],
            key: "w".to_owned(),
        });

        //add goto workspace
        for i in 1..10 {
            commands.push(Keybind {
                command: Command::GotoTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned()],
                key: i.to_string(),
            });
        }

        //add move to workspace
        for i in 1..10 {
            commands.push(Keybind {
                command: Command::MoveToTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: i.to_string(),
            });
        }

        Config {
            workspace: vec![],
            modkey: "Mod4".to_owned(), //win key
            keybind: commands,
        }
    }
}
