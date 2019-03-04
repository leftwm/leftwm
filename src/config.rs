mod keybind;

use super::Command;
use std::default::Default;

pub use keybind::Keybind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub modkey: String,
    pub keybind: Vec<Keybind>,
}

pub fn load() -> Config {
    Config::default()
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
            //modkey: "Mod1".to_owned(), //alt
            modkey: "Mod4".to_owned(), //win key
            keybind: commands,
        }
    }
}
