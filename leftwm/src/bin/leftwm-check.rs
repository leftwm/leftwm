use anyhow::{bail, Result};
use clap::{arg, command};
use leftwm::utils::file_handler::{
    check_file_type, check_path, get_default_path, load_config_file, migrate_config,
};
use leftwm::Config;
use leftwm::ThemeSetting;
use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = command!("LeftWM Check")
        .about("Checks syntax of the configuration file")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-v --verbose "Outputs received configuration file."),
            arg!(migrate: -m --"migrate-toml-to-ron" "Migrates an exesting `toml` based config to a `ron` based one.\nKeeps the old file for reference, please delete it manually."),
            arg!([INPUT] "Sets the input file to use. Uses first in PATH otherwise."),
        ])
        .get_matches();

    let config_file = matches.get_one::<String>("INPUT").map(String::as_str);
    let verbose = matches.get_flag("verbose");

    println!(
        "\x1b[0;94m::\x1b[0m LeftWM version: {}",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "\x1b[0;94m::\x1b[0m LeftWM git hash: {}",
        git_version::git_version!(fallback = option_env!("GIT_HASH").unwrap_or("NONE"))
    );
    if matches.get_flag("migrate") {
        println!("\x1b[0;94m::\x1b[0m Migrating configuration . . .");
        let toml_file = if let Some(config_file) = config_file {
            PathBuf::from(config_file)
        } else {
            get_default_path()?.with_extension("toml")
        };
        migrate_config(toml_file, verbose)?;

        return Ok(());
    }

    match check_enabled_features() {
        Ok(_) => {}
        Err(err) => {
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {err} \x1b[0m");
        }
    }

    println!("\x1b[0;94m::\x1b[0m Loading configuration . . .");
    match check_config_file(config_file, verbose) {
        Ok(config) => {
            println!("\x1b[0;92m    -> Configuration loaded OK \x1b[0m");
            if verbose {
                dbg!(&config);
            }
            config.check_mousekey(verbose);
            #[cfg(not(feature = "lefthk"))]
            println!("\x1b[1;93mWARN: Ignoring checks on keybinds as you compiled for an external hot key daemon.\x1b[0m");
            #[cfg(feature = "lefthk")]
            config.check_keybinds(verbose);
        }
        Err(e) => {
            println!("Configuration failed. Reason: {e:?}");
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
fn check_config_file(fspath: Option<&str>, verbose: bool) -> Result<Config> {
    let config_path = if let Some(fspath) = fspath {
        Some(PathBuf::from(fspath))
    } else {
        None
    };

    let config_filename = match check_path(config_path, verbose) {
        Ok(path) => path,
        Err(_) => {
            load_config_file(None)?;
            get_default_path()?
        }
    };

    if verbose {
        dbg!(&config_filename);
    }
    let contents = fs::read_to_string(&config_filename)?;
    if verbose {
        dbg!(&contents);
    }
    if check_file_type(&config_filename) == leftwm::utils::file_handler::ConfigFileType::TomlFile {
        println!("\x1b[1;35mYou are using TOML as config language which will be deprecated in the future.\nPlease consider migrating you config to RON. For further info visit the leftwm wiki. \x1b[0m");
    };
    load_config_file(Some(config_filename))
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
                println!(":: XDG_RUNTIME_DIR: {val}, LOGINCTL OKAY");
            }

            println!("\x1b[0;92m    -> Environment OK \x1b[0m");

            Ok(())
        }
        (Ok(val), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {val}, LOGINCTL not installed");
            }

            println!("\x1b[0;92m    -> Environment OK (has XDG_RUNTIME_DIR) \x1b[0m");

            Ok(())
        }
        (Err(e), false) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR_ERROR: {e:?}, LOGINCTL BAD");
            }

            bail!(
                "Elogind not installed/operating and no alternative XDG_RUNTIME_DIR is set. \
                See https://github.com/leftwm/leftwm/wiki/XDG_RUNTIME_DIR for more information."
            );
        }
        (Err(e), true) => {
            if verbose {
                println!(":: XDG_RUNTIME_DIR: {e:?}, LOGINCTL OKAY");
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
    let err_formatter = |s| println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {s} \x1b[0m");

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
        returns.push(format!("File not found: {missing_file}"));
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
            f if f.ends_with("theme.ron") => match check_theme_ron(f, verbose) {
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
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {error} \x1b[0m");
        }
        false
    }
}

fn missing_expected_file(filepaths: &[PathBuf]) -> impl Iterator<Item = &&str> {
    ["up", "down", "theme.ron"]
        .iter()
        .filter(move |f| !filepaths.iter().any(|fp| fp.ends_with(f)))
}

fn check_current_theme_set(filepath: &Option<PathBuf>, verbose: bool) -> Result<&PathBuf> {
    match &filepath {
        Some(p) => {
            if verbose {
                if fs::symlink_metadata(p)?.file_type().is_symlink() {
                    println!(
                        "Found symlink `current`, pointing to theme folder: {:?}",
                        fs::read_link(p).unwrap()
                    );
                } else {
                    println!("\x1b[1;93mWARN: Found `current` theme folder: {p:?}. Use of a symlink is recommended, instead.\x1b[0m");
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
    let contents = fs::read_to_string(filepath.as_path())?;

    if metadata.is_file() {
        if verbose {
            println!("Found: {}", filepath.display());
        }

        match toml::from_str::<ThemeSetting>(&contents) {
            Ok(_) => {
                if verbose {
                    println!("The theme file looks OK.");
                }
                println!(
                    "\x1b[1;93mWARN: TOML as config format is about to be deprecated.
      Please consider migrating to RON or contact the theme creator about this topic.
      Note: make sure the `up` script is loading the correct theme file.\x1b[0m"
                );
                Ok(filepath)
            }
            Err(err) => bail!("Could not parse theme file: {}", err),
        }
    } else {
        bail!("No `theme.toml` found at path: {}", filepath.display());
    }
}

fn check_theme_ron(filepath: PathBuf, verbose: bool) -> Result<PathBuf> {
    let metadata = fs::metadata(&filepath)?;
    let contents = fs::read_to_string(filepath.as_path())?;

    if metadata.is_file() {
        if verbose {
            println!("Found: {}", filepath.display());
        }

        match ron::from_str::<ThemeSetting>(&contents) {
            Ok(_) => {
                if verbose {
                    println!("The theme file looks OK.");
                }
                Ok(filepath)
            }
            Err(err) => bail!("Could not parse theme file: {}", err),
        }
    } else {
        bail!("No `theme.ron` found at path: {}", filepath.display())
    }
}
// this function is called only when specific features are enabled.
#[allow(dead_code)]
fn check_feature<T, E, F>(name: &str, predicate: F) -> Result<()>
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Debug,
{
    match predicate() {
        Ok(_) => {
            println!("\x1b[0;92m    -> {name} OK\x1b[0m");
            Ok(())
        }
        Err(err) => bail!("Check for feature {} failed: {:?}", name, err),
    }
}

fn check_enabled_features() -> Result<()> {
    if env!("LEFTWM_FEATURES").is_empty() {
        println!("\x1b[0;94m::\x1b[0m Built with no enabled features.");
        return Ok(());
    }

    println!(
        "\x1b[0;94m::\x1b[0m Enabled features:{}",
        env!("LEFTWM_FEATURES")
    );

    println!("\x1b[0;94m::\x1b[0m Checking feature dependencies . . .");

    #[cfg(feature = "journald-log")]
    check_feature("journald-log", tracing_journald::layer)?;
    #[cfg(feature = "lefthk")]
    // TODO once we refactor all file handling into a utiliy module, we want to call a `path-builder` method from that module
    check_feature("lefthk", || {
        if let Ok(path) = env::var("PATH") {
            for p in path.split(':') {
                let path = format!("{p}/{}", "lefthk-worker");
                if Path::new(&path).exists() {
                    return Ok(());
                }
            }
        }
        Err("Could not find lefthk")
    })?;

    Ok(())
}
