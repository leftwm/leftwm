use clap::{App, Arg};
use leftwm::errors::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM Command")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Sends external commands to LeftWM")
        .arg(
            Arg::with_name("command")
                .help("The command to be sent.")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let file_path = BaseDirectories::with_prefix("leftwm")?
        .find_runtime_file("commands.pipe")
        .expect("ERROR: Couldn't find commands.pipe");
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("ERROR: Couldn't open commands.pipe");
    if let Some(commands) = matches.values_of("command") {
        for command in commands {
            if let Err(e) = writeln!(file, "{}", command) {
                eprintln!(" ERROR: Couldn't write to commands.pipe: {}", e);
            }
        }
    }
    Ok(())
}
