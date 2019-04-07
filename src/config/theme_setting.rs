use std::default::Default;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ThemeSettingLoadable {
    border_width: Option<u32>,
    margin: Option<u32>,
    default_border_color: Option<String>,
    floating_border_color: Option<String>,
    focused_border_color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ThemeSetting {
    pub border_width: u32,
    pub margin: u32,
    pub default_border_color: String,
    pub floating_border_color: String,
    pub focused_border_color: String,
}

/// Convert Theme Setting Loadable into Theme Settings
/// This will only override fields that have been provided
impl From<ThemeSettingLoadable> for ThemeSetting {
    fn from(file: ThemeSettingLoadable) -> Self {
        let mut theme = ThemeSetting::default();
        if let Some(x) = file.border_width {
            theme.border_width = x
        }
        if let Some(x) = file.margin {
            theme.margin = x
        }
        if let Some(x) = file.default_border_color {
            theme.default_border_color = x
        }
        if let Some(x) = file.floating_border_color {
            theme.floating_border_color = x
        }
        if let Some(x) = file.focused_border_color {
            theme.focused_border_color = x
        }
        theme
    }
}

impl ThemeSetting {
    pub fn load(path: &PathBuf) -> ThemeSetting {
        let mut theme = ThemeSetting::default();
        if let Ok(file) = load_theme_file(path) {
            theme = file.into();
        }
        theme
    }
}

fn load_theme_file(path: &PathBuf) -> Result<ThemeSettingLoadable, Box<std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let from_file: ThemeSettingLoadable = toml::from_str(&contents)?;
    Ok(from_file)
}

impl Default for ThemeSetting {
    fn default() -> Self {
        ThemeSetting {
            border_width: 1,
            margin: 10,
            default_border_color: "#000000".to_owned(),
            floating_border_color: "#000000".to_owned(),
            focused_border_color: "#FF0000".to_owned(),
        }
    }
}
