//! `LeftWM` general configuration

mod checks;
mod default;
mod keybind;

use self::keybind::Modifier;

#[cfg(feature = "lefthk")]
use super::BaseCommand;
use super::ThemeSetting;
#[cfg(feature = "lefthk")]
use crate::config::keybind::Keybind;
use anyhow::Result;
use leftwm_core::{
    config::{InsertBehavior, ScratchPad, Workspace},
    layouts::LayoutMode,
    models::{FocusBehaviour, Gutter, Margins, Size, Window, WindowState, WindowType, Handle},
    state::State,
    DisplayAction, DisplayServer, Manager, ReturnPipe,
};

use leftwm_layouts::Layout;
use regex::Regex;
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::Write;
use std::path::{Path, PathBuf};
use std::{convert::TryInto, fs::OpenOptions};
use std::{default::Default, error::Error};
use xdg::BaseDirectories;

/// Path to file where state will be dumped upon soft reload.
const STATE_FILE: &str = "/tmp/leftwm.state";

/// Selecting by `WM_CLASS` and/or window title, allow the user to define if a
/// window should spawn on a specified tag and/or its floating state.
///
/// # Example
///
///
/// In `config.ron`
///
/// ```ron
/// window_rules: [
///     (window_class: "krita", spawn_on_tag: 3, spawn_floating: false),
/// ]
/// ```
///
/// In the deprecated `config.toml`
///
/// ```toml
/// [[window_config_by_class]]
/// wm_class = "krita"
/// spawn_on_tag = 3
/// spawn_floating = false
/// ```
///
/// windows whose `WM_CLASS` is "krita" will spawn on tag 3 (1-indexed) and not floating.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct WindowHook {
    // Use serde default field attribute to fallback to None option in case of missing field in
    // config. Without this attribute deserializer will fail on missing field due to it's inability
    // to treat missing value as Option::None
    /// `WM_CLASS` in X11
    #[serde(
        default,
        deserialize_with = "from_regex",
        serialize_with = "to_config_string"
    )]
    pub window_class: Option<Regex>,
    /// `_NET_WM_NAME` in X11
    #[serde(
        default,
        deserialize_with = "from_regex",
        serialize_with = "to_config_string"
    )]
    pub window_title: Option<Regex>,
    pub spawn_on_tag: Option<usize>,
    pub spawn_on_workspace: Option<usize>,
    pub spawn_floating: Option<bool>,
    pub spawn_sticky: Option<bool>,
    pub spawn_fullscreen: Option<bool>,
    /// Handle the window as if it was of this `_NET_WM_WINDOW_TYPE`
    pub spawn_as_type: Option<WindowType>,
}

impl WindowHook {
    /// Score the similarity between a [`leftwm_core::models::Window`] and a [`WindowHook`].
    ///
    /// Multiple [`WindowHook`]s might match a `WM_CLASS` but we want the most
    /// specific one to apply: matches by title are scored greater than by `WM_CLASS`.
    fn score_window<H: Handle>(&self, window: &Window<H>) -> u8 {
        // returns true if any of the items in the provided `Vec<&Option<String>>` is Some and matches the `&Regex`
        let matches_any = |re: &Regex, strs: Vec<&Option<String>>| {
            strs.iter().any(|str| {
                str.as_ref().map_or(false, |s| {
                    // we match the class/title to the window rule by checking if replacing the text
                    // with the regex makes the string empty. if the original string is already
                    // empty, this will match it to every regex, so we need to check for that.
                    // however, if the window rule is explicitly for empty strings, we still
                    // want empty strings to match to it.
                    re.replace(s, "") == "" && (!s.is_empty() || re.as_str().is_empty())
                })
            })
        };

        let class_score = self.window_class.as_ref().map_or(0, |re| {
            u8::from(matches_any(re, vec![&window.res_class, &window.res_name]))
        });

        let title_score = self.window_title.as_ref().map_or(0, |re| {
            u8::from(matches_any(re, vec![&window.legacy_name, &window.name]))
        });

        class_score + 2 * title_score
    }

