use clap::{arg, command};
use leftwm_core::errors::Result;
use leftwm_core::models::dto::{DisplayState, ManagerState};
use std::ffi::OsStr;
use std::path::Path;
use std::str;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::UnixStream;
use xdg::BaseDirectories;

type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = get_command().get_matches();

    let template_file = matches.get_one::<String>("template");
    let string_literal = matches.get_one::<String>("string");
    let ws_num = matches.get_one("workspace").copied();
    let newline = matches.get_flag("newline");
    let once = matches.get_flag("quit");

    let mut stream_reader = stream_reader().await?;
    if let Some(template_file) = template_file {
        let path = Path::new(template_file);
        let partials = get_partials(path.parent()).await?;
        let template_str = fs::read_to_string(template_file).await?;
        let template = liquid::ParserBuilder::with_stdlib()
            .partials(partials)
            .build()
            .expect("Unable to build template")
            .parse(&template_str)
            .expect("Unable to parse template");
        while let Some(line) = stream_reader.next_line().await? {
            let _droppable = template_handler(&template, newline, ws_num, &line);
            if once {
                break;
            }
        }
    } else if let Some(string_literal) = string_literal {
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .expect("Unable to build template")
            .parse(string_literal)
            .expect("Unable to parse template");
        while let Some(line) = stream_reader.next_line().await? {
            let _droppable = template_handler(&template, newline, ws_num, &line);
            if once {
                break;
            }
        }
    } else {
        while let Some(line) = stream_reader.next_line().await? {
            let _droppable2 = raw_handler(&line);
            if once {
                break;
            }
        }
    }

    Ok(())
}

async fn get_partials(dir: Option<&Path>) -> Result<Partials> {
    let mut partials = Partials::empty();
    match dir {
        Some(d) => {
            let entries = fs::read_dir(d).await?;

            let partial_paths = partials_in_dir_entries(entries).await?;

            for path in partial_paths {
                partials.add(
                    path.path().as_path().to_str().unwrap_or(" "),
                    fs::read_to_string(path.path()).await?,
                );
            }
            Ok(partials)
        }
        None => Ok(partials),
    }
}

async fn partials_in_dir_entries(mut entries: fs::ReadDir) -> Result<Vec<fs::DirEntry>> {
    let mut partial_paths = vec![];
    while let Some(entry) = entries.next_entry().await? {
        let f_n = entry.file_name();
        if is_partial_filename(&f_n) {
            partial_paths.push(entry);
        }
    }
    Ok(partial_paths)
}

fn is_partial_filename(filename: &OsStr) -> bool {
    let f_n = filename.to_str().unwrap_or(" ");
    f_n.starts_with('_') && f_n.ends_with(".liquid")
}

fn raw_handler(line: &str) -> Result<()> {
    let s: ManagerState = serde_json::from_str(line)?;
    let display: DisplayState = s.into();
    let json = serde_json::to_string(&display)?;
    println!("{}", json);
    Ok(())
}

fn template_handler(
    template: &liquid::Template,
    newline: bool,
    ws_num: Option<usize>,
    line: &str,
) -> Result<()> {
    let s: ManagerState = serde_json::from_str(line)?;
    let display: DisplayState = s.into();

    let globals = if let Some(ws_num) = ws_num {
        let json = serde_json::to_string(&display.workspaces[ws_num])?;
        let workspace: liquid::model::Object = serde_json::from_str(&json)?;
        let mut globals = liquid::model::Object::new();
        globals.insert(
            "window_title".into(),
            liquid::model::Value::scalar(display.window_title),
        );
        globals.insert("workspace".into(), liquid::model::Value::Object(workspace));
        globals
    } else {
        let json = serde_json::to_string(&display)?;
        let globals: liquid::model::Object = serde_json::from_str(&json)?;
        globals
    };

    let mut output = template.render(&globals).unwrap();
    output = str::replace(&output, "\r", "");
    // We use newline rather than !newline to avoid negative logic,
    // but note the difference between print! and println!. Trying to skip println!
    // will result in theme degradation, as in #263.
    if newline {
        print!("{}", output);
    } else {
        output = str::replace(&output, "\n", "");
        println!("{}", output);
    }
    Ok(())
}

async fn stream_reader() -> Result<Lines<BufReader<UnixStream>>> {
    let base = BaseDirectories::with_prefix("leftwm")?;
    let socket_file = base.place_runtime_file("current_state.sock")?;
    let stream = UnixStream::connect(socket_file).await?;
    Ok(BufReader::new(stream).lines())
}

fn get_command() -> clap::Command {
    command!("LeftWM State")
        .about("Prints out the current state of LeftWM")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-t --template [FILE] "A liquid template to use for the output"),
            arg!(-s --string [STRING] "Use a liquid template string literal to use for the output"),
            arg!(-w --workspace [WS_NUM] "render only info about a given workspace [0..]")
                .value_parser(clap::value_parser!(usize)),
            arg!(-n --newline "Print new lines in the output"),
            arg!(-q --quit "Prints the state once and quits"),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_identifies_partial_template_filenames() {
        let file_names = vec![
            "main.liquid",
            "_partial.liquid",
            "\u{7c0}nonascii-in-filename.liquid", // first char U07C0
            "1_partial.liquid",
            "_liquid.txt",
        ];
        let partials = file_names
            .iter()
            .map(|f_n| OsStr::new(*f_n))
            .filter(|f_n| is_partial_filename(f_n))
            .collect::<Vec<&OsStr>>();

        assert!(partials == vec![OsStr::new("_partial.liquid")]);
    }
}
