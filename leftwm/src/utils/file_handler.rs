use crate::{Config, ThemeSetting};
use anyhow::{self, Result};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
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
enum FileType {
    RonFile,
    TomlFile,
}

// TODO: wirte unified `fn load_file` that loads file with path and returns Type `Result<Config>` or `Result<ThemeSetting>`

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
    let config_file = path.place_config_file("config.*")?;
    let file_type = check_file_type(&config_file);

    // the checks and fallback for `toml` can be removed when toml gets eventually deprecated
    // let config_file_ron = path.place_config_file("config.ron")?;
    // let config_file_toml = path.place_config_file("config.toml")?;

    if Path::new(&config_file).exists() && file_type == FileType::RonFile {
        tracing::debug!("Config file '{}' found.", config_file.to_string_lossy());
        let ron = Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let contents = fs::read_to_string(config_file)?;
        let config = ron.from_str(&contents)?;
        Ok(config)
    } else if Path::new(&config_file).exists() && file_type == FileType::TomlFile {
        tracing::debug!("Config file '{}' found.", config_file.to_string_lossy());
        let contents = fs::read_to_string(config_file)?;
        let config = toml::from_str(&contents)?;
        tracing::info!("You are using TOML as config language which will be deprecated in the future.\nPlease consider migrating you config to RON. For further info visit the leftwm wiki.");
        Ok(config)
    } else {
        tracing::debug!("Config file not found. Using default config file.");

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
pub fn load_theme_file(path: impl AsRef<Path>) -> Result<ThemeSetting> {
    let contents = fs::read_to_string(&path)?;
    if path.as_ref().extension() == Some(std::ffi::OsStr::new("ron")) {
        let ron = Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let from_file: ThemeSetting = ron.from_str(&contents)?;
        Ok(from_file)
    } else {
        let from_file: ThemeSetting = toml::from_str(&contents)?;
        Ok(from_file)
    }
}

/// # Errors
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

fn check_file_type(path: impl AsRef<Path>) -> FileType {
    if path.as_ref().extension() == Some(std::ffi::OsStr::new("ron")) {
        FileType::RonFile
    } else {
        FileType::TomlFile
    }
}
