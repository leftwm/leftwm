//! Theme configuration for `LeftWM`
#[cfg(feature = "unstable")]
use crate::config::Task;
use crate::errors::Result;
use crate::models::Gutter;
use crate::models::Margins;
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable")]
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::Path;

/// Holds theme settings commonly pertinent for both `leftwm-theme` and `leftwm`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct GlobalTheme {
    /// The commit hash for a git or mercurial theme
    commit: Option<String>,
    /// A vector of strings containing the dependency commands for `leftwm-theme` to find.
    /// Runs on apply, upgrade.
    dependencies: Option<Vec<String>>,
    /// A descriptive statement of the theme, such as "An exciting theme inspired by sunflowers."
    description: Option<String>,
    /// String of compatible `leftwm` versions. Follows SEMVER. See
    /// https://docs.rs/semver/0.11.0/semver/
    leftwm_versions: Option<String>,
    /// The name of the theme. This *must* be included.
    name: String,
    /// The location of the repository of the theme. Git preffered. Ex:
    /// https://github.com/lex148/leftwm-tng (can have .git on end).
    repository: Option<String>,
    /// Semantic version of the theme.
    version: Option<String>,
}

impl Default for GlobalTheme {
    fn default() -> Self {
        Self {
            commit: None,
            dependencies: None,
            description: Some("Unknown LeftWM theme".to_owned()),
            leftwm_versions: None,
            name: "unknown-leftwm-theme".to_owned(),
            repository: None,
            version: None,
        }
    }
}

/// Holds theme settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
#[cfg(not(feature = "unstable"))]
pub struct ThemeSetting {
    /// Border width for each window
    pub border_width: i32,
    /// Margins on the edges of the screen/between windows. Uses [top right bottom left] or
    /// [top/bottom right/left] like HTML.
    pub margin: Margins,
    pub workspace_margin: Margins,
    pub gutter: Option<Vec<Gutter>>,
    /// The border color around non-floating, non-focused windows. Uses HEX (e.g. #500000).
    pub default_border_color: String,
    /// The border color around floating windows. Uses HEX (e.g. #500000).
    pub floating_border_color: String,
    /// The border color around focused windows. Uses HEX (e.g. #500000).
    pub focused_border_color: String,
    /// Commands to run when new windows are created
    #[serde(rename = "on_new_window")]
    pub on_new_window_cmd: Option<String>,
}

/// Holds theme settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
#[cfg(feature = "unstable")]
pub struct ThemeSetting {
    /// Border width for each window
    pub border_width: i32,
    /// Margins on the edges of the screen/between windows. Uses [top right bottom left] or
    /// [top/bottom right/left] like HTML.
    pub window_margin: Margins,
    /// Margin around workspace:
    pub workspace_margin: Margins,
    /// Gutter around workspace
    pub gutter: Option<Vec<Gutter>>,
    /// The border color around non-floating, non-focused windows. Uses HEX (e.g. #500000).
    pub default_border_color: String,
    /// The border color around floating windows. Uses HEX (e.g. #500000).
    pub floating_border_color: String,
    /// The border color around focused windows. Uses HEX (e.g. #500000).
    pub focused_border_color: String,
    /// Commands to run when new windows are created
    #[serde(rename = "on_new_window")]
    pub on_new_window_cmd: Option<String>,
    /// Global configuration
    pub global: Option<GlobalTheme>,
    /// Tasks to run when leftwm-worker (re)starts or stops.
    pub task: Option<HashMap<String, Task>>,
}

impl ThemeSetting {
    #[must_use]
    pub fn load(path: &Path) -> ThemeSetting {
        load_theme_file(path).unwrap_or_default()
    }

    #[must_use]
    pub fn get_list_of_gutters(&self) -> Vec<Gutter> {
        if let Some(gutters) = &self.gutter {
            return gutters.clone();
        }
        vec![]
    }
}

fn load_theme_file(path: &Path) -> Result<ThemeSetting> {
    let contents = fs::read_to_string(path)?;
    let from_file: ThemeSetting = toml::from_str(&contents)?;
    Ok(from_file)
}

