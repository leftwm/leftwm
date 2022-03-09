use anyhow::{bail, Result};
use clap::{App, Arg};
use leftwm::{Config, ThemeSetting};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
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

    println!(
        "\x1b[0;94m::\x1b[0m LeftWM version: {}",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "\x1b[0;94m::\x1b[0m LeftWM git hash: {}",
        git_version::git_version!(fallback = option_env!("GIT_HASH").unwrap_or("NONE"))
    );
    println!("\x1b[0;94m::\x1b[0m Loading configuration . . .");
    match load_from_file(config_file, verbose) {
        Ok(config) => {
            println!("\x1b[0;92m    -> Configuration loaded OK \x1b[0m");
            if verbose {
                dbg!(&config);
            }
            config.check_mousekey(verbose);
            config.check_workspace_ids(verbose);
            config.check_keybinds(verbose);
        }
        Err(e) => {
            println!("Configuration failed. Reason: {:?}", e);
        }
    }
    println!("\x1b[0;94m::\x1b[0m Checking environment . . .");
    check_elogind(verbose)?;
    println!("\x1b[0;94m::\x1b[0m Checking theme . . .");
    check_theme(verbose);

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

        let config = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let mut file = File::create(&config_filename)?;
        file.write_all(toml.as_bytes())?;
        Ok(config)
    }
}

fn check_elogind(verbose: bool) -> Result<()> {
    // We assume that if it is in the path it's all good
    // We also cross-reference the ENV variable
    match (
        std::env::var("XDG_RUNTIME_DIR"),
        leftwm::is_program_in_path("loginctl"),
    ) {
        (Ok(val), true) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {}, LOGINCTL OKAY", val);
            }

            println!("\x1b[0;92m    -> Environment OK \x1b[0m");

            Ok(())
        }
        (Ok(val), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {}, LOGINCTL not installed", val);
            }

            println!("\x1b[0;92m    -> Environment OK (has XDG_RUNTIME_DIR) \x1b[0m");

            Ok(())
        }
        (Err(e), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR_ERROR: {:?}, LOGINCTL BAD", e);
            }

            bail!(
                "Elogind not installed/operating and no alternative XDG_RUNTIME_DIR is set. \
                See https://github.com/leftwm/leftwm/wiki/XDG_RUNTIME_DIR for more information."
            );
        }
        (Err(e), true) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {:?}, LOGINCTL OKAY", e);
            }
            println!(
                "\x1b[1;93mWARN: Elogind/systemd installed but XDG_RUNTIME_DIR not set.\nThis may be because elogind isn't started. \x1b[0m",
            );
            Ok(())
        }
    }
}

/// Checks if `.config/leftwm/theme/current/` is a valid path
/// Checks if `up` and `down` scripts are in the `current` directory and have executable permission
/// Checks if `theme.toml` is in the `current` path
fn check_theme(verbose: bool) -> bool {
    let xdg_base_dir = BaseDirectories::with_prefix("leftwm/themes");
    let err_formatter = |s| println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {} \x1b[0m", s);

    if let Err(e) = xdg_base_dir {
        err_formatter(e.to_string());
        return false;
    }

    let xdg_base_dir = xdg_base_dir.unwrap();
    let path_current_theme = xdg_base_dir.find_config_file("current");

    match check_current_theme_set(&path_current_theme, verbose) {
        Ok(_) => check_theme_contents(xdg_base_dir.list_config_files("current"), verbose),
        Err(e) => {
            err_formatter(e.to_string());
            false
        }
    }
}

fn check_theme_contents(filepaths: Vec<PathBuf>, verbose: bool) -> bool {
    let mut returns = Vec::new();
    let missing_files = missing_expected_file(&filepaths);

    for missing_file in missing_files {
        returns.push(format!("File not found: {}", missing_file));
    }

    for filepath in filepaths {
        match filepath {
            f if f.ends_with("up") => match check_permissions(f, verbose) {
                Ok(fp) => match check_up_file(fp) {
                    Ok(_) => continue,
                    Err(e) => returns.push(e.to_string()),
                },
                Err(e) => returns.push(e.to_string()),
            },
            f if f.ends_with("down") => match check_permissions(f, verbose) {
                Ok(_fp) => continue,
                Err(e) => returns.push(e.to_string()),
            },
            f if f.ends_with("theme.toml") => match check_theme_toml(f, verbose) {
                Ok(_fp) => continue,
                Err(e) => returns.push(e.to_string()),
            },
            _ => continue,
        }
    }

    if returns.is_empty() {
        println!("\x1b[0;92m    -> Theme OK \x1b[0m");
        true
    } else {
        for error in &returns {
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {} \x1b[0m", error);
        }
        false
    }
}

fn missing_expected_file<'a>(filepaths: &'a [PathBuf]) -> impl Iterator<Item = &&'a str> {
    ["up", "down", "theme.toml"]
        .iter()
        .filter(move |f| !filepaths.iter().any(|fp| fp.ends_with(f)))
}

fn check_current_theme_set(filepath: &Option<PathBuf>, verbose: bool) -> Result<&PathBuf> {
    match &filepath {
        Some(p) => {
            if verbose {
                if fs::symlink_metadata(&p)?.file_type().is_symlink() {
                    println!(
                        "Found symlink `current`, pointing to theme folder: {:?}",
                        fs::read_link(&p).unwrap()
                    );
                } else {
                    println!("\x1b[1;93mWARN: Found `current` theme folder: {:?}. Use of a symlink is recommended, instead.\x1b[0m", p);
                }
            }
            Ok(p)
        }
        None => bail!("No theme folder or symlink `current` found."),
    }
}

fn check_permissions(filepath: PathBuf, verbose: bool) -> Result<PathBuf> {
    let metadata = fs::metadata(&filepath)?;
    let permissions = metadata.permissions();
    if metadata.is_file() && (permissions.mode() & 0o111 != 0) {
        if verbose {
            println!(
                "Found `{}` with executable permissions: {:?}",
                filepath.display(),
                permissions.mode() & 0o111 != 0,
            );
        }

        Ok(filepath)
    } else {
        bail!(
            "Found `{}`, but missing executable permissions!",
            filepath.display(),
        );
    }
}

fn check_up_file(filepath: PathBuf) -> Result<()> {
    let contents = fs::read_to_string(filepath)?;
    // Deprecate commands.pipe after 97de790. See #652 for details.
    if contents.contains("leftwm/commands.pipe") {
        bail!("`commands.pipe` is deprecated. See https://github.com/leftwm/leftwm/issues/652 for workaround.");
    }
    Ok(())
}

fn check_theme_toml(filepath: PathBuf, verbose: bool) -> Result<PathBuf> {
    let metadata = fs::metadata(&filepath)?;
    let contents = fs::read_to_string(&filepath.as_path())?;

    if metadata.is_file() {
        if verbose {
            println!("Found: {}", filepath.display());
        }

        match toml::from_str::<ThemeSetting>(&contents) {
            Ok(_) => {
                if verbose {
                    println!("The theme file looks OK.");
                }
                Ok(filepath)
            }
            Err(err) => bail!("Could not parse theme file: {}", err),
        }
    } else {
        bail!("No `theme.toml` found at path: {}", filepath.display());
    }
}
