use leftwm::config::ThemeSetting;
use leftwm::errors::Result;
use leftwm::models::Margins;
use std::fs;
use std::path::Path;

pub struct ThemeLoader;

impl leftwm::config::ThemeLoader for ThemeLoader {
    fn load(&self, path: &Path) -> ThemeSetting {
        load_theme_file(path).unwrap_or_else(|_| self.default())
    }

    fn default(&self) -> ThemeSetting {
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
}

fn load_theme_file(path: &Path) -> Result<ThemeSetting> {
    let contents = fs::read_to_string(path)?;
    let from_file: ThemeSetting = toml::from_str(&contents)?;
    Ok(from_file)
}
