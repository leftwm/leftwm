use clap::{value_t, App, Arg};
use leftwm::errors::Result;
use leftwm::models::dto::{DisplayState, ManagerState};
use std::str;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::UnixStream;
use xdg::BaseDirectories;

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

    let ws_num: Option<usize> = match value_t!(matches, "workspace", usize) {
        Ok(x) => Some(x),
        Err(_) => None,
    };

    let mut stream_reader = stream_reader().await?;
    let once = matches.occurrences_of("quit") == 1;
    let newline = matches.occurrences_of("newline") == 1;

    if let Some(template_file) = template_file {
        let template_str = fs::read_to_string(template_file).await?;
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(&template_str)
            .unwrap();
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

fn raw_handler(line: &str) -> Result<()> {
    let s: ManagerState = serde_json::from_str(&line)?;
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
    let s: ManagerState = serde_json::from_str(&line)?;
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
    if !newline {
        output = str::replace(&output, "\n", "");
    }
    print!("{}", output);
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
