//! Starts leftwm programs.
//!
//! If no arguments are passed, starts `leftwm-worker`. If arguments are passed, starts
//! `leftwm-{check, command, state, theme}` as specified, and passes along any extra arguments.

use clap::{command, crate_version, App, Arg, ArgMatches};
use leftwm_core::child_process::{self, Nanny};
use std::env;
use std::process::{exit, Command};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

type Subcommand = String;
type SubcommandArgs = Vec<String>;

const APP_ARG_NAME: &'static str = "trailing";
const SUBCOMMAND_PREFIX: &'static str = "leftwm-";
const APP_VERSION: &'static str = const_format::formatcp!(
    "{}, Git-Hash: {}",
    crate_version!(),
    git_version::git_version!(fallback = option_env!("GIT_HASH").unwrap_or("NONE"))
);

const SUBCOMMAND_NAME_INDEX: usize = 0;
const AVAILABLE_SUBCOMMANDS: [[&'static str; 2]; 4] = [
    ["check", "Check syntax of the configuration file"],
    ["command", "Send external commands to LeftWM"],
    ["state", "Print the current state of LeftWM"],
    ["theme", "Manage LeftWM themes"],
];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SubcommandStatus {
    ExitSuccess,
    ExitFailure,
}

fn main() {
    let app = get_app();
    let app_matches = app.get_matches();

    if let Some((subcommand, subcommand_args)) = get_subcommand_with_args(&app_matches) {
        match execute_subcommand(subcommand, subcommand_args) {
            SubcommandStatus::ExitSuccess => exit(0),
            SubcommandStatus::ExitFailure => exit(1),
        }
    }

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
fn execute_subcommand(subcommand: Subcommand, subcommand_args: SubcommandArgs) -> SubcommandStatus {
    let subcommand_file = format!("{}{}", SUBCOMMAND_PREFIX, subcommand);
    match &mut Command::new(&subcommand_file).args(subcommand_args).spawn() {
        Ok(child) => {
            // Wait for process to end, otherwise it may continue to run in the background.
            child.wait().expect("Failed to wait for child.");
            SubcommandStatus::ExitSuccess
        }
        Err(e) => {
            eprintln!("Failed to execute {}. {}", subcommand, e);
            SubcommandStatus::ExitFailure
        }
    }
}

fn get_app() -> App<'static> {
    command!()
        .long_about(
            "Starts LeftWM if no arguments are supplied. If a subcommand is given, executes the \
             the corresponding leftwm program, e.g. 'leftwm theme' will execute 'leftwm-theme', if \
             it is installed.",
        )
        .version(APP_VERSION)
        .arg(Arg::new(APP_ARG_NAME).multiple_values(true))
        .trailing_var_arg(true)
}

fn get_subcommand_with_args(app_matches: &ArgMatches) -> Option<(Subcommand, SubcommandArgs)> {
    if let Some(args) = app_matches.get_many::<String>(APP_ARG_NAME) {
        let mut args2 = args.clone();

        let subcommand = args2.next().unwrap().to_string();
        let subcommand_args = args2.map(|entry| (*entry).clone()).collect::<Vec<String>>();

        if is_subcommand(&subcommand) {
            Some((subcommand, subcommand_args))
        } else {
            None
        }
    } else {
        None
    }
}

fn is_subcommand<T: AsRef<str>>(subcommand: T) -> bool {
    return AVAILABLE_SUBCOMMANDS
        .into_iter()
        .find(|entry| entry[SUBCOMMAND_NAME_INDEX] == (subcommand.as_ref()))
        .is_some();
}
