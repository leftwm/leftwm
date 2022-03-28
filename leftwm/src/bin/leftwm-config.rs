use anyhow::Result;
use clap::{App, Arg};
use leftwm::Config;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM Config")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Edit the config with the default editor")
        .arg(
            Arg::with_name("new")
                .short("n")
                .long("new")
                .help("Only generate a new config file"),
        )
        .get_matches();

    if matches.occurrences_of("new") >= 1 {
        generate_new_config()?;
    } else {
        run_editor()?;
    }

    Ok(())
}

fn find_config_file() -> Result<PathBuf> {
    let path = BaseDirectories::with_prefix("leftwm")?.place_config_file("config.toml")?;

    if !Path::new(&path).exists() {
        let config = Config::default();
        let toml = toml::to_string(&config)?;
        let mut file = File::create(&path)?;
        file.write_all(toml.as_bytes())?;
    }

    Ok(path)
}

//will not do anything if a config already exists
fn generate_new_config() -> Result<()> {
    let path = BaseDirectories::with_prefix("leftwm")?.place_config_file("config.toml")?;

    if Path::new(&path).exists() {
        println!(
            "\x1b[0;94m::\x1b[0m A config file already exists, do you want to override it? [y/N]"
        );
        let mut line = String::new();
        let _ = std::io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        if line.contains('y') || line.contains('Y') {
            let config = Config::default();
            let toml = toml::to_string(&config)?;
            let mut file = File::create(&path)?;
            file.write_all(toml.as_bytes())?;
        }
    }

    Ok(())
}

fn run_editor() -> Result<()> {
    let editor = env::var("EDITOR")?;
    let config_path = find_config_file()?
        .to_str()
        .expect("Couldn't find or create the config file")
        .to_string();

    let mut process = Command::new(&editor).arg(config_path).spawn()?;
    match process.wait()?.success() {
        true => Ok(()),
        false => Err(anyhow::Error::msg(format!("Failed to run {}", &editor))),
    }?;

    Ok(())
}
