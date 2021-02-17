use clap::{App, Arg};
use leftwm::config::Config;
use leftwm::errors::Result;
use std::fs;
use std::path::Path;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM Check")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("checks syntax of the configuration file")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use. Uses first in PATH otherwise.")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Outputs received configuration file."),
        )
        .get_matches();

    let config_file = matches.value_of("INPUT");
    let verbose = matches.occurrences_of("verbose") >= 1;

    dbg!(config_file);
    //    use leftwm::config::*;

    pub fn load_from_file() -> Result<Config> {
        let path = BaseDirectories::with_prefix("leftwm")?;
        let config_filename = path.place_config_file("config.toml")?;
        if Path::new(&config_filename).exists() {
            let contents = fs::read_to_string(config_filename)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Err(leftwm::errors::LeftError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Configuration not found in path",
            )))
        }
    }

    match load_from_file() {
        Ok(config) => {
            println!("Configuration loaded successfully");
            if verbose {
                dbg!(config);
            }
        }
        Err(e) => {
            println!("Configuration failed. Reason: {:?}", e);
        }
    }

    Ok(())
}
