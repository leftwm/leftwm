use anyhow::{Context, Result};
use clap::{arg, command};
use leftwm::BaseCommand;
use leftwm_core::ReturnPipe;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = get_command().get_matches();

    let file_name = leftwm_core::pipe_name();
    let file_path = BaseDirectories::with_prefix("leftwm")?
        .find_runtime_file(&file_name)
        .with_context(|| format!("ERROR: Couldn't find {}", file_name.display()))?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .with_context(|| format!("ERROR: Couldn't open {}", file_name.display()))?;
    let mut exit_code = 0;
    if let Some(commands) = matches.get_many::<String>("COMMAND") {
        let mut ret_pipe = get_return_pipe().await?;
        for command in commands {
            if let Err(e) = writeln!(file, "{command}") {
                eprintln!("ERROR: Couldn't write to commands.pipe: {e}");
                continue;
            }
            tokio::select! {
                Some(res) = ret_pipe.read_return() => {
                    if let Some((result, msg)) = res.split_once(' ') {
                        match result {
                            "OK:" => println!("{command}: {msg}"),
                            "ERROR:" => {eprintln!("{command}: {msg}"); exit_code = 1;},
                            _ => println!("{command}: {res}"),
                        }
                    } else {
                        println!("{command}: {res}");
                    }
                }
                () = timeout(5000) => {eprintln!("WARN: timeout connecting to return pipe. Command may have executed, but errors will not be displayed."); exit_code = 1;}
                else => {eprintln!("WARN: timeout connection to return pipe. Command may have executed, but errors will not be displayed."); exit_code = 1;}
            }
        }
        drop(ret_pipe);
    }

    if matches.get_flag("list") {
        print_commandlist();
    }
    exit(exit_code);
}

fn get_command() -> clap::Command {
    command!("LeftWM Command")
        .about("Sends external commands to LeftWM. After executing a command, errors will be logged to both stderr and to the log (see leftwm-log for more details)")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-l --list "Print a list of available commands with their arguments."),
            arg!([COMMAND] ... "The command to be sent. See 'list' flag."),
        ])
}

fn print_commandlist() {
    println!(
        "\
Available Commands:{}
    SendWorkspaceToTag
        Args: <workspace_index> <tag_index> (int)
    SendWindowToTag
        Args: <tag_index> (int)

Note about commands with arguments:
    Use quotations for the command and arguments, like this:
    leftwm-command \"<command> <args>\"
For more information please visit:
https://github.com/leftwm/leftwm/wiki/External-Commands\
",
        BaseCommand::documentation().replace('\n', "\n    ")
    );
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    #[error("Couldn't create the file: '{0}'")]
    CreateFile(PathBuf),

    #[error("Couldn't connect to file: '{0}'")]
    ConnectToFile(PathBuf),
}

async fn get_return_pipe() -> Result<ReturnPipe, Error> {
    let file_name = ReturnPipe::pipe_name();

    let pipe_file =
        place_runtime_file(&file_name).map_err(|_| Error::CreateFile(file_name.clone()))?;

    ReturnPipe::new(pipe_file)
        .await
        .map_err(|_| Error::ConnectToFile(file_name))
}

fn place_runtime_file<P>(path: P) -> std::io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    xdg::BaseDirectories::with_prefix("leftwm")?.place_runtime_file(path)
}

async fn timeout(mills: u64) {
    use tokio::time::{sleep, Duration};
    sleep(Duration::from_millis(mills)).await;
}
