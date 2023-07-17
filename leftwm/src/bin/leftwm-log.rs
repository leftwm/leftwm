use clap::{arg, command, ArgGroup, Id};
use std::process::{exit, Command};

fn main() {
    let matches = get_command().get_matches();
    let follow = matches.get_flag("follow");

    match matches.get_one::<Id>("log").map(clap::Id::as_str) {
        #[cfg(feature = "journald-log")]
        Some("journald") | None => journald_log(follow),
        #[cfg(feature = "sys-log")]
        Some("syslog") | None => syslog(follow),
        #[cfg(feature = "file-log")]
        Some("file") | None => file_log(follow),
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
        ])
        .group(
            ArgGroup::new("log")
                .args(vec!["journald", "syslog", "file"])
                .required(false),
        )
}

#[cfg(feature = "journald-log")]
fn journald_log(follow: bool) {
    let flag = if follow { " -f" } else { "" };
    match &mut Command::new("/bin/sh")
        .args([
            "-c",
            format!("journalctl{flag} $(which leftwm-worker) $(which lefthk-worker) $(which leftwm-command)").as_str(),
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
fn file_log(follow: bool) {
    let cmd = match follow {
        true => "tail -f",
        false => "cat",
    };
    match {
        let file_path = leftwm::utils::log::file::get_log_path();
        println!("output from {}:", file_path.display());
        &mut Command::new("/bin/sh")
            .args([
                "-c",
                format!("{cmd} {}", file_path.to_str().unwrap()).as_str(),
            ])
            .spawn()
    } {
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