    fn apply<H: Handle>(&self, state: &mut State<H>, window: &mut Window<H>) {
        if let Some(tag) = self.spawn_on_tag {
            window.tag = Some(tag);
        }
        if self.spawn_on_workspace.is_some() {
            if let Some(workspace) = state
                .workspaces
                .iter()
                .find(|ws| Some(ws.id) == self.spawn_on_workspace)
            {
                if let Some(tag) = workspace.tag {
                    // In order to apply the correct margin multiplier we want to copy this value
                    // from any window already present on the target tag
                    let margin_multiplier = match state.windows.iter().find(|w| w.has_tag(&tag)) {
                        Some(w) => w.margin_multiplier(),
                        None => 1.0,
                    };

                    window.untag();
                    window.set_floating(self.spawn_floating.unwrap_or_default());
                    window.tag(&tag);
                    window.apply_margin_multiplier(margin_multiplier);
                    let act = DisplayAction::SetWindowTag(window.handle, Some(tag));
                    state.actions.push_back(act);

                    state.sort_windows();
                }
            } else {
                // the function is called directly before the hook is displayed in debug mode.
                tracing::debug!("Could not find workspace for following hook.");
            }
        }
        if let Some(should_float) = self.spawn_floating {
            window.set_floating(should_float);
        }
        if let Some(fullscreen) = self.spawn_fullscreen {
            let act = DisplayAction::SetState(window.handle, fullscreen, WindowState::Fullscreen);
            state.actions.push_back(act);
            state.handle_window_focus(&window.handle);
        }
        if let Some(sticky) = self.spawn_sticky {
            let act = DisplayAction::SetState(window.handle, sticky, WindowState::Sticky);
            state.actions.push_back(act);
            state.handle_window_focus(&window.handle);
        }
        if let Some(w_type) = self.spawn_as_type.clone() {
            window.r#type = w_type;
        }
    }
}

/// General configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub modkey: String,
    pub mousekey: Option<Modifier>,
    pub workspaces: Option<Vec<Workspace>>,
    pub tags: Option<Vec<String>>,
    pub max_window_width: Option<Size>,
    pub layouts: Vec<String>,
    pub layout_definitions: Vec<Layout>,
    pub layout_mode: LayoutMode,
    pub insert_behavior: InsertBehavior,
    pub scratchpad: Option<Vec<ScratchPad>>,
    pub window_rules: Option<Vec<WindowHook>>,
    // If you are on tag "1" and you goto tag "1" this takes you to the previous tag
    pub disable_current_tag_swap: bool,
    pub disable_tile_drag: bool,
    pub disable_window_snap: bool,
    pub focus_behaviour: FocusBehaviour,
    pub focus_new_windows: bool,
    pub single_window_border: bool,
    pub sloppy_mouse_follows_focus: bool,
    pub create_follows_cursor: Option<bool>,
    pub auto_derive_workspaces: bool,
    pub disable_cursor_reposition_on_resize: bool,
    #[cfg(feature = "lefthk")]
    pub keybind: Vec<Keybind>,
    pub state_path: Option<PathBuf>,
    // NOTE: any newly added parameters must be inserted before `pub keybind: Vec<Keybind>,`
    //       at least when `TOML` is used as config language
    #[serde(skip)]
    pub theme_setting: ThemeSetting,
}

#[must_use]
pub fn load() -> Config {
    load_from_file()
        .map_err(|err| eprintln!("ERROR LOADING CONFIG: {err:?}"))
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
    tracing::debug!("Loading config file");

    let path = BaseDirectories::with_prefix("leftwm")?;

    // the checks and fallback for `toml` can be removed when toml gets eventually deprecated
    let config_file_ron = path.place_config_file("config.ron")?;
    let config_file_toml = path.place_config_file("config.toml")?;

    if Path::new(&config_file_ron).exists() {
        tracing::debug!("Config file '{}' found.", config_file_ron.to_string_lossy());
        let ron = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        let contents = fs::read_to_string(config_file_ron)?;
        let config: Config = ron.from_str(&contents)?;
        Ok(config)
    } else if Path::new(&config_file_toml).exists() {
        tracing::debug!(
            "Config file '{}' found.",
            config_file_toml.to_string_lossy()
        );
        let contents = fs::read_to_string(config_file_toml)?;
        let config = toml::from_str(&contents)?;
        tracing::info!("You are using TOML as config language which will be deprecated in the future.\nPlease consider migrating you config to RON. For further info visit the leftwm wiki.");
        Ok(config)
    } else {
        tracing::debug!("Config file not found. Using default config file.");

        let config = Config::default();
        let ron_pretty_conf = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        let ron = to_string_pretty(&config, ron_pretty_conf).unwrap();
        let comment_header = String::from(
            r"//  _        ___                                      ___ _
// | |      / __)_                                   / __|_)
// | | ____| |__| |_ _ _ _ ____      ____ ___  ____ | |__ _  ____    ____ ___  ____
// | |/ _  )  __)  _) | | |    \    / ___) _ \|  _ \|  __) |/ _  |  / ___) _ \|  _ \
// | ( (/ /| |  | |_| | | | | | |  ( (__| |_| | | | | |  | ( ( | |_| |  | |_| | | | |
// |_|\____)_|   \___)____|_|_|_|   \____)___/|_| |_|_|  |_|\_|| (_)_|   \___/|_| |_|
// A WindowManager for Adventurers                         (____/
// For info about configuration please visit https://github.com/leftwm/leftwm/wiki

",
        );
        let ron_with_header = comment_header + &ron;

        let mut file = File::create(&config_file_ron)?;
        file.write_all(ron_with_header.as_bytes())?;

        Ok(config)
    }
}

