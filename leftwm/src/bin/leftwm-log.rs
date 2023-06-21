use clap::{arg, command, ArgGroup};
#[cfg(feature = "file-log")]
use leftwm::utils::log::file::get_log_path;
use std::process::{exit, Command};

fn main() {
    let matches = get_command().get_matches();

    if matches.get_flag("journald") {
        match cfg!(feature = "journald-log") || matches.get_flag("ignore-build-opts") {
            true => journald_log(),
            false => {
                eprintln!("Failed to execute: leftwm was not build with journald logging");
                exit(1)
            }
        }
    } else if matches.get_flag("syslog") {
        match cfg!(feature = "sys-log") || matches.get_flag("ignore-build-opts") {
            true => syslog(),
            false => {
                eprintln!("Failed to execute: leftwm was not build with syslog logging");
                exit(1)
            }
        }
    } else if matches.get_flag("file") {
        #[cfg(feature = "file-log")]
        file_log();
        #[cfg(not(feature = "file-log"))]
        {
            eprintln!("Failed to execute: leftwm was not build with file logging");
            exit(1);
        }
    } else if cfg!(feature = "journald-log") {
        journald_log();
    } else if cfg!(feature = "sys-log") {
        syslog();
    } else if cfg!(feature = "file-log") {
        #[cfg(feature = "file-log")]
        file_log();
    } else {
        eprintln!("Failed to execute: logging not enabled");
        exit(1);
    }
}

fn get_command() -> clap::Command {
    command!("LeftWM Log")
        .about("retrieves information logged by leftwm-worker")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-j --journald "use journald log (default)"),
            arg!(-s --syslog "use syslog (default if built with no journald support"),
            arg!(-f --file "use file (default if built with no syslog support"),
            arg!(-i --"ignore-build-opts" "attempt logging regardless of build options"),
        ])
        .group(
            ArgGroup::new("log")
                .args(vec!["journald", "syslog", "file"])
                .required(false),
        )
}

fn journald_log() {
    match &mut Command::new("/bin/sh")
        .args(["-c", "journalctl -f \"$(which leftwm-worker)\""])
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

fn syslog() {
    match &mut Command::new("/bin/sh")
        .args(["-c", "tail -f /var/log/syslog | grep leftwm"])
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
fn file_log() {
    match {
        let file_path = get_log_path();
        &mut Command::new("/bin/sh")
            .args([
                "-c",
                format!("tail -f {}", file_path.to_str().unwrap()).as_str(),
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
