use super::display_servers::DisplayServer;
use super::manager::Manager;
use super::utils;
use std::fs;
use toml;
use xdg;
mod config_structs;
pub use config_structs::*;

pub fn apply_config<T: DisplayServer>(manager: &mut Manager<T>) {
    // default to tags 1 to 9
    (1..10)
        .iter()
        .for_each(|i| manager.tags.push(i.to_string()));
}

pub fn parse_config() -> Config {
    let path = config_path();
    let config_contents = fs::read_to_string(path).expect("Something went wrong reading the file");
    let config = toml::from_str::<Config>(&config_contents);

    config.unwrap_or_default()
}

fn config_path() -> std::path::PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("whatawm").unwrap();

    let config_path = xdg_dirs
        .place_config_file("config.toml")
        .expect("cannot create configuration directory");

    if !config_path.exists() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        fs::write(&config_path, toml).expect("Unable to write config.toml file");
    }

    config_path
}
