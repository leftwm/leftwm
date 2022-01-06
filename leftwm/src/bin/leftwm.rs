//! Starts leftwm programs.
//!
//! If no arguments are passed, starts `leftwm-worker`. If arguments are passed, starts
//! `leftwm-{check, command, state, theme}` as specified, and passes along any extra arguments.

use clap::{crate_version, App, AppSettings, SubCommand};
use leftwm_core::child_process::{self, Nanny};
use std::collections::BTreeMap;
use std::env;
use std::process::{exit, Command};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn main() {
    let mut subcommands = BTreeMap::new();

    // This is a complete list of accepted subcommands. To add a new one, add a new `insert()` here.
    subcommands.insert("check", "Check syntax of the configuration file");
    subcommands.insert("command", "Send external commands to LeftWM");
    subcommands.insert("state", "Print the current state of LeftWM");
    subcommands.insert("theme", "Manage LeftWM themes");

    let subcommand_names: Vec<&str> = subcommands.keys().copied().collect();

    let args: Vec<String> = env::args().collect();

    // If called with arguments, attempt to execute a subcommand.
    if args.len() > 1 {
        match execute_subcommand(&args, &subcommand_names) {
            // Subcommand executed. Exit success.
            Some(true) => exit(0),
            // Subcommand was valid, but failed to execute. Exit failure.
            Some(false) => exit(1),
            // Subcommand was invalid. Let clap handle help, version or error messages.
            None => handle_help_or_version_flags(&args, &subcommands),
        }
        // execute_subcommand() should return `None` if no valid subcommand was given, and in that
        // case handle_help_or_version_flags() should display a help, version, or error message and
        // exit. If we get here, something unexpected has happened.
        unreachable!();
    }

    // If _not_ invoked with a subcommand, start leftwm.
    if let Ok(current_exe) = std::env::current_exe() {
        // Boot everything WM agnostic or LeftWM related in ~/.config/autostart
        env::set_var("XDG_CURRENT_DESKTOP", "LeftWM");
        let mut children = Nanny::autostart();

        let flag = Arc::new(AtomicBool::new(false));
        child_process::register_child_hook(flag.clone());

        // Fix for Java apps so they repaint correctly
        env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");

        let worker_path = current_exe.with_file_name("leftwm-worker");

        loop {
            let mut worker = Command::new(&worker_path)
                .spawn()
                .expect("failed to start leftwm");

            // Wait until worker exits.
            while worker
                .try_wait()
                .expect("failed to wait on worker")
                .is_none()
            {
                // Not worker, then it might be autostart programs.
                children.reap();
                // Wait for SIGCHLD signal flag to be set.
                while !flag.swap(false, Ordering::SeqCst) {
                    nix::unistd::pause();
                }
                // Either worker or autostart program exited.
            }

            // TODO: either add more details or find a better workaround.
            //
            // Left is too fast for some logging managers. We need to
            // wait to give the logging manager a second to boot.
            #[cfg(feature = "slow-dm-fix")]
            {
                let delay = std::time::Duration::from_millis(2000);
                std::thread::sleep(delay);
            }
        }
    }
}

/// Executes a subcommand.
///
/// If a valid subcommand is supplied, executes that subcommand, passing `args` to the program.
/// Prints an error to `STDERR` and exits non-zero if an invalid subcommand is supplied, or there is
/// some error while executing the subprocess.
///
/// # Arguments
///
/// + `args` - The command line arguments leftwm was called with. Must be length >= 2, or this will
///   panic.
/// + `subcommands` - A list of subcommands that should be considered valid. Subcommands not in this
///   list will not be executed.
///
/// # Panics
///
/// Panics if `args` has length < 2.
///
/// # Returns
///
/// Returns `Some(true)` if the subcommand ran.
/// Returns `Some(false)` if the first argument is a valid subcommand, but the associated program
/// failed to run.
/// Returns `None` if the first argument is not a valid subcommand.
fn execute_subcommand(args: &[String], subcommands: &[&str]) -> Option<bool> {
    // If the second argument is a valid subcommand
    if subcommands.iter().any(|x| x == &args[1]) {
        // Run the command
        let cmd = format!("leftwm-{}", &args[1]);
        match &mut Command::new(&cmd).args(&args[2..]).spawn() {
            Ok(child) => {
                // Wait for process to end, otherwise it may continue to run in the background.
                child.wait().expect("Failed to wait for child.");
                Some(true)
            }
            Err(e) => {
                eprintln!("Failed to execute {}. {}", cmd, e);
                Some(false)
            }
        }
    } else {
        None
    }
}

/// Show program help text and exit if `--help` or `--version` flags are passed, or if an invalid
/// argument is given.
///
/// If the first argument is a valid subcommand, this will do nothing, and will not exit.
/// This function is not intended to be called with valid subcommands as arguments, as it will exit
/// when given valid subcommands along with arguments. This is because we don't keep track of what
/// arguments are valid for each subcommand here, and `clap` assumes that all undocumented arguments
/// are erroneous.
///
/// # Arguments
///
/// + `args` - The command line arguments leftwm was called with. Do not pass in valid subcommands.
/// + `subcommands` - A map of subcommand names and their descriptions. This determines what
///   subcommands are listed in the help text, as well as what subcommands are considered as valid
///   arguments.
///
/// # Exits
///
/// Exits early if `--help` or `--version` flags are passed.
/// Exits early if an invalid subcommand is given.
/// Exits early if a valid subcommand is given along with arguments to it. Avoid this usage, as the
/// outcome is undesireable.
fn handle_help_or_version_flags(args: &[String], subcommands: &BTreeMap<&str, &str>) {
    // If there are more than two arguments, do not invoke `clap`, since `clap` will get confused
    // about arguments to subcommands and throw spurrious errors.
    let version = format!(
        "{}, Git-Hash: {}",
        crate_version!(),
        git_version::git_version!(fallback = option_env!("GIT_HASH").unwrap_or("NONE"))
    );
    let mut app = App::new("LeftWM")
        .author("Lex Childs <lex.childs@gmail.com>")
        .about("A window manager for adventurers.")
        .long_about(
            "Starts LeftWM if no arguments are supplied. If a subcommand is given, executes the \
             the corresponding leftwm program, e.g. 'leftwm theme' will execute 'leftwm-theme', if \
             it is installed.",
        )
        .version(&*version)
        .settings(&[AppSettings::DisableHelpSubcommand, AppSettings::ColoredHelp]);
    for (&subcommand, &description) in subcommands {
        app = app.subcommand(SubCommand::with_name(subcommand).about(description));
    }
    app.get_matches_from(args);
}
