use anyhow::Result;
use leftwm::models::{Gutter, Margins};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ThemeSetting {
    pub border_width: i32,
    pub margin: CustomMargins,
    pub workspace_margin: Option<CustomMargins>,
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
            margin: CustomMargins::Int(10),
            workspace_margin: Some(CustomMargins::Int(10)),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
            CustomMargins::Int(size) => Ok(Self {
                top: size,
                right: size,
                bottom: size,
                left: size,
            }),
            CustomMargins::Vec(vec) => {
                match vec.len() {
                    1 => Ok(Self::new(vec[0])),
                    2 => Ok(Self {
                        top: *vec[0],
                        right: *vec[0],
                        bottom: *vec[1],
                        left: *vec[1],
                    }),
                    3 => Ok(Self {
                        top: *vec[0],
                        right: *vec[1],
                        bottom: *vec[2],
                        left: *vec[2],
                    }),
                    4 => Ok(Self {
                        top: *vec[0],
                        right: *vec[1],
                        bottom: *vec[2],
                        left: *vec[3],
                    }),
                    0 => {
                        log::error!("Empty margin or border array");
                        Self::new(10) //assume 10 px borders for now
                    }
                    _ => {
                        log::error!("Too many entries in margin or border array");
                        Ok(Self {
                            top: *vec[0],
                            right: *vec[1],
                            bottom: *vec[2],
                            left: *vec[3],
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leftwm::models::Side;

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
                margin: CustomMargins::Int(5),
                workspace_margin: Some(CustomMargins::Int(5)),
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
