//! `LeftWM` general configuration

use super::{BaseCommand, ThemeSetting};
use anyhow::{Context, Result};
use leftwm_core::{
    config::{ScratchPad, Workspace},
    layouts::{Layout, LAYOUTS},
    models::{FocusBehaviour, Gutter, LayoutMode, Margins, Size},
    state::State,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::default::Default;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use xdg::BaseDirectories;

/// Path to file where state will be dumper upon soft reload.
const STATE_FILE: &str = "/tmp/leftwm.state";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: BaseCommand,
    pub value: Option<String>,
    pub modifier: Vec<String>,
    pub key: String,
}

impl TryFrom<Keybind> for leftwm_core::Keybind {
    type Error = anyhow::Error;

    fn try_from(k: Keybind) -> Result<Self> {
        let command = match k.command {
            BaseCommand::Execute => {
                leftwm_core::Command::Execute(k.value.context("missing command for Execute")?)
            }
            BaseCommand::CloseWindow => leftwm_core::Command::CloseWindow,
            BaseCommand::SwapTags => leftwm_core::Command::SwapScreens,
            BaseCommand::SoftReload => leftwm_core::Command::SoftReload,
            BaseCommand::HardReload => leftwm_core::Command::HardReload,
            BaseCommand::ToggleScratchPad => leftwm_core::Command::ToggleScratchPad(
                k.value.context("missing name for ToggleScratchPad")?,
            ),
            BaseCommand::ToggleFullScreen => leftwm_core::Command::ToggleFullScreen,
            BaseCommand::ToggleSticky => leftwm_core::Command::ToggleSticky,
            BaseCommand::GotoTag => leftwm_core::Command::GotoTag(
                usize::from_str(&k.value.context("missing index value for GotoTag")?)
                    .context("invalid index value for GotoTag")?,
            ),
            BaseCommand::FloatingToTile => leftwm_core::Command::FloatingToTile,
            BaseCommand::MoveWindowUp => leftwm_core::Command::MoveWindowUp,
            BaseCommand::MoveWindowDown => leftwm_core::Command::MoveWindowDown,
            BaseCommand::MoveWindowTop => leftwm_core::Command::MoveWindowTop,
            BaseCommand::FocusNextTag => leftwm_core::Command::FocusNextTag,
            BaseCommand::FocusPreviousTag => leftwm_core::Command::FocusPreviousTag,
            BaseCommand::FocusWindowUp => leftwm_core::Command::FocusWindowUp,
            BaseCommand::FocusWindowDown => leftwm_core::Command::FocusWindowDown,
            BaseCommand::FocusWorkspaceNext => leftwm_core::Command::FocusWorkspaceNext,
            BaseCommand::FocusWorkspacePrevious => leftwm_core::Command::FocusWorkspacePrevious,
            BaseCommand::MoveToTag => leftwm_core::Command::SendWindowToTag(
                usize::from_str(&k.value.context("missing index value for SendWindowToTag")?)
                    .context("invalid index value for SendWindowToTag")?,
            ),
            BaseCommand::MoveToLastWorkspace => leftwm_core::Command::MoveWindowToLastWorkspace,
            BaseCommand::MoveWindowToNextWorkspace => {
                leftwm_core::Command::MoveWindowToNextWorkspace
            }
            BaseCommand::MoveWindowToPreviousWorkspace => {
                leftwm_core::Command::MoveWindowToPreviousWorkspace
            }
            BaseCommand::MouseMoveWindow => leftwm_core::Command::MouseMoveWindow,
            BaseCommand::NextLayout => leftwm_core::Command::NextLayout,
            BaseCommand::PreviousLayout => leftwm_core::Command::PreviousLayout,
            BaseCommand::SetLayout => leftwm_core::Command::SetLayout(
                Layout::from_str(&k.value.context("missing layout for SetLayout")?)
                    .context("could not parse layout for command SetLayout")?,
            ),
            BaseCommand::RotateTag => leftwm_core::Command::RotateTag,
            BaseCommand::IncreaseMainWidth => leftwm_core::Command::IncreaseMainWidth(
                i8::from_str(&k.value.context("missing value for IncreaseMainWidth")?)
                    .context("invalid value for IncreaseMainWidth")?,
            ),
            BaseCommand::DecreaseMainWidth => leftwm_core::Command::DecreaseMainWidth(
                i8::from_str(&k.value.context("missing value for DecreaseMainWidth")?)
                    .context("invalid value for DecreaseMainWidth")?,
            ),
            BaseCommand::SetMarginMultiplier => leftwm_core::Command::SetMarginMultiplier(
                f32::from_str(
                    &k.value
                        .context("missing index value for SetMarginMultiplier")?,
                )
                .context("invalid index value for SetMarginMultiplier")?,
            ),
            BaseCommand::UnloadTheme => leftwm_core::Command::Other("UnloadTheme".into()),
            BaseCommand::LoadTheme => leftwm_core::Command::Other(format!(
                "LoadTheme {}",
                k.value.context("missing index value for LoadTheme")?,
            )),
        };

        Ok(Self {
            command,
            modifier: k.modifier,
            key: k.key,
        })
    }
}

