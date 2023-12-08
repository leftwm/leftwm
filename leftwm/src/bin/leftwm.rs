//! Starts leftwm programs.
//!
//! If no arguments are passed, starts `leftwm-worker`. If arguments are passed, starts
//! `leftwm-{check, command, state, theme}` as specified, and passes along any extra arguments.

use clap::{command, Parser, Subcommand};
use leftwm_core::child_process::{self, Nanny};
use std::env;
use std::path::Path;
use std::process::{exit, Child, Command, ExitStatus};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

type SubcommandArgs = Vec<String>;
type LeftwmArgs = Vec<String>;

const SUBCOMMAND_PREFIX: &str = "leftwm-";

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Starts LeftWM if no arguments are supplied. If a subcommand is given, executes the \
             the corresponding leftwm program, e.g. 'leftwm theme' will execute 'leftwm-theme', if \
             it is installed."
)]
struct LeftwmCli {
    /// The backend used to run leftwm on.
    /// Can be either of xlib or x11rb
    #[arg(short, long)]
    backend: Option<String>,

    /// Execute a leftwm subcommand
    #[command(subcommand)]
    subcommand: Option<LeftwmSubcommand>,

    // /// Remaining args for subcommands
    // args: Vec<String>
}

/// Every leftwm subcommand
#[derive(Debug, Subcommand)]
enum LeftwmSubcommand {
    /// Check syntax of the configuration file
    Check,
    /// Send external commands to LeftWM
    Command,
    /// Print the current state of LeftWM
    State,
    /// Manage LeftWM themes
    Theme,
    /// Manage LeftWM configuration file
    Config,
}

impl ToString for LeftwmSubcommand {
    fn to_string(&self) -> String {
        String::from(match self {
            LeftwmSubcommand::Check => "check",
            LeftwmSubcommand::Command => "command",
            LeftwmSubcommand::State => "state",
            LeftwmSubcommand::Theme => "theme",
            LeftwmSubcommand::Config => "config",
        })
    }
}

fn main() {
    let args: LeftwmArgs = env::args().collect();
    let cli = LeftwmCli::parse();

    if let Some(subcommand) = cli.subcommand {
        let mut subcommand_args = args[1..].to_vec();
        subcommand_args.retain(|f| f != &subcommand.to_string());
        execute_subcommand(subcommand, subcommand_args);
    } else if let Some(backend) = cli.backend {
        start_leftwm(&backend);
    } else {
        println!("You need to specify a backend or a subcommand.");
        println!("Type 'leftwm --help' for more informations.");
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
/// - `subcommand`: The `leftwm-{subcommand}` which should be executed
/// - `subcommand_args`: The arguments which should be given to the `leftwm-{subcommand}`
fn execute_subcommand(subcommand: LeftwmSubcommand, subcommand_args: SubcommandArgs) -> ! {
    let subcommand_file = format!("{SUBCOMMAND_PREFIX}{}", subcommand.to_string());
    match &mut Command::new(subcommand_file).args(subcommand_args).spawn() {
        Ok(child) => {
            let status = child.wait().expect("Failed to wait for child.");
            exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            eprintln!("Failed to execute {}. {e}", subcommand.to_string());
            exit(1);
        }
    };
}

/// Sets some relevant environment variables for leftwm
fn set_env_vars() {
    env::set_var("XDG_CURRENT_DESKTOP", "LeftWM");

    // Fix for Java apps so they repaint correctly
    env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");
}

fn get_current_exe() -> std::path::PathBuf {
    #[cfg(not(target_os = "openbsd"))]
    {
        std::env::current_exe().expect("can't get path to leftwm-binary")
    }

    #[cfg(target_os = "openbsd")]
    {
        // OpenBSD panics at current_exe() call because the OS itself does not
        // provide a function to get the current executable. For LeftWM
        // purposes just args[0] works fine under OpenBSD.
        let arg0 = std::env::args()
            .next()
            .expect("Cannot get args[0] to compute leftwm executable path");
        std::path::PathBuf::from(arg0)
    }
}

/// The main-entry-point. The leftwm-session is prepared here
fn start_leftwm(backend_name: &String) {
    let current_exe = get_current_exe();

    set_env_vars();

    // Boot everything WM agnostic or LeftWM related in ~/.config/autostart
    let mut children = Nanny::autostart();

    let flag = get_sigchld_flag();

    let mut error_occured = false;
    let mut session_exit_status: Option<ExitStatus> = None;
    while !error_occured {
        let mut leftwm_session = start_leftwm_session(&current_exe, backend_name);
        #[cfg(feature = "lefthk")]
        let mut lefthk_session = start_lefthk_session(&current_exe);

        while session_is_running(&mut leftwm_session) {
            // remove all child processes which finished
            children.remove_finished_children();

            while is_suspending(&flag) {
                nix::unistd::pause();
            }
        }

        // we don't want a rougue lefthk session so we kill it when the leftwm one ended
        #[cfg(feature = "lefthk")]
        kill_lefthk_session(&mut lefthk_session);

        session_exit_status = get_exit_status(&mut leftwm_session);
        error_occured = check_error_occured(session_exit_status);

        // TODO: either add more details or find a better workaround.
        //
        // Left is too fast for some login managers. We need to
        // wait to give the login manager a second to boot.
        #[cfg(feature = "slow-dm-fix")]
        {
            let delay = std::time::Duration::from_millis(2000);
            std::thread::sleep(delay);
        }
    }

    if error_occured {
        print_crash_message();
    }

    match session_exit_status {
        Some(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
        None => std::process::exit(1),
    };
}

/// checks if leftwm is still running
fn session_is_running(leftwm_session: &mut Child) -> bool {
    leftwm_session
        .try_wait()
        .expect("failed to wait on worker")
        .is_none()
}

/// starts the leftwm session and returns the process/leftwm-session
fn start_leftwm_session(current_exe: &Path, backend_name: &String) -> Child {
    let worker_file = current_exe.with_file_name("leftwm-worker");

    Command::new(worker_file)
        .arg(backend_name)
        .spawn()
        .expect("failed to start leftwm")
}

/// Starts the lefthk session and returns the process/lefthk-session
#[cfg(feature = "lefthk")]
fn start_lefthk_session(current_exe: &Path) -> Child {
    let worker_file = current_exe.with_file_name("lefthk-worker");

    Command::new(worker_file)
        .spawn()
        .expect("failed to start lefthk")
}

/// Kills the lefthk session
#[cfg(feature = "lefthk")]
fn kill_lefthk_session(lefthk_session: &mut Child) {
    if lefthk_session.kill().is_ok() {
        while lefthk_session
            .try_wait()
            .expect("failed to reap lefthk")
            .is_none()
        {}
    }
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

fn get_exit_status(leftwm_session: &mut Child) -> Option<ExitStatus> {
    leftwm_session.wait().ok()
}

fn check_error_occured(session_exit_status: Option<ExitStatus>) -> bool {
    if let Some(exit_status) = session_exit_status {
        !exit_status.success()
    } else {
        true
    }
}

fn print_crash_message() {
    println!(concat!(
        "Leftwm crashed due to an unexpected error.\n",
        "Please create a new issue and post its log if possible.\n",
        "\n",
        "NOTE: You can restart leftwm with `startx`."
    ));
}