impl Default for ThemeSetting {
    #[cfg(not(feature = "unstable"))]
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            margin: Margins::Int(10),
            workspace_margin: Margins::Int(10),
            gutter: None,
            default_border_color: "#000000".to_owned(),
            floating_border_color: "#000000".to_owned(),
            focused_border_color: "#FF0000".to_owned(),
            on_new_window_cmd: None,
        }
    }

    #[cfg(feature = "unstable")]
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            window_margin: Margins::Int(10),
            workspace_margin: Margins::Int(10),
            gutter: None,
            default_border_color: "#000000".to_owned(),
            floating_border_color: "#000000".to_owned(),
            focused_border_color: "#FF0000".to_owned(),
            on_new_window_cmd: None,
            global: None,
            task: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Component;
    use crate::config::Task;
    use crate::models::Side;

    #[test]
    fn deserialize_empty_theme_config() {
        let config = "";
        let config: ThemeSetting = toml::from_str(config).unwrap();
        let default_config = ThemeSetting::default();

        assert_eq!(config, default_config);
    }

    #[test]
    fn deserialize_custom_theme_config_for_older_themes() {
        let config = r#"
border_width = 0
margin = 5
workspace_margin = 5
window_margin = 5
default_border_color = '#222222'
floating_border_color = '#005500'
focused_border_color = '#FFB53A'
on_new_window = 'echo Hello World'

[[gutter]]
side = "Top"
value = 0
"#;
        let config: ThemeSetting = toml::from_str(config).unwrap();

        #[cfg(feature = "unstable")]
        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                window_margin: Margins::Int(5),
                workspace_margin: Margins::Int(5),
                gutter: Some(vec![Gutter {
                    side: Side::Top,
                    value: 0
                }]),
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
                global: None,
                task: None,
            }
        );

        #[cfg(not(feature = "unstable"))]
        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                margin: Margins::Int(5),
                workspace_margin: Margins::Int(5),
                gutter: Some(vec![Gutter {
                    side: Side::Top,
                    value: 0
                }]),
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }

    #[test]
    fn deserialize_custom_theme_config() {
        let config = r###"
                    border_width = 0
                    window_margin = 5
                    default_border_color = '#222222'
                    floating_border_color = '#005500'
                    focused_border_color = '#FFB53A'
                    on_new_window = 'echo Hello World'
                    [global]
                    name = "mytheme"
                    description = "A basic theme"
                    repository = "https://github.com/mautamu/theme"
                    commit = "*" # pin at a specific version as a "release"
                    version = "1.0.0"
                    leftwm_versions = ">2.6.7" # only install on compatible leftwms
                    dependencies = ["polybar", "dunst"]  # Used to check if required packages are installed in the system
                    [task.polybar.up]
                    command = "polybar"
                    args = ["mybar"]
                    "###;
        let config: ThemeSetting = toml::from_str(config).unwrap();

        let mut task_hmap = std::collections::HashMap::new();
        task_hmap.insert(
            "polybar".to_string(),
            Task {
                up: Some(Component {
                    command: "polybar".to_string(),
                    args: Some(vec!["mybar".to_string()]),
                    group: None,
                }),
                down: None,
                install: None,
            },
        );
        #[cfg(feature = "unstable")]
        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                window_margin: Margins::Int(5),
                workspace_margin: Margins::Int(10),
                gutter: None,
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
                global: Some(GlobalTheme {
                    commit: Some("*".to_string()),
                    dependencies: Some(vec!["polybar".to_string(), "dunst".to_string()]),
                    description: Some("A basic theme".to_string()),
                    leftwm_versions: Some(">2.6.7".to_string()),
                    name: "mytheme".to_string(),
                    repository: Some("https://github.com/mautamu/theme".to_string()),
                    version: Some("1.0.0".to_string()),
                }),
                task: Some(task_hmap),
            }
        );

        #[cfg(not(feature = "unstable"))]
        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                margin: Margins::Int(10),
                workspace_margin: Margins::Int(10),
                gutter: None,
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }
}
