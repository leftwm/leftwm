use crate::{Config, ThemeSetting};
use anyhow::{self, Result};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use serde::de::DeserializeOwned;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;

const COMMENT_HEADER: &str = r#"//  _        ___                                      ___ _
// | |      / __)_                                   / __|_)
// | | ____| |__| |_ _ _ _ ____      ____ ___  ____ | |__ _  ____    ____ ___  ____
// | |/ _  )  __)  _) | | |    \    / ___) _ \|  _ \|  __) |/ _  |  / ___) _ \|  _ \
// | ( (/ /| |  | |_| | | | | | |  ( (__| |_| | | | | |  | ( ( | |_| |  | |_| | | | |
// |_|\____)_|   \___)____|_|_|_|   \____)___/|_| |_|_|  |_|\_|| (_)_|   \___/|_| |_|
// A WindowManager for Adventurers                         (____/
// For info about configuration please visit https://github.com/leftwm/leftwm/wiki

"#;

#[derive(PartialEq)]
enum ConfigFileType {
    RonFile,
    TomlFile,
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
pub fn load_config_file(_config_filename: &Option<PathBuf>) -> Result<Config> {
    tracing::debug!("Loading config file");

    let path = BaseDirectories::with_prefix("leftwm")?;
    let config_file = path.place_config_file("config.ron")?;

    if Path::new(&config_file).exists() {
        tracing::debug!("Config file '{}' found.", config_file.to_string_lossy());
        read_ron_config(&config_file)
    } else {
        // Deprecated TOML handling
        let config_file_toml = path.get_config_file("config.toml");
        if Path::new(&config_file_toml).exists() {
            tracing::debug!(
                "Config file '{}' found.",
                config_file_toml.to_string_lossy()
            );
            let config = read_toml_file(&config_file)?;
            tracing::info!("You are using TOML as config language which will be deprecated in the future.\nPlease consider migrating you config to RON. For further info visit the leftwm wiki.");
            return Ok(config);
        }

        tracing::warn!("Config file not found. Creating default config file.");

        let config = Config::default();
        write_to_file(&config_file, &config)?;
        Ok(config)
    }
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
pub fn load_theme_file(path: &PathBuf) -> Result<ThemeSetting> {
    let file_type = check_file_type(path);
    if file_type == ConfigFileType::RonFile {
        read_ron_config(path)
    } else {
        read_toml_file(path)
    }
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
fn read_ron_config<T: DeserializeOwned>(config_file: &PathBuf) -> Result<T, anyhow::Error> {
    let ron = Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
    let contents = fs::read_to_string(config_file)?;
    let config = ron.from_str(&contents)?;
    Ok(config)
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
fn read_toml_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T, anyhow::Error> {
    let from_file = toml::from_str(&fs::read_to_string(path)?)?;
    Ok(from_file)
}

/// # Errors
///
/// This function errors when:
/// - serialization of the config fails
/// - writing to file fails
pub fn write_to_file(ron_file: &Path, config: &Config) -> Result<(), anyhow::Error> {
    let ron_pretty_conf = PrettyConfig::new()
        .depth_limit(2)
        .extensions(Extensions::IMPLICIT_SOME);
    let ron = to_string_pretty(&config, ron_pretty_conf)?;
    let ron_with_header = String::from(COMMENT_HEADER) + &ron;
    let mut file = File::create(ron_file)?;
    file.write_all(ron_with_header.as_bytes())?;
    Ok(())
}

fn check_file_type(path: impl AsRef<Path>) -> ConfigFileType {
    if path.as_ref().extension() == Some(std::ffi::OsStr::new("ron")) {
        ConfigFileType::RonFile
    } else {
        ConfigFileType::TomlFile
    }
}
