use clap::{value_t, App, Arg};
use leftwm::errors::Result;
use leftwm::models::dto::{DisplayState, ManagerState};
use std::path::Path;
use std::str;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::UnixStream;
use xdg::BaseDirectories;

type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM State")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("prints out the current state of LeftWM")
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .value_name("FILE")
                .help("A liquid template to use for the output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("string")
                .short("s")
                .long("string")
                .value_name("STRING")
                .help("Use a liquid template string literal to use for the output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("workspace")
                .short("w")
                .long("workspace")
                .value_name("WS_NUM")
                .help("render only info about a given workspace [0..]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("newline")
                .short("n")
                .long("newline")
                .help("Print new lines in the output"),
        )
        .arg(
            Arg::with_name("quit")
                .short("q")
                .long("quit")
                .help("Prints the state once and quits"),
        )
        .get_matches();

    let template_file = matches.value_of("template");

    let string_literal = matches.value_of("string");

    let ws_num: Option<usize> = match value_t!(matches, "workspace", usize) {
        Ok(x) => Some(x),
        Err(_) => None,
    };

    let mut stream_reader = stream_reader().await?;
    let once = matches.occurrences_of("quit") == 1;
    let newline = matches.occurrences_of("newline") == 1;

    if let Some(template_file) = template_file {
        let path = Path::new(template_file);
        let partials = get_partials(path.parent()).await?;
        println!("{:?}", partials);
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
            let mut entries = fs::read_dir(d).await?;
            let mut partial_paths = vec![];
            while let Some(entry) = entries.next_entry().await? {
                let f_n = entry.file_name();
                let f_n_str = f_n.to_str().unwrap_or(" ");
                if f_n_str.starts_with('_') && f_n_str.ends_with(".liquid") {
                    partial_paths.push(entry)
                }
            }
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
        //liquid only does time in utc. BUG: https://github.com/cobalt-org/liquid-rust/issues/332
        //as a workaround we are setting a time locally
        globals.insert(
            "localtime".into(),
            liquid::model::Value::scalar(get_localtime()),
        );
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

fn get_localtime() -> String {
    let now = chrono::Local::now();
    now.format("%m/%d/%Y %l:%M %p").to_string()
}

async fn stream_reader() -> Result<Lines<BufReader<UnixStream>>> {
    let base = BaseDirectories::with_prefix("leftwm")?;
    let socket_file = base.place_runtime_file("current_state.sock")?;
    let stream = UnixStream::connect(socket_file).await?;
    Ok(BufReader::new(stream).lines())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use escargot::CargoBuild;
    use predicates::prelude::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    const UNKNOWN_PARTIAL_ERR: &str = r#"thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Error { inner: InnerError { msg: Singleton("Unknown partial-template"#;

    #[test]
    fn template_flag_partials() -> Result<()> {
        // several tests included in one to avoid writing files mult times

        let temp_dir = tempdir()?;
        let main_template_name = "temp.liquid";
        let main_template_path = temp_dir.path().join(main_template_name);

        let partial_template_name = "_partial.liquid";
        let partial_template_path = temp_dir.path().join(partial_template_name);

        // created the path but not the file yet
        assert!(!std::path::Path::new(&partial_template_path).exists());

        let mut main_template_file = File::create(&main_template_path)?;
        let main_template_content = format!(
            "{{% include \"{}\" %}}",
            &partial_template_path.to_str().unwrap()
        );
        main_template_file.write_all(main_template_content.as_bytes())?;
        let bin_for_test = CargoBuild::new()
            .bin("leftwm-state")
            .current_release()
            .current_target()
            .run()
            .unwrap();

        let mut cmd = bin_for_test.command();
        cmd.arg("-t").arg(&main_template_path).arg("-q");

        // run the command before creating partial file should give err:
        cmd.assert()
            .stderr(predicate::str::starts_with(UNKNOWN_PARTIAL_ERR))
            .failure();

        let mut partial_template_file = File::create(&partial_template_path)?;
        partial_template_file.write_all(b"This is the partial content.")?;

        // created the partial file
        assert!(std::path::Path::new(&partial_template_path).exists());

        // check that partial file gets registered and used
        cmd.assert()
            .stdout(predicate::str::contains(
                partial_template_path.to_str().unwrap(),
            ))
            .stdout(predicate::str::contains("This is the partial content."))
            .success();

        drop(main_template_file);
        drop(partial_template_file);
        temp_dir.close()?;
        Ok(())
    }
}