/// General configuration
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub modkey: String,
    pub mousekey: String,
    pub workspaces: Option<Vec<Workspace>>,
    pub tags: Option<Vec<String>>,
    pub max_window_width: Option<Size>,
    pub layouts: Vec<Layout>,
    pub layout_mode: LayoutMode,
    pub scratchpad: Option<Vec<ScratchPad>>,
    //of you are on tag "1" and you goto tag "1" this takes you to the previous tag
    pub disable_current_tag_swap: bool,
    pub focus_behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub keybind: Vec<Keybind>,
    pub state: Option<PathBuf>,

    #[serde(skip)]
    pub theme_setting: ThemeSetting,
}

#[must_use]
pub fn load() -> Config {
    load_from_file()
        .map_err(|err| eprintln!("ERROR LOADING CONFIG: {:?}", err))
        .unwrap_or_default()
}

/// # Panics
///
/// Function can only panic if toml cannot be serialized. This should not occur as it is defined
/// globally.
///
/// # Errors
///
/// Function will throw an error if `BaseDirectories` doesn't exist, if user doesn't have
/// permissions to place config.toml, if config.toml cannot be read (access writes, malformed file,
/// etc.).
/// Function can also error from inability to save config.toml (if it is the first time running
/// `LeftWM`).
fn load_from_file() -> Result<Config> {
    let path = BaseDirectories::with_prefix("leftwm")?;
    let config_filename = path.place_config_file("config.toml")?;
    if Path::new(&config_filename).exists() {
        let contents = fs::read_to_string(config_filename)?;
        let config = toml::from_str(&contents)?;
        if check_workspace_ids(&config) {
            Ok(config)
        } else {
            log::warn!("Invalid workspace ID configuration in config.toml. Falling back to default config.");
            Ok(Config::default())
        }
    } else {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let mut file = File::create(&config_filename)?;
        file.write_all(toml.as_bytes())?;
        Ok(config)
    }
}

#[must_use]
pub fn check_workspace_ids(config: &Config) -> bool {
    config.workspaces.clone().map_or(true, |wss| {
        let ids = get_workspace_ids(&wss);
        if ids.iter().any(Option::is_some) {
            all_ids_some(&ids) && all_ids_unique(&ids)
        } else {
            true
        }
    })
}

#[must_use]
pub fn get_workspace_ids(wss: &[Workspace]) -> Vec<Option<i32>> {
    wss.iter().map(|ws| ws.id).collect()
}

pub fn all_ids_some(ids: &[Option<i32>]) -> bool {
    ids.iter().all(Option::is_some)
}

#[must_use]
pub fn all_ids_unique(ids: &[Option<i32>]) -> bool {
    let mut sorted = ids.to_vec();
    sorted.sort();
    sorted.dedup();
    ids.len() == sorted.len()
}

