use clap::{arg, command, ArgGroup, Id};
use std::process::exit;
#[cfg(any(feature = "sys-log", feature = "journald-log", feature = "file-log"))]
use std::process::Command;

fn main() {
    let matches = get_command().get_matches();
    #[cfg(any(feature = "sys-log", feature = "journald-log", feature = "file-log"))]
    let follow = matches.get_flag("follow");
    #[cfg(any(feature = "sys-log", feature = "journald-log", feature = "file-log"))]
    let level = matches.get_count("verbose");

    #[allow(unreachable_patterns)]
    match matches.get_one::<Id>("log").map(clap::Id::as_str) {
        #[cfg(feature = "journald-log")]
        Some("journald") | None => journald_log(follow, level),
        #[cfg(feature = "sys-log")]
        Some("syslog") | None => syslog(follow),
        #[cfg(feature = "file-log")]
        Some("file") | None => file_log(follow, level),
        #[cfg(not(any(feature = "journald-log", feature = "sys-log", feature = "file-log")))]
        _ => {
            eprintln!("Failed to execute: logging not enabled");
            exit(1);
        }
        _ => unreachable!("Unreachable feature set!"),
    }
}

fn get_command() -> clap::Command {
    command!("LeftWM Log")
        .about("retrieves information logged by leftwm-worker")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-J --journald "use journald log (default)"),
            arg!(-S --syslog "use syslog (default if built with no journald support)"),
            arg!(-F --file "use file (default if built with no syslog support)"),
            arg!(-f --follow "output appended data as the log grows"),
            arg!(-v --verbose... "verbosity level"),
        ])
        .group(
            ArgGroup::new("log")
                .args(vec!["journald", "syslog", "file"])
                .required(false),
        )
}

#[cfg(feature = "journald-log")]
fn journald_log(follow: bool, level: u8) {
    let follow_flag = if follow { " -f" } else { "" };
    let level_flag = level + 4; // Default level is warn (4)
    match &mut Command::new("/bin/sh")
        .args([
            "-c",
            format!("journalctl{follow_flag} -p {level_flag} $(which leftwm-worker) $(which lefthk-worker) $(which leftwm-command)").as_str(),
        ])
        .spawn()
    {
        Ok(child) => {
            let status = child.wait().expect("Failed to wait for child.");
            exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            eprintln!("Failed to execute . {e}");
            exit(1);
        }
    }
}

#[cfg(feature = "sys-log")]
fn syslog(follow: bool) {
    let cmd = if follow { "tail -f" } else { "cat" };
    match &mut Command::new("/bin/sh")
        .args([
            "-c",
            format!("{cmd} /var/log/syslog | grep \"left[wh][mk].*\"").as_str(),
        ])
        .spawn()
    {
        Ok(child) => {
            let status = child.wait().expect("Failed to wait for child.");
            exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            eprintln!("Failed to execute . {e}");
            exit(1);
        }
    }
}

#[cfg(feature = "file-log")]
fn file_log(follow: bool, level: u8) {
    const TIME_REGEX: &str =
        "[0-9]{4}-[01][1-9]-[1-3][0-9]T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z.{10}";
    let cmd = if follow { "tail -f" } else { "cat" };
    let filter = match level {
        0 => "ERROR|WARN",
        1 => "ERROR|WARN|INFO",
        2 => "ERROR|WARN|INFO|DEBUG",
        _ => "ERROR|WARN|INFO|DEBUG|TRACE",
    };
    let res = {
        let file_path = leftwm::utils::log::file::get_log_path();
        // ugly shadowing to make the borrow checker happy
        let file_path = file_path.to_string_lossy();
        println!("Output from {file_path} - {filter}:");
        &mut Command::new("/bin/sh")
            .args([
                "-c",
                format!("{cmd} {file_path} | grep -E \"{TIME_REGEX}{filter}\"").as_str(),
            ])
            .spawn()
    };
    match res {
        Ok(child) => {
            let status = child.wait().expect("Failed to wait for child.");
            exit(status.code().unwrap_or(0));
        }
        Err(e) => {
            eprintln!("Failed to execute . {e}");
            exit(1);
        }
    };
}