#[must_use]
pub fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{p}/{program}");
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Returns a terminal to set for the default mod+shift+enter keybind.
#[cfg(feature = "lefthk")]
fn default_terminal<'s>() -> &'s str {
    // order from least common to most common.
    // the thinking is if a machine has an uncommon terminal installed, it is intentional
    let terms = &[
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

    // If no terminal found in path, default to a good one
    terms
        .iter()
        .find(|terminal| is_program_in_path(terminal))
        .unwrap_or(&"termite")
}

/// Returns default keybind value for exiting `LeftWM`.
// On systems that have elogind and/or systemd, the recommended way to
// kill LeftWM is to use loginctl. As we have no consistent way of knowing
// whether it is implemented on non-systemd machines,so we instead look
// to see if loginctl is in the path. If it isn't then we default to
// `pkill leftwm`, which may leave zombie processes on a machine.
#[cfg(feature = "lefthk")]
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

#[cfg(feature = "lefthk")]
impl lefthk_core::config::Config for Config {
    fn mapped_bindings(&self) -> Vec<lefthk_core::config::Keybind> {
        // copy keybinds substituting "modkey" modifier with a new "modkey".
        self.keybind
            .clone()
            .into_iter()
            .map(|mut keybind| {
                if let Some(ref mut modifier) = keybind.modifier {
                    match modifier {
                        Modifier::Single(m) if m == "modkey" => *m = self.modkey.clone(),
                        Modifier::List(ms) => {
                            for m in ms {
                                if m == "modkey" {
                                    *m = self.modkey.clone();
                                }
                            }
                        }
                        Modifier::Single(_) => {}
                    }
                }

                keybind
            })
            .filter_map(
                |keybind| match keybind.try_convert_to_lefthk_keybind(self) {
                    Ok(lefthk_keybind) => Some(lefthk_keybind),
                    Err(err) => {
                        tracing::error!("Invalid key binding: {}\n{:?}", err, keybind);
                        None
                    }
                },
            )
            .collect()
    }
}

impl leftwm_core::Config for Config {
    fn create_list_of_tag_labels(&self) -> Vec<String> {
        if let Some(tags) = &self.tags {
            return tags.clone();
        }
        Self::default()
            .tags
            .expect("we created it in the Default impl; qed")
    }

    fn workspaces(&self) -> Option<Vec<Workspace>> {
        self.workspaces.clone()
    }

    fn focus_behaviour(&self) -> FocusBehaviour {
        self.focus_behaviour
    }

    fn mousekey(&self) -> Vec<String> {
        self.mousekey
            .as_ref()
            .unwrap_or(&"Mod4".into())
            .clone()
            .into()
    }

    fn create_list_of_scratchpads(&self) -> Vec<ScratchPad> {
        if let Some(scratchpads) = &self.scratchpad {
            return scratchpads.clone();
        }
        vec![]
    }

    fn layouts(&self) -> Vec<String> {
        self.layouts.clone()
    }

    fn layout_definitions(&self) -> Vec<Layout> {
        let mut layouts = vec![];
        for custom_layout in &self.layout_definitions {
            layouts.push(custom_layout.clone());
        }
        layouts
    }

    fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    fn insert_behavior(&self) -> InsertBehavior {
        self.insert_behavior
    }

    fn single_window_border(&self) -> bool {
        self.single_window_border
    }

    fn focus_new_windows(&self) -> bool {
        self.focus_new_windows
    }

    fn command_handler<H: Handle, SERVER: DisplayServer<H>>(
        command: &str,
        manager: &mut Manager<H, Self, SERVER>,
    ) -> bool {
        let mut return_pipe = get_return_pipe();
        if let Some((command, value)) = command.split_once(' ') {
            match command {
                "LoadTheme" => {
                    if let Some(absolute) = absolute_path(value.trim()) {
                        manager.config.theme_setting.load(absolute);
                        write_to_pipe(&mut return_pipe, "OK: Command executed successfully");
                    } else {
                        tracing::warn!("Path submitted does not exist.");
                        write_to_pipe(&mut return_pipe, "ERROR: Path submitted does not exist");
                    }
                    manager.reload_config()
                }
                "UnloadTheme" => {
                    manager.config.theme_setting = ThemeSetting::default();
                    write_to_pipe(&mut return_pipe, "OK: Command executed successfully");
                    manager.reload_config()
                }
                _ => {
                    tracing::warn!("Command not recognized: {}", command);
                    write_to_pipe(&mut return_pipe, "ERROR: Command not recognized");
                    false
                }
            }
        } else {
            match command {
                "LoadTheme" => {
                    tracing::warn!("Missing parameter theme_path");
                    write_to_pipe(&mut return_pipe, "ERROR: Missing parameter theme_path");
                    false
                }
                "UnloadTheme" => {
                    manager.config.theme_setting = ThemeSetting::default();
                    write_to_pipe(&mut return_pipe, "OK: Command executed successfully");
                    manager.reload_config()
                }
                _ => {
                    tracing::warn!("Command not recognized: {}", command);
                    write_to_pipe(&mut return_pipe, "ERROR: Command not recognized");
                    false
                }
            }
        }
    }

    fn border_width(&self) -> i32 {
        self.theme_setting.border_width.unwrap_or(1)
    }

    fn margin(&self) -> Margins {
        match self
            .theme_setting
            .margin
            .clone()
            .unwrap_or(crate::CustomMargins::Int(10))
            .try_into()
        {
            Ok(margins) => margins,
            Err(err) => {
                tracing::warn!("Could not read margin: {}", err);
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
                    tracing::warn!("Could not read margin: {}", err);
                    None
                }
            })
    }

    fn gutter(&self) -> Option<Vec<Gutter>> {
        self.theme_setting.gutter.clone()
    }

    fn default_border_color(&self) -> String {
        self.theme_setting
            .default_border_color
            .clone()
            .unwrap_or_else(|| "#000000".to_string())
    }

    fn floating_border_color(&self) -> String {
        self.theme_setting
            .floating_border_color
            .clone()
            .unwrap_or_else(|| "#000000".to_string())
    }

    fn background_color(&self) -> String {
        self.theme_setting
            .background_color
            .clone()
            .unwrap_or_else(|| "#333333".to_string())
    }

    fn disable_window_snap(&self) -> bool {
        self.disable_window_snap
    }

    fn always_float(&self) -> bool {
        self.theme_setting.always_float.unwrap_or(false)
    }

    fn default_width(&self) -> i32 {
        self.theme_setting.default_width.unwrap_or(800)
    }

    fn default_height(&self) -> i32 {
        self.theme_setting.default_height.unwrap_or(600)
    }

    fn focused_border_color(&self) -> String {
        self.theme_setting
            .focused_border_color
            .clone()
            .unwrap_or_else(|| "#FF0000".to_string())
    }

    fn on_new_window_cmd(&self) -> Option<String> {
        self.theme_setting.on_new_window_cmd.clone()
    }

    fn get_list_of_gutters(&self) -> Vec<Gutter> {
        self.theme_setting.gutter.clone().unwrap_or_default()
    }

    fn disable_tile_drag(&self) -> bool {
        self.disable_tile_drag
    }

    fn save_state<H: Handle>(&self, state: &State<H>) {
        let path = self.state_file();
        let state_file = match File::create(path) {
            Ok(file) => file,
            Err(err) => {
                tracing::error!("Cannot create file at path {}: {}", path.display(), err);
                return;
            }
        };
        if let Err(err) = ron::ser::to_writer(state_file, state) {
            tracing::error!("Cannot save state: {}", err);
        }
    }

    fn load_state<H: Handle>(&self, state: &mut State<H>) {
        let path = self.state_file().to_owned();
        match File::open(&path) {
            Ok(file) => {
                match ron::de::from_reader(file) {
                    Ok(old_state) => state.restore_state(&old_state),
                    Err(err) => tracing::error!("Cannot load old state: {}", err),
                }

                // Clean old state.
                if let Err(err) = std::fs::remove_file(&path) {
                    tracing::error!("Cannot remove old state file: {}", err);
                }
            }
            Err(err) => tracing::error!("Cannot open old state: {}", err),
        }
    }

    /// Pick the best matching [`WindowHook`], if any, and apply its config.
    fn setup_predefined_window<H: Handle>(&self, state: &mut State<H>, window: &mut Window<H>) -> bool {
        if let Some(window_rules) = &self.window_rules {
            let best_match = window_rules
                .iter()
                // map first instead of using max_by_key directly...
                .map(|wh| (wh, wh.score_window(window)))
                // ...since this filter is required (0 := non-match)
                .filter(|(_wh, score)| score != &0)
                .max_by_key(|(_wh, score)| *score);
            if let Some((hook, _)) = best_match {
                hook.apply(state, window);
                tracing::trace!(
                    "Window [[ TITLE={:?}, {:?}; WM_CLASS={:?}, {:?} ]] spawned in tag={:?} on workspace={:?} as type={:?} with floating={:?}, sticky={:?} and fullscreen={:?}",
                    window.name,
                    window.legacy_name,
                    window.res_name,
                    window.res_class,
                    hook.spawn_as_type,
                    hook.spawn_on_tag,
                    hook.spawn_on_workspace,
                    hook.spawn_floating,
                    hook.spawn_sticky,
                    hook.spawn_fullscreen,
                );
                return true;
            }
            return false;
        }
        false
    }

    fn sloppy_mouse_follows_focus(&self) -> bool {
        self.sloppy_mouse_follows_focus
    }

    fn auto_derive_workspaces(&self) -> bool {
        self.auto_derive_workspaces
    }

    fn reposition_cursor_on_resize(&self) -> bool {
        !self.disable_cursor_reposition_on_resize
    }

    // Determines if a new window should be created under the cursor or on the workspace which has the focus
    fn create_follows_cursor(&self) -> bool {
        // If follow behaviour has been explicitly set, use that value.
        // If not, set it to true in Sloppy mode only.
        self.create_follows_cursor
            .unwrap_or(self.focus_behaviour == FocusBehaviour::Sloppy)
    }
}

