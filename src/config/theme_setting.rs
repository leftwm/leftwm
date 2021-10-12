use crate::errors::Result;
use crate::models::Gutter;
use crate::models::Margins;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct ThemeSetting {
    pub border_width: i32,
    pub default_width: i32,
    pub default_height: i32,
    pub always_float: bool,
    pub margin: Margins,
    pub workspace_margin: Margins,
    pub gutter: Option<Vec<Gutter>>,
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
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            default_width: 1000,
            default_height: 800,
            always_float: false,
            margin: Margins::Int(10),
            workspace_margin: Margins::Int(10),
            gutter: None,
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
    use crate::models::Side;

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
default_width = 400
default_height = 400
always_float = true
margin = 5
workspace_margin = 5
default_border_color = '#222222'
floating_border_color = '#005500'
focused_border_color = '#FFB53A'
on_new_window = 'echo Hello World'

[[gutter]]
side = "Top"
value = 0
"#;
        let config: ThemeSetting = toml::from_str(config).unwrap();

        assert_eq!(
            config,
            ThemeSetting {
                border_width: 0,
                default_width: 400,
                default_height: 400,
                always_float: true,
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
}
