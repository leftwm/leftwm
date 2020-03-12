mod keybind;
mod theme_setting;
mod workspace_config;

use super::Command;
use crate::errors::Result;
use std::default::Default;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use toml;
use xdg::BaseDirectories;

pub use keybind::Keybind;
pub use theme_setting::ThemeSetting;
pub use workspace_config::WorkspaceConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub modkey: String,
    pub workspaces: Option<Vec<WorkspaceConfig>>,
    pub tags: Option<Vec<String>>,
    pub keybind: Vec<Keybind>,
}

/// Path to file where state will be dumper upon soft reload.
pub const STATE_FILE: &str = "/tmp/leftwm.state";

pub fn load() -> Config {
    load_from_file()
        .map_err(|err| eprintln!("ERROR LOADING CONFIG: {:?}", err))
        .unwrap_or_default()
}

fn load_from_file() -> Result<Config> {
    let path = BaseDirectories::with_prefix("leftwm")?;
    let config_filename = path.place_config_file("config.toml")?;
    if Path::new(&config_filename).exists() {
        let contents = fs::read_to_string(config_filename)?;
        Ok(toml::from_str(&contents)?)
    } else {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let mut file = File::create(&config_filename)?;
        file.write_all(&toml.as_bytes())?;
        Ok(config)
    }
}

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

fn default_terminal<'s>() -> &'s str {
    // order on least common to most common.
    // the thinking is if a person has a uncommon terminal install it is intentional
    let terms = vec![
        "alacritty",
        "termite",
        "urxvt",
        "rxvt",
        "st",
        "roxterm",
        "eterm",
        "xterm",
        "terminator",
        "terminology",
        "gnome-terminal",
    ];
    for t in terms {
        if is_program_in_path(t) {
            return t;
        }
    }
    "termite" //no terninal found in path, default to a good one
}

impl Config {
    /// Returns a collection of bindings with the mod key mapped.
    pub fn mapped_bindings(&self) -> Vec<Keybind> {
        // copy keybinds substituting "modkey" modifier with a new "modkey".
        self.keybind
            .clone()
            .into_iter()
            .map(|mut keybind| {
                for m in &mut keybind.modifier {
                    if m == "modkey" {
                        *m = self.modkey.clone();
                    }
                }
                keybind
            })
            .collect()
    }

    pub fn get_list_of_tags(&self) -> Vec<String> {
        if let Some(tags) = &self.tags {
            return tags.clone();
        }
        Config::default().tags.unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut commands = vec![
            // Mod + p => Open dmenu
            Keybind {
                command: Command::Execute,
                value: Some("dmenu_run".to_owned()),
                modifier: vec!["modkey".to_owned()],
                key: "p".to_owned(),
            },
            // Mod + Shift + Enter => Open A Shell
            Keybind {
                command: Command::Execute,
                value: Some(default_terminal().to_owned()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Return".to_owned(),
            },
            // Mod + Shift + q => kill focused window
            Keybind {
                command: Command::CloseWindow,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "q".to_owned(),
            },
            // Mod + Shift + r => soft reload leftwm
            Keybind {
                command: Command::SoftReload,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "r".to_owned(),
            },
            // Mod + Shift + x => exit leftwm
            Keybind {
                command: Command::Execute,
                value: Some("pkill leftwm".to_owned()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "x".to_owned(),
            },
            // Mod + Ctrl + l => lock the screen
            Keybind {
                command: Command::Execute,
                value: Some("slock".to_owned()),
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "l".to_owned(),
            },
            // Mod + Shift + w => swap the tags on the last to active workspaces
            Keybind {
                command: Command::MoveToLastWorkspace,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "w".to_owned(),
            },
            // Mod + w => move the active window to the previous workspace
            Keybind {
                command: Command::SwapTags,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "w".to_owned(),
            },
            Keybind {
                command: Command::MoveWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: Command::MoveWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: Command::FocusWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: Command::FocusWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: Command::NextLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: Command::PreviousLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: Command::FocusWorkspaceNext,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Right".to_owned(),
            },
            Keybind {
                command: Command::FocusWorkspacePrevious,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Left".to_owned(),
            },
        ];

        const WORKSPACES_NUM: usize = 10;

        // add "goto workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: Command::GotoTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned()],
                key: i.to_string(),
            });
        }

        // and "move to workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: Command::MoveToTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: i.to_string(),
            });
        }

        let tags = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Config {
            workspaces: Some(vec![]),
            tags: Some(tags),
            modkey: "Mod4".to_owned(), //win key
            keybind: commands,
        }
    }
}