impl Config {
    #[cfg(feature = "lefthk")]
    pub fn clear_keybinds(&mut self) {
        self.keybind.clear();
    }

    fn state_file(&self) -> &Path {
        self.state_path
            .as_deref()
            .unwrap_or_else(|| Path::new(STATE_FILE))
    }
}

// Regular expression in leftwm config should correspond to RE2 syntax, described here:
// https://github.com/google/re2/wiki/Syntax
fn from_regex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Regex>, D::Error> {
    let res: Option<String> = Deserialize::deserialize(deserializer)?;
    res.map_or(Ok(None), |s| {
        Regex::new(&s).map_or(Ok(None), |re| Ok(Some(re)))
    })
}

fn to_config_string<S: Serializer>(wc: &Option<Regex>, s: S) -> Result<S::Ok, S::Error> {
    match wc {
        Some(ref re) => s.serialize_some(re.as_str()),
        None => s.serialize_none(),
    }
}

fn get_return_pipe() -> Result<File, Box<dyn std::error::Error>> {
    let file_name = ReturnPipe::pipe_name();
    let file_path = BaseDirectories::with_prefix("leftwm")?;
    let file_path = file_path
        .find_runtime_file(file_name)
        .ok_or("Unable to open return pipe")?;
    Ok(OpenOptions::new().append(true).open(file_path)?)
}

fn write_to_pipe(return_pipe: &mut Result<File, Box<dyn Error>>, msg: &str) {
    if let Ok(pipefile) = return_pipe {
        if let Err(e) = writeln!(pipefile, "{msg}") {
            tracing::warn!("Unable to connect to return pipe: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_serializes_to_valid_ron_test() {
        let config = Config::default();

        // Check RON
        let ron_pretty_conf = ron::ser::PrettyConfig::new()
            .depth_limit(2)
            .extensions(ron::extensions::Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        let ron = ron::ser::to_string_pretty(&config, ron_pretty_conf);
        assert!(ron.is_ok(), "Could not serialize default config");

        let ron_config = ron::from_str::<'_, Config>(ron.unwrap().as_str());
        assert!(ron_config.is_ok(), "Could not deserialize default config");
    }
}
