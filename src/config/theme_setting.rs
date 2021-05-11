use crate::errors::Result;
use crate::models::Margins;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct ThemeSetting {
    pub border_width: i32,
    pub window_margin: Margins,
    pub workspace_margin: Margins,
    pub gutter: i32,
    pub default_border_color: String,
    pub floating_border_color: String,
    pub focused_border_color: String,
    #[serde(rename = "on_new_window")]
    pub on_new_window_cmd: Option<String>,
}

impl ThemeSetting {
    #[must_use]
    pub fn load(path: &Path) -> ThemeSetting {
        load_theme_file(path).unwrap_or_default()
    }
}

fn load_theme_file(path: &Path) -> Result<ThemeSetting> {
    let contents = fs::read_to_string(path)?;
    let from_file: ThemeSetting = toml::from_str(&contents)?;
    Ok(from_file)
}

impl Default for ThemeSetting {
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            window_margin: Margins::Int(10),
            workspace_margin: Margins::Int(10),
            gutter: 0,
            default_border_color: "#000000".to_owned(),
            floating_border_color: "#000000".to_owned(),
            focused_border_color: "#FF0000".to_owned(),
            on_new_window_cmd: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_theme_config() {
        let config = "";
        let config: ThemeSetting = toml::from_str(config).unwrap();
        let default_config = ThemeSetting::default();

        assert_eq!(config, default_config);
    }

    #[test]
    fn deserialize_custom_theme_config() {
        let config = r#"
border_width = 0
window_margin = 5
workspace_margin = 5
gutter = 0
default_border_color = '#222222'
floating_border_color = '#005500'
focused_border_color = '#FFB53A'
on_new_window = 'echo Hello World'
"#;
        let config: ThemeSetting = toml::from_str(config).unwrap();

        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                window_margin: Margins::Int(5),
                workspace_margin: Margins::Int(5),
                gutter: 0,
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }
}
