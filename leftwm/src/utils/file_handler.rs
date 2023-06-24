use crate::{Config, ThemeSetting};
use anyhow::{self, Result};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use serde::de::DeserializeOwned;
use std::{
    ffi::OsStr,
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
pub enum ConfigFileType {
    RonFile,
    TomlFile,
}

/// # Panics
///
/// Function can only panic if ron cannot be serialized. This should not occur as it is defined
/// globally.
///
/// # Errors
///
/// Function will throw an error if `BaseDirectories` doesn't exist, if user doesn't have
/// permissions to place config.ron, if config.ron or config.toml cannot be read (access writes, malformed file,
/// etc.).
/// Function can also error from inability to save config.toml (if it is the first time running
/// `LeftWM).
pub fn load_config_file(config_filename: Option<PathBuf>) -> Result<Config> {
    tracing::debug!("Loading config file");
    let config_file = match check_path(config_filename, false) {
        Ok(path) => path,
        Err(_) => {
            tracing::warn!("Config file not found. Creating default config file.");
            let config = Config::default();
            let file_path = get_default_path()?;
            write_to_file(&file_path, &config)?;
            file_path
        }
    };

    read_ron_config(&config_file)
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns LeftError`.
pub fn load_theme_file(path: &PathBuf) -> Result<ThemeSetting> {
    if check_file_type(path) == ConfigFileType::RonFile {
        read_ron_config(path)
    } else {
        tracing::error!("`TOML` config and theming is deprecatd, please convert to `RON`. Refer to the leftwm wiki for more info.");
        Err(leftwm_core::errors::LeftError::TomlDeprecationError.into())
    }
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
fn read_ron_config<T: DeserializeOwned>(config_file: &PathBuf) -> Result<T, anyhow::Error> {
    let ron = Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES);
    let contents = fs::read_to_string(config_file)?;
    Ok(ron.from_str(&contents)?)
}

/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
fn read_toml_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T, anyhow::Error> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

/// # Errors
///
/// This function errors when:
/// - serialization of the config fails
/// - writing to file fails
fn write_to_file(ron_file: &PathBuf, config: &Config) -> Result<(), anyhow::Error> {
    let ron_pretty_conf = PrettyConfig::new()
        .depth_limit(2)
        .extensions(Extensions::IMPLICIT_SOME)
        .extensions(Extensions::UNWRAP_NEWTYPES);
    let ron = to_string_pretty(&config, ron_pretty_conf)?;
    let ron_with_header = String::from(COMMENT_HEADER) + &ron;
    let mut file = File::create(ron_file)?;
    file.write_all(ron_with_header.as_bytes())?;
    Ok(())
}

pub fn check_file_type(path: impl AsRef<Path>) -> ConfigFileType {
    // we want to assume any other file type as `toml` is `ron` so files like `config.backup` can be checked manually and get parsed as ron
    match path.as_ref().extension() {
        Some(e) if e == OsStr::new("toml") => {
            tracing::warn!("You are using TOML as config language which will be deprecated in the future.\nPlease consider migrating you config to RON. For further info visit the leftwm wiki.");
            ConfigFileType::TomlFile
        }
        Some(e) if e == OsStr::new("ron") => ConfigFileType::RonFile,
        _ => {
            tracing::info!("Non-matching file type, assuming `RON`.");
            ConfigFileType::RonFile
        }
    }
}

pub fn check_path(
    config_filename: Option<PathBuf>,
    verbose: bool,
) -> Result<PathBuf, anyhow::Error> {
    let file_path = match config_filename {
        Some(c) => {
            if verbose {
                println!(
                    "\x1b[1;35mNote: Using file {} \x1b[0m",
                    &c.to_string_lossy()
                );
            }
            c.clone()
        }
        None => get_default_path()?,
    };
    if Path::new(&file_path).exists() {
        tracing::debug!("Config file '{}' found.", file_path.to_string_lossy());
        Ok(file_path)
    } else {
        Err(anyhow::Error::msg("file not found"))
    }
}

pub fn get_default_path() -> Result<PathBuf, anyhow::Error> {
    let config_path = BaseDirectories::with_prefix("leftwm")?;
    Ok(config_path.place_config_file("config.ron")?)
}

pub fn migrate_config(path: PathBuf, verbose: bool) -> Result<(), anyhow::Error> {
    if verbose {
        println!("Using `TOML` file with path: {}", path.to_string_lossy());
    }
    let toml_config = read_toml_file(&path)?;
    let ron_file = get_default_path()?;
    write_to_file(&ron_file, &toml_config)
}
