use clap::{arg, command, ArgGroup};
#[cfg(feature = "file-log")]
use leftwm::utils::log::file::get_log_path;
use std::process::{exit, Command};

fn main() {
    let matches = get_command().get_matches();
    let follow = matches.get_flag("follow");

    if matches.get_flag("journald") {
        match cfg!(feature = "journald-log") || matches.get_flag("ignore-build-opts") {
            true => journald_log(follow),
            false => {
                eprintln!("Failed to execute: leftwm was not built with journald logging");
                exit(1)
            }
        }
    } else if matches.get_flag("syslog") {
        match cfg!(feature = "sys-log") || matches.get_flag("ignore-build-opts") {
            true => syslog(follow),
            false => {
                eprintln!("Failed to execute: leftwm was not built with syslog logging");
                exit(1)
            }
        }
    } else if matches.get_flag("file") {
        #[cfg(feature = "file-log")]
        file_log(follow);
        #[cfg(not(feature = "file-log"))]
        {
            eprintln!("Failed to execute: leftwm was not built with file logging");
            exit(1);
        }
    } else if cfg!(feature = "journald-log") {
        journald_log(follow);
    } else if cfg!(feature = "sys-log") {
        syslog(follow);
    } else if cfg!(feature = "file-log") {
        #[cfg(feature = "file-log")]
        file_log(follow);
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
            arg!(-J --journald "use journald log (default)"),
            arg!(-S --syslog "use syslog (default if built with no journald support"),
            arg!(-F --file "use file (default if built with no syslog support"),
            arg!(-i --"ignore-build-opts" "attempt logging regardless of build options"),
            arg!(-f --follow "output appended data as the log grows"),
        ])
        .group(
            ArgGroup::new("log")
                .args(vec!["journald", "syslog", "file"])
                .required(false),
        )
}

fn journald_log(follow: bool) {
    let flag = match follow {
        true => " -f",
        false => "",
    };
    match &mut Command::new("/bin/sh")
        .args([
            "-c",
            format!("journalctl{flag} | grep \"left[wh][mk].*\"").as_str(),
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

fn syslog(follow: bool) {
    let cmd = match follow {
        true => "tail -f",
        false => "cat",
    };
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
        let file_path = get_log_path();
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
