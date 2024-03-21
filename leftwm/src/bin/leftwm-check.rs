use anyhow::{bail, Result};
use clap::{arg, command};
use leftwm::{Config, ThemeConfig};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
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
        option_env!("GIT_HASH").unwrap_or(git_version::git_version!(fallback = "unknown"))
    );
    if matches.get_flag("migrate") {
        println!("\x1b[0;94m::\x1b[0m Migrating configuration . . .");
        let path = BaseDirectories::with_prefix("leftwm")?;
        let ron_file = path.place_config_file("config.ron")?;
        let toml_file = path.place_config_file("config.toml")?;

        let config = load_from_file(toml_file.as_os_str().to_str(), verbose)?;

        write_to_file(&ron_file, &config)?;

        return Ok(());
    }

    match check_enabled_features(verbose) {
        Ok(()) => {}
        Err(err) => {
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {err} \x1b[0m");
        }
    }

    match check_binaries(verbose) {
        Ok(()) => {}
        Err(err) => {
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {err} \x1b[0m");
        }
    }

    println!("\x1b[0;94m::\x1b[0m Loading configuration . . .");
    match load_from_file(config_file, verbose) {
        Ok(config) => {
            println!("\x1b[0;92m    -> Configuration loaded OK \x1b[0m");
            if verbose {
                dbg!(&config);
            }
            config.check_mousekey(verbose);
            config.check_log_level(verbose);
            #[cfg(not(feature = "lefthk"))]
            println!("\x1b[1;93mWARN: Ignoring checks on keybinds as you compiled for an external hot key daemon.\x1b[0m");
            #[cfg(feature = "lefthk")]
            config.check_keybinds(verbose);
        }
        Err(e) => {
            println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m Configuration failed. Reason: {e:?}");
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
    let config_filename = if let Some(fspath) = fspath {
        println!("\x1b[1;35mNote: Using file {fspath} \x1b[0m");
        PathBuf::from(fspath)
    } else {
        let ron_file = BaseDirectories::with_prefix("leftwm")?.place_config_file("config.ron")?;
        let toml_file = BaseDirectories::with_prefix("leftwm")?.place_config_file("config.toml")?;
        if Path::new(&ron_file).exists() {
            ron_file
        } else if Path::new(&toml_file).exists() {
            println!(
                "\x1b[1;93mWARN: TOML as config format is about to be deprecated.
      Please consider migrating to RON manually or by using `leftwm-check -m`.\x1b[0m"
            );
            toml_file
        } else {
            let config = Config::default();
            write_to_file(&ron_file, &config)?;
            return Ok(config);
        }
    };

    if verbose {
        dbg!(&config_filename);
    }
    let contents = fs::read_to_string(&config_filename)?;
    if verbose {
        dbg!(&contents);
    }
    if config_filename.as_path().extension() == Some(std::ffi::OsStr::new("ron")) {
        let ron = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        let config: Config = ron.from_str(&contents)?;
        Ok(config)
    } else {
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}

fn write_to_file(ron_file: &Path, config: &Config) -> Result<(), anyhow::Error> {
    let ron_pretty_conf = PrettyConfig::new()
        .depth_limit(2)
        .extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
    let ron = to_string_pretty(&config, ron_pretty_conf)?;
    let comment_header = String::from(
        r"//  _        ___                                      ___ _
// | |      / __)_                                   / __|_)
// | | ____| |__| |_ _ _ _ ____      ____ ___  ____ | |__ _  ____    ____ ___  ____
// | |/ _  )  __)  _) | | |    \    / ___) _ \|  _ \|  __) |/ _  |  / ___) _ \|  _ \
// | ( (/ /| |  | |_| | | | | | |  ( (__| |_| | | | | |  | ( ( | |_| |  | |_| | | | |
// |_|\____)_|   \___)____|_|_|_|   \____)___/|_| |_|_|  |_|\_|| (_)_|   \___/|_| |_|
// A WindowManager for Adventurers                         (____/
// For info about configuration please visit https://github.com/leftwm/leftwm/wiki

",
    );
    let ron_with_header = comment_header + &ron;
    let mut file = File::create(ron_file)?;
    file.write_all(ron_with_header.as_bytes())?;
    Ok(())
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
                    Ok(()) => continue,
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

        match toml::from_str::<ThemeConfig>(&contents) {
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

        let ron = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);
        match ron.from_str::<ThemeConfig>(&contents) {
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

/// Checks the enabled features and attempts to find their dependencies/binaries as applicable
///
/// # Errors
/// - An enabled feature is missing a dependency
///     Resolutions may include:
///         - Disable the feature (remove from --features at compile time)
///         - Install any dependency/dependencies which are missing
///         - Ensure all binaries are installed to a location in your PATH
fn check_enabled_features(verbose: bool) -> Result<()> {
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
    check_feature("lefthk", || check_binary("lefthk-worker", verbose))?;

    Ok(())
}

/// Check to determine if the standard leftwm binaries are present
///
/// Binaries checked in this function: leftwm-worker, leftwm, leftwm-state, leftwm-check, leftwm-command
///
/// # Errors
/// - At least one binary has failed to be found in your PATH variable
fn check_binaries(verbose: bool) -> Result<()> {
    // Assumption: Required binaries are leftwm-worker, leftwm-state, leftwm, leftwm-check,
    // leftwm-command
    // Assumption: lefthk-worker is checked in check_enabled_features if needed
    println!("\x1b[0;94m::\x1b[0m Checking for leftwm binaries . . .");
    let binaries: [&str; 5] = [
        "leftwm",
        "leftwm-worker",
        "leftwm-state",
        "leftwm-command",
        "leftwm-check",
    ];
    let mut failures: bool = false;
    for binary in binaries {
        match check_binary(binary, verbose) {
            Ok(()) => {}
            Err(err) => {
                failures = true;
                println!("\x1b[1;91mERROR:\x1b[0m\x1b[1m {err} \x1b[0m");
            }
        }
    }
    if failures {
        bail!("Not all required binaries are present");
    }
    println!("\x1b[0;92m    -> Binaries OK \x1b[0m");
    Ok(())
}

/// Check to determine if `binary` exists in PATH
///
/// # Errors
/// - Will return an error if the listed `binary` can not be found in PATH
///     Resolutions may include:
///         - Installing leftwm using `cargo install --path {path}` where {path} is a directory in
///         PATH
///         - Setting the PATH variable, usually in .bashrc, .profile, or similar depending on your
///         shell
/// - Will return an error if the PATH environmental variable is not set
///     Resolutions may include:
///         - Setting the PATH variable, usually in .bashrc, .profile, or similar depending on your
///         shell
fn check_binary(binary: &str, verbose: bool) -> Result<()> {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let path = format!("{p}/{binary}");
            if Path::new(&path).exists() {
                if verbose {
                    println!("In search for binaries, found {}", &path);
                }
                return Ok(());
            }
        }
        bail!("Could not find binary {} in PATH", binary)
    }

    bail!("Binaries not checked. This is an error with leftwm-check, we would appreciate a bug report: https://github.com/leftwm/leftwm/issues/new?assignees=&labels=bug&projects=&template=bug_report.yml")
}