#[must_use]
pub fn is_program_in_path(program: &str) -> bool {
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

/// Returns a terminal to set for the default mod+shift+enter keybind.
fn default_terminal<'s>() -> &'s str {
    // order from least common to most common.
    // the thinking is if a machine has a uncommon terminal install it is intentional
    let terms = vec![
        "alacritty",
        "termite",
        "kitty",
        "urxvt",
        "rxvt",
        "st",
        "roxterm",
        "eterm",
        "xterm",
        "terminator",
        "terminology",
        "gnome-terminal",
        "xfce4-terminal",
        "konsole",
        "uxterm",
        "guake", // at the bottom because of odd behaviour. guake wants F12 and should really be
                 // started using autostart instead of LeftWM keybind.
    ];
    for t in terms {
        if is_program_in_path(t) {
            return t;
        }
    }
    "termite" //no terminal found in path, default to a good one
}

/// Returns default keybind value for exiting `LeftWM`.
// On systems that have elogind and/or systemd, the recommended way to
// kill LeftWM is to use loginctl. As we have no consistent way of knowing
// whether it is implemented on non-systemd machines,so we instead look
// to see if loginctl is in the path. If it isn't then we default to
// `pkill leftwm`, which may leave zombie processes on a machine.
fn exit_strategy<'s>() -> &'s str {
    if is_program_in_path("loginctl") {
        return "loginctl kill-session $XDG_SESSION_ID";
    }
    "pkill leftwm"
}

fn absolute_path(path: &str) -> Option<PathBuf> {
    let exp_path = shellexpand::full(path).ok()?;
    std::fs::canonicalize(exp_path.as_ref()).ok()
}

impl leftwm_core::Config for Config {
    fn mapped_bindings(&self) -> Vec<leftwm_core::Keybind> {
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
            .filter_map(|keybind| match TryFrom::try_from(keybind.clone()) {
                Ok(internal_keybind) => Some(internal_keybind),
                Err(err) => {
                    log::error!("Invalid key binding: {}\n{:?}", err, keybind);
                    None
                }
            })
            .collect()
    }

    fn create_list_of_tags(&self) -> Vec<String> {
        if let Some(tags) = &self.tags {
            return tags.clone();
        }
        Self::default()
            .tags
            .expect("we created it in the Default impl; qed")
    }

    fn workspaces(&self) -> Option<&[Workspace]> {
        self.workspaces.as_deref()
    }

    fn focus_behaviour(&self) -> FocusBehaviour {
        self.focus_behaviour
    }

    fn mousekey(&self) -> &str {
        &self.mousekey
    }

    fn disable_current_tag_swap(&self) -> bool {
        self.disable_current_tag_swap
    }

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        if let Some(scratchpads) = &self.scratchpad {
            return scratchpads.clone();
        }
        return vec![];
    }

    fn layouts(&self) -> Vec<Layout> {
        self.layouts.clone()
    }

    fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    fn focus_new_windows(&self) -> bool {
        self.focus_new_windows
    }

    fn command_handler(command: &str, state: &mut State<Self>) -> bool {
        if let Some((command, value)) = command.split_once(' ') {
            match command {
                "LoadTheme" => {
                    if let Some(absolute) = absolute_path(value.trim()) {
                        state.config.theme_setting.load(absolute);
                    } else {
                        log::warn!("Path submitted does not exist.");
                    }
                    return state.update_for_theme();
                }
                "UnloadTheme" => {
                    state.config.theme_setting = Default::default();
                    return state.update_for_theme();
                }
                _ => {
                    log::warn!("Command not recognized: {}", command);
                    return false;
                }
            }
        }
        false
    }

    fn border_width(&self) -> i32 {
        self.theme_setting.border_width
    }

    fn margin(&self) -> Margins {
        match self.theme_setting.margin.clone().try_into() {
            Ok(margins) => margins,
            Err(err) => {
                log::warn!("Could not read margin: {}", err);
                Margins::new(0)
            }
        }
    }

    fn workspace_margin(&self) -> Option<Margins> {
        self.theme_setting
            .workspace_margin
            .clone()
            .and_then(|custom_margin| match custom_margin.try_into() {
                Ok(margins) => Some(margins),
                Err(err) => {
                    log::warn!("Could not read margin: {}", err);
                    None
                }
            })
    }

    fn gutter(&self) -> Option<Vec<Gutter>> {
        self.theme_setting.gutter.clone()
    }

    fn default_border_color(&self) -> &str {
        &self.theme_setting.default_border_color
    }

    fn floating_border_color(&self) -> &str {
        &self.theme_setting.floating_border_color
    }

    fn focused_border_color(&self) -> &str {
        &self.theme_setting.focused_border_color
    }

    fn on_new_window_cmd(&self) -> Option<String> {
        self.theme_setting.on_new_window_cmd.clone()
    }

    fn get_list_of_gutters(&self) -> Vec<Gutter> {
        self.theme_setting.gutter.clone().unwrap_or_default()
    }

    fn max_window_width(&self) -> Option<Size> {
        self.max_window_width
    }

    fn save_state(state: &State<Self>) {
        let path = state.config.state_file();
        let state_file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Cannot create file at path {}: {}", path.display(), err);
                return;
            }
        };
        if let Err(err) = serde_json::to_writer(state_file, state) {
            log::error!("Cannot save state: {}", err);
        }
    }

    fn load_state(state: &mut State<Self>) {
        let path = state.config.state_file().to_owned();
        match File::open(&path) {
            Ok(file) => {
                match serde_json::from_reader(file) {
                    Ok(old_state) => state.restore_state(&old_state),
                    Err(err) => log::error!("Cannot load old state: {}", err),
                }
                // Clean old state.
                if let Err(err) = std::fs::remove_file(&path) {
                    log::error!("Cannot remove old state file: {}", err);
                }
            }
            Err(err) => log::error!("Cannot open old state: {}", err),
        }
    }
}

