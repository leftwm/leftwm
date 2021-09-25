use anyhow::Result;
use leftwm::models::{Gutter, Margins};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ThemeSetting {
    pub border_width: i32,
    pub margin: Margins,
    pub workspace_margin: Option<Margins>,
    pub gutter: Option<Vec<Gutter>>,
    pub default_border_color: String,
    pub floating_border_color: String,
    pub focused_border_color: String,
    #[serde(rename = "on_new_window")]
    pub on_new_window_cmd: Option<String>,
}

impl ThemeSetting {
    pub fn load(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        match load_theme_file(path) {
            Ok(theme) => *self = theme,
            Err(err) => {
                log::error!("Could not load theme at path {}: {}", path.display(), err);
            }
        }
    }
}

impl Default for ThemeSetting {
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            margin: Margins::new(10),
            workspace_margin: Some(Margins::new(10)),
            gutter: None,
            default_border_color: "#000000".to_owned(),
            floating_border_color: "#000000".to_owned(),
            focused_border_color: "#FF0000".to_owned(),
            on_new_window_cmd: None,
        }
    }
}

fn load_theme_file(path: impl AsRef<Path>) -> Result<ThemeSetting> {
    let contents = fs::read_to_string(path)?;
    let from_file: ThemeSetting = toml::from_str(&contents)?;
    Ok(from_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Side;

    #[test]
    fn deserialize_custom_theme_config() {
        let config = r#"
border_width = 0
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
                margin: Margins::Int(5),
                workspace_margin: Some(Margins::Int(5)),
                gutter: Some(vec![Gutter {
                    side: Side::Top,
                    value: 0,
                    wsid: None,
                }]),
                default_border_color: "#222222".to_string(),
                floating_border_color: "#005500".to_string(),
                focused_border_color: "#FFB53A".to_string(),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }
}
