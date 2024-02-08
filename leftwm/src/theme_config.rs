use anyhow::Result;
use leftwm_core::models::{Gutter, Margins};
use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ThemeConfig {
    pub border_width: Option<i32>,
    pub margin: Option<CustomMargins>,
    pub workspace_margin: Option<CustomMargins>,
    pub default_width: Option<i32>,
    pub default_height: Option<i32>,
    pub always_float: Option<bool>,
    pub gutter: Option<Vec<Gutter>>,
    pub default_border_color: Option<String>,
    pub floating_border_color: Option<String>,
    pub focused_border_color: Option<String>,
    pub background_color: Option<String>,
    #[serde(rename = "on_new_window")]
    pub on_new_window_cmd: Option<String>,
}

impl ThemeConfig {
    pub fn load(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        match load_theme_file(path) {
            Ok(theme) => *self = theme,
            Err(err) => {
                tracing::error!("Could not load theme at path {}: {}", path.display(), err);
            }
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            border_width: Some(1),
            margin: Some(CustomMargins::Int(10)),
            workspace_margin: Some(CustomMargins::Int(10)),
            default_width: Some(1000),
            default_height: Some(700),
            always_float: Some(false),
            gutter: None,
            default_border_color: Some("#000000".to_owned()),
            floating_border_color: Some("#000000".to_owned()),
            focused_border_color: Some("#FF0000".to_owned()),
            background_color: Some("#333333".to_owned()),
            on_new_window_cmd: None,
        }
    }
}

fn load_theme_file(path: impl AsRef<Path>) -> Result<ThemeConfig> {
    let contents = fs::read_to_string(&path)?;
    if path.as_ref().extension() == Some(std::ffi::OsStr::new("ron")) {
        let ron = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        let from_file: ThemeConfig = ron.from_str(&contents)?;
        Ok(from_file)
    } else {
        let from_file: ThemeConfig = toml::from_str(&contents)?;
        Ok(from_file)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum CustomMargins {
    Int(u32),
    // format: [top, right, bottom, left] as per HTML
    Vec(Vec<u32>),
}

impl std::convert::TryFrom<CustomMargins> for Margins {
    type Error = &'static str;

    fn try_from(c: CustomMargins) -> Result<Self, Self::Error> {
        match c {
            CustomMargins::Int(size) => Ok(Self::new(size)),
            CustomMargins::Vec(vec) => match vec.len() {
                1 => Ok(Self::new(vec[0])),
                2 => Ok(Self::new_from_pair(vec[0], vec[1])),
                3 => Ok(Self::new_from_triple(vec[0], vec[1], vec[2])),
                4 => Ok(Self {
                    top: vec[0],
                    right: vec[1],
                    bottom: vec[2],
                    left: vec[3],
                }),
                0 => Err("Empty margin or border array"),
                _ => Err("Too many entries in margin or border array"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leftwm_core::models::Side;

    #[test]
    fn deserialize_custom_theme_config_toml() {
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
background_color = '#333333'
on_new_window = 'echo Hello World'

[[gutter]]
side = "Top"
value = 0
"#;
        let config: ThemeConfig = toml::from_str(config).unwrap();

        assert_eq!(
            config,
            ThemeConfig {
                border_width: Some(0),
                margin: Some(CustomMargins::Int(5)),
                workspace_margin: Some(CustomMargins::Int(5)),
                default_width: Some(400),
                default_height: Some(400),
                always_float: Some(true),
                gutter: Some(vec![Gutter {
                    side: Side::Top,
                    value: 0,
                    id: None
                }]),
                default_border_color: Some("#222222".to_string()),
                floating_border_color: Some("#005500".to_string()),
                focused_border_color: Some("#FFB53A".to_string()),
                background_color: Some("#333333".to_owned()),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }

    #[test]
    fn deserialize_custom_theme_config_ron() {
        let config = r##"
(
    border_width: Some(0),
    default_width: Some(400),
    default_height: Some(400),
    always_float: Some(true),
    margin: Some(5),
    workspace_margin: Some(5),
    default_border_color: Some("#222222"),
    floating_border_color: Some("#005500"),
    focused_border_color: Some("#FFB53A"),
    background_color: Some("#333333"),
    on_new_window: Some("echo Hello World"),

    gutter: Some([Gutter (
        side: Top,
        value: 0,
        )]
    )
)"##;
        let config: ThemeConfig = ron::from_str(config).unwrap();

        assert_eq!(
            config,
            ThemeConfig {
                border_width: Some(0),
                margin: Some(CustomMargins::Int(5)),
                workspace_margin: Some(CustomMargins::Int(5)),
                default_width: Some(400),
                default_height: Some(400),
                always_float: Some(true),
                gutter: Some(vec![Gutter {
                    side: Side::Top,
                    value: 0,
                    id: None,
                }]),
                default_border_color: Some("#222222".to_string()),
                floating_border_color: Some("#005500".to_string()),
                focused_border_color: Some("#FFB53A".to_string()),
                background_color: Some("#333333".to_owned()),
                on_new_window_cmd: Some("echo Hello World".to_string()),
            }
        );
    }
}
