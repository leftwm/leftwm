mod common;

use clap::{App, Arg};
use leftwm::config::{Keybind, Workspace};
use leftwm::errors::Result;
use leftwm::utils;
use leftwm::Command;
use std::fs;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

use common::config::Config;

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

    println!(
        "\x1b[0;94m::\x1b[0m LeftWM version: {}",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "\x1b[0;94m::\x1b[0m LeftWM git hash: {}",
        git_version::git_version!(fallback = "NONE")
    );
    println!("\x1b[0;94m::\x1b[0m Loading configuration . . .");
    match load_from_file(config_file, verbose) {
        Ok(config) => {
            println!("\x1b[0;92m    -> Configuration loaded OK \x1b[0m");
            if config == Config::default() {
                println!("\x1b[1;32mWARN: The file loaded was the default. Your configuration is likely invalid \x1b[0m");
            }
            if verbose {
                dbg!(&config);
            }
            check_workspace_ids(config.workspaces, verbose);
            check_keybinds(config.keybind, verbose);
        }
        Err(e) => {
            println!("Configuration failed. Reason: {:?}", e);
        }
    }
    println!("\x1b[0;94m::\x1b[0m Checking environment . . .");
    check_elogind(verbose)?;

    Ok(())
}

/// Loads configuration from either specified file (preferred) or default.
/// # Errors
///
/// Errors if file cannot be read. Indicates filesystem error
/// (inadequate permissions, disk full, etc.)
/// If a path is specified and does not exist, returns `LeftError`.
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

/// Checks defined workspaces to ensure no ID collisions occur.
fn check_workspace_ids(workspaces: Option<Vec<Workspace>>, verbose: bool) -> bool {
    workspaces.map_or(true, |wss|
    {
        if verbose {
            println!("Checking config for valid workspace definitions.");
        }
        let ids = common::config::get_workspace_ids(&wss);
        if ids.iter().any(|id| id.is_some()) {
            if !common::config::all_ids_some(&ids)
            {
                println!("Your config.toml specifies an ID for some but not all workspaces. This can lead to ID collisions and is not allowed. The default config will be used instead.");
                false
            } else if !common::config::all_ids_unique(&ids) {
                println!("Your config.toml contains duplicate workspace IDs. Please assign unique IDs to workspaces. The default config will be used instead.");
                false
            } else {
                true
            }
        } else {
            true
        }
    }
    )
}

/// Check all keybinds to ensure that required values are provided
/// Checks to see if value is provided (if required)
/// Checks to see if keys are valid against Xkeysym
/// Ideally, we will pass this to the command handler with a dummy config
fn check_keybinds(keybinds: Vec<Keybind>, verbose: bool) -> bool {
    let mut returns = Vec::new();
    let value_required_commands = vec![
        Command::ToggleScratchPad,
        Command::MoveToTag,
        Command::GotoTag,
        Command::Execute,
        Command::IncreaseMainWidth,
        Command::DecreaseMainWidth,
        Command::SetLayout,
        Command::SetMarginMultiplier,
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

fn check_elogind(verbose: bool) -> Result<()> {
    // We assume that if it is in the path it's all good
    // We also cross-reference the ENV variable
    match (
        std::env::var("XDG_RUNTIME_DIR"),
        common::config::is_program_in_path("loginctl"),
    ) {
        (Ok(val), true) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {}, LOGINCTL OKAY", val);
            }

            println!("\x1b[0;92m    -> Environment OK \x1b[0;92m");

            Ok(())
        }
        (Ok(val), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {}, LOGINCTL not installed", val);
            }

            println!("\x1b[0;92m    -> Environment OK (has XDG_RUNTIME_DIR) \x1b[0;92m");

            Ok(())
        }
        (Err(e), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR_ERROR: {:?}, LOGINCTL BAD", e);
            }
            println!("\x1b[1;91mERROR: XDG_RUNTIME_DIR not set and elogind not found.\nSee https://github.com/leftwm/leftwm/wiki/XDG_RUNTIME_DIR for more information.\x1b[0m",);

            Err(leftwm::errors::LeftError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Elogind not installed/operating and no alternative XDG_RUNTIME_DIR is set.",
            )))
        }
        (Err(e), true) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {:?}, LOGINCTL OKAY", e);
            }
            println!(
                "\x1b[1;33mWARN: Elogind/systemd installed but XDG_RUNTIME_DIR not set.\nThis may be because elogind isn't started. \x1b[0m",
            );
            Ok(())
        }
    }
}
