//! Starts leftwm programs.
//!
//! If no arguments are passed, starts `leftwm-worker`. If arguments are passed, starts
//! `leftwm-{check, command, state, theme}` as specified, and passes along any extra arguments.

use clap::{command, crate_version};
use leftwm_core::child_process::{self, Nanny};
use std::env;
use std::path::Path;
use std::process::{exit, Child, Command};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

type Subcommand<'a> = &'a str;
type SubcommandArgs = Vec<String>;
type LeftwmArgs = Vec<String>;

const SUBCOMMAND_PREFIX: &str = "leftwm-";

const SUBCOMMAND_NAME_INDEX: usize = 0;
const SUBCOMMAND_DESCRIPTION_INDEX: usize = 1;
const AVAILABLE_SUBCOMMANDS: [[&str; 2]; 4] = [
    ["check", "Check syntax of the configuration file"],
    ["command", "Send external commands to LeftWM"],
    ["state", "Print the current state of LeftWM"],
    ["theme", "Manage LeftWM themes"],
];

fn main() {
    let args: LeftwmArgs = env::args().collect();

    let has_subcommands = args.len() > 1;
    if has_subcommands {
        parse_subcommands(&args);
    }

    start_leftwm();
}

/// Executes a subcommand.
///
/// If a valid subcommand is supplied, executes that subcommand, passing `args` to the program.
/// Prints an error to `STDERR` and exits non-zero if an invalid subcommand is supplied, or there is
/// some error while executing the subprocess.
///
/// # Arguments
///
/// - `subcommand`: The `leftwm-{subcommand}` which should be executed
/// - `subcommand_args`: The arguments which should be given to the `leftwm-{subcommand}`
fn execute_subcommand(subcommand: Subcommand, subcommand_args: SubcommandArgs) -> ! {
    let subcommand_file = format!("{}{}", SUBCOMMAND_PREFIX, subcommand);
    match &mut Command::new(&subcommand_file).args(subcommand_args).spawn() {
        Ok(child) => {
            let status = child.wait().expect("Failed to wait for child.");
            exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            eprintln!("Failed to execute {}. {}", subcommand, e);
            exit(1);
        }
    };
}

/// Prints the help page of leftwm (the output of `leftwm --help`)
fn print_help_page() {
    let version = format!(
        "{}, Git-Hash: {}",
        crate_version!(),
        git_version::git_version!(fallback = option_env!("GIT_HASH").unwrap_or("NONE"))
    );

    let subcommands = {
        let mut subcommands = Vec::new();
        for entry in AVAILABLE_SUBCOMMANDS {
            let subcommand_name = entry[SUBCOMMAND_NAME_INDEX];
            let subcommand_description = entry[SUBCOMMAND_DESCRIPTION_INDEX];

            subcommands.push(clap::Command::new(subcommand_name).about(subcommand_description));
        }
        subcommands
    };

    command!()
        .long_about(
            "Starts LeftWM if no arguments are supplied. If a subcommand is given, executes the \
             the corresponding leftwm program, e.g. 'leftwm theme' will execute 'leftwm-theme', if \
             it is installed.",
        )
        .version(version.as_str())
        .subcommands(subcommands)
        .print_help()
        .unwrap();
}

/// Checks if the given subcommand-string is a `leftwm-{subcommand}`
fn is_subcommand(subcommand: &str) -> bool {
    AVAILABLE_SUBCOMMANDS
        .into_iter()
        .any(|entry| entry[SUBCOMMAND_NAME_INDEX] == subcommand)
}

/// Tries to parse the subcommands from the arguments of leftwm and executes them if suitalbe.
/// Otherwise it's calling the help-page.
fn parse_subcommands(args: &LeftwmArgs) -> ! {
    const SUBCOMMAND_INDEX: usize = 1;
    const SUBCOMMAND_ARGS_INDEX: usize = 2;

    let subcommand = &args[SUBCOMMAND_INDEX];
    let subcommand_args = args[SUBCOMMAND_ARGS_INDEX..].to_vec();

    if is_subcommand(subcommand) {
        execute_subcommand(subcommand, subcommand_args);
    } else {
        print_help_page();
    }

    exit(0);
}

/// Sets some relevant environment variables for leftwm
fn set_env_vars() {
    env::set_var("XDG_CURRENT_DESKTOP", "LeftWM");

    // Fix for Java apps so they repaint correctly
    env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");
}

/// The main-entry-point. The leftwm-session is prepared here
fn start_leftwm() {
    let current_exe = std::env::current_exe().expect("can't get path to leftwm-binary");

    set_env_vars();

    // Boot everything WM agnostic or LeftWM related in ~/.config/autostart
    let mut children = Nanny::autostart();

    let flag = get_sigchld_flag();

    loop {
        let mut leftwm_session = start_leftwm_session(&current_exe);
        while leftwm_is_still_running(&mut leftwm_session) {
            // remove all child processes which finished
            children.remove_finished_children();

            while is_suspending(&flag) {
                nix::unistd::pause();
            }
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

/// checks if leftwm is still running
fn leftwm_is_still_running(leftwm_session: &mut Child) -> bool {
    leftwm_session
        .try_wait()
        .expect("failed to wait on worker")
        .is_none()
}

/// starts the leftwm session and returns the process/leftwm-session
fn start_leftwm_session(current_exe: &Path) -> Child {
    let worker_file = current_exe.with_file_name("leftwm-worker");

    Command::new(&worker_file)
        .spawn()
        .expect("failed to start leftwm")
}

/// The SIGCHLD can be set by the children of leftwm if their window need a refresh for example.
/// So we're returning the flag to check when leftwm can be suspended and when not.
/// Click [here](https://frameboxxindore.com/linux/what-is-sigchld-in-linux.html) for an
/// example-description.
fn get_sigchld_flag() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    child_process::register_child_hook(flag.clone());

    flag
}

/// Looks, if leftwm can be suspended at the moment.
/// ## Returns
/// - `true` if leftwm doesn't need to do anything at them moment
/// - `false` if leftwm needs to refresh its state
fn is_suspending(flag: &Arc<AtomicBool>) -> bool {
    !flag.swap(false, Ordering::SeqCst)
}
