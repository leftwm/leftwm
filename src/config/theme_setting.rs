use crate::models::Gutter;
use crate::models::Margins;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait ThemeLoader {
    fn load(&self, path: &Path) -> ThemeSetting;
    fn default(&self) -> ThemeSetting;
}

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
    #[must_use]
    pub fn get_list_of_gutters(&self) -> Vec<Gutter> {
        if let Some(gutters) = &self.gutter {
            return gutters.clone();
        }
        vec![]
    }
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