impl Config {
    fn state_file(&self) -> &Path {
        self.state
            .as_deref()
            .unwrap_or_else(|| Path::new(STATE_FILE))
    }
}

impl Default for Config {
    // We allow this because this function would be difficult to reduce. If someone would like to
    // move the commands builder out, perhaps make a macro, this function could be reduced in size
    // considerably.
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        const WORKSPACES_NUM: usize = 10;
        let mut commands = vec![
            // Mod + p => Open dmenu
            Keybind {
                command: BaseCommand::Execute,
                value: Some("dmenu_run".to_owned()),
                modifier: vec!["modkey".to_owned()],
                key: "p".to_owned(),
            },
            // Mod + Shift + Enter => Open A Shell
            Keybind {
                command: BaseCommand::Execute,
                value: Some(default_terminal().to_owned()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Return".to_owned(),
            },
            // Mod + Shift + q => kill focused window
            Keybind {
                command: BaseCommand::CloseWindow,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "q".to_owned(),
            },
            // Mod + Shift + r => soft reload leftwm
            Keybind {
                command: BaseCommand::SoftReload,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "r".to_owned(),
            },
            // Mod + Shift + x => exit leftwm
            Keybind {
                command: BaseCommand::Execute,
                value: Some(exit_strategy().to_owned()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "x".to_owned(),
            },
            // Mod + Ctrl + l => lock the screen
            Keybind {
                command: BaseCommand::Execute,
                value: Some("slock".to_owned()),
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "l".to_owned(),
            },
            // Mod + Shift + w => swap the tags on the last to active workspaces
            Keybind {
                command: BaseCommand::MoveToLastWorkspace,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "w".to_owned(),
            },
            // Mod + w => move the active window to the previous workspace
            Keybind {
                command: BaseCommand::SwapTags,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "w".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowTop,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Return".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::NextLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::PreviousLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspaceNext,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "l".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspacePrevious,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "h".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowUp,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowDown,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::NextLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::PreviousLayout,
                value: None,
                modifier: vec!["modkey".to_owned(), "Control".to_owned()],
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspaceNext,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Right".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspacePrevious,
                value: None,
                modifier: vec!["modkey".to_owned()],
                key: "Left".to_owned(),
            },
        ];

        // add "goto workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: BaseCommand::GotoTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned()],
                key: i.to_string(),
            });
        }

        // and "move to workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: BaseCommand::MoveToTag,
                value: Some(i.to_string()),
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: i.to_string(),
            });
        }

        let tags = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        Self {
            workspaces: Some(vec![]),
            tags: Some(tags),
            layouts: LAYOUTS.to_vec(),
            layout_mode: LayoutMode::Workspace,
            // TODO: add sane default for scratchpad config.
            // Currently default values are set in sane_dimension fn.
            scratchpad: Some(vec![]),
            disable_current_tag_swap: false,
            focus_behaviour: FocusBehaviour::Sloppy, // default behaviour: mouse move auto-focuses window
            focus_new_windows: true, // default behaviour: focuses windows on creation
            modkey: "Mod4".to_owned(), //win key
            mousekey: "Mod4".to_owned(), //win key
            keybind: commands,
            theme_setting: ThemeSetting::default(),
            max_window_width: None,
            state: None,
        }
    }
}
