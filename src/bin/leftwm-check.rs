use clap::{App, Arg};
use leftwm::config::Config;
use leftwm::config::Keybind;
use leftwm::errors::Result;
use leftwm::utils;
use leftwm::Command;
use std::fs;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM Check")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("checks syntax of the configuration file")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use. Uses first in PATH otherwise.")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Outputs received configuration file."),
        )
        .get_matches();

    let config_file = matches.value_of("INPUT");
    let verbose = matches.occurrences_of("verbose") >= 1;

    //    dbg!(config_file);
    //    use leftwm::config::*;

    pub fn load_from_file(fspath: Option<&str>, verbose: bool) -> Result<Config> {
        let config_filename = match fspath {
            Some(fspath) => {
                println!("\x1b[1;35mNote: Using file {} \x1b[0m", fspath);
                PathBuf::from(fspath)
            }

            None => BaseDirectories::with_prefix("leftwm")?.place_config_file("config.toml")?,
        };
        if verbose {
            dbg!(&config_filename);
        }
        if Path::new(&config_filename).exists() {
            let contents = fs::read_to_string(config_filename)?;
            if verbose {
                dbg!(&contents);
            }
            Ok(toml::from_str(&contents)?)
        } else {
            Err(leftwm::errors::LeftError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Configuration not found in path",
            )))
        }
    }

    println!("\x1b[0;94m::\x1b[0m Loading configuration . . .");
    match load_from_file(config_file, verbose) {
        Ok(config) => {
            println!("\x1b[0;92m    -> Configuration loaded OK \x1b[0;92m");
            if config == Config::default() {
                println!("\x1b[1;32mWARNING: The file loaded was the default. Your configuration is likely invalid \x1b[0m");
            }
            if verbose {
                dbg!(&config);
            }
            check_keybinds(config.keybind, verbose);
        }
        Err(e) => {
            println!("Configuration failed. Reason: {:?}", e);
        }
    }

    Ok(())
}

fn check_keybinds(keybinds: Vec<Keybind>, verbose: bool) -> bool {
    let mut returns = Vec::new();
    let value_required_commands = vec![
        Command::MoveToTag,
        Command::GotoTag,
        Command::Execute,
        Command::IncreaseMainWidth,
        Command::DecreaseMainWidth,
        Command::SetLayout,
    ];
    println!("\x1b[0;94m::\x1b[0m Checking keybinds . . .");
    for keybind in keybinds {
        if verbose {
            println!("Keybind: {:?} {}", keybind, keybind.value.is_none());
        }
        if value_required_commands
            .iter()
            .any(|i| i.clone() == keybind.command)
            && keybind.value.is_none()
        {
            returns.push((
                keybind.clone(),
                "This keybind requres a `string`".to_owned(),
            ));
        }
        if utils::xkeysym_lookup::into_keysym(&keybind.key).is_none() {
            returns.push((
                keybind.clone(),
                format!("Key `{}` is not valid", keybind.key),
            ));
        }
    }
    if returns.is_empty() {
        println!("\x1b[0;92m    -> All keybinds OK\x1b[0m");
        true
    } else {
        for error in returns {
            println!(
                "\x1b[1;91mERROR: {} for keybind {:?}\x1b[0m",
                error.1, error.0
            );
        }
        false
    }
}
