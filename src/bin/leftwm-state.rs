use clap::{value_t, App, Arg};
use leftwm::errors::Result;
use leftwm::models::dto::*;
use std::fs;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::str;
use xdg::BaseDirectories;

fn main() -> Result<()> {
    let matches = App::new("LeftWM State")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version("0.2.3")
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

    let mut stream_reader = StreamReader::new().unwrap();
    let once = matches.occurrences_of("quit") == 1;
    let newline = matches.occurrences_of("newline") == 1;

    if template_file.is_some() {
        let template_str = fs::read_to_string(template_file.unwrap()).unwrap();
        let template = liquid::ParserBuilder::with_liquid()
            .build()
            .unwrap()
            .parse(&template_str)
            .unwrap();
        while let Ok(line) = stream_reader.read() {
            let _ = template_handler(&template, newline, ws_num, line);
            if once {
                break;
            }
        }
    } else {
        while let Ok(line) = stream_reader.read() {
            let _ = raw_handler(line);
            if once {
                break;
            }
        }
    }

    Ok(())
}

fn raw_handler(line: String) -> Result<()> {
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
    line: String,
) -> Result<()> {
    let s: ManagerState = serde_json::from_str(&line)?;
    let display: DisplayState = s.into();

    let globals = match ws_num {
        Some(ws_num) => {
            let json = serde_json::to_string(&display.workspaces[ws_num])?;
            let workspace: liquid::value::Object = serde_json::from_str(&json)?;
            let mut globals = liquid::value::Object::new();
            globals.insert(
                "window_title".into(),
                liquid::value::Value::scalar(display.window_title),
            );
            globals.insert("workspace".into(), liquid::value::Value::Object(workspace));
            //liquid only does time in utc. BUG: https://github.com/cobalt-org/liquid-rust/issues/332
            //as a workaround we are setting a time locally
            globals.insert(
                "localtime".into(),
                liquid::value::Value::scalar(get_localtime()),
            );
            globals
        }
        None => {
            let json = serde_json::to_string(&display)?;
            let globals: liquid::value::Object = serde_json::from_str(&json)?;
            globals
        }
    };

    let mut output = template.render(&globals).unwrap();
    output = str::replace(&output, "\r", "");
    if !newline {
        output = str::replace(&output, "\n", "");
        println!("{}", output);
    } else {
        print!("{}", output);
    }
    Ok(())
}

fn get_localtime() -> String {
    let now = chrono::Local::now();
    now.format("%m/%d/%Y %l:%M %p").to_string()
}

struct StreamReader {
    stream: UnixStream,
    buffer: [u8; 4096],
}

impl StreamReader {
    fn new() -> Result<StreamReader> {
        let base = BaseDirectories::with_prefix("leftwm")?;
        let socket_file = base.place_runtime_file("current_state.sock")?;
        let stream = UnixStream::connect(socket_file)?;
        let buffer = [0; 4096];
        Ok(StreamReader { buffer, stream })
    }

    fn read(&mut self) -> Result<String> {
        let size = self.stream.read(&mut self.buffer)?;
        let raw = str::from_utf8(&self.buffer[0..size]).unwrap();
        if let Some(raw) = raw.lines().last() {
            Ok(raw.to_string())
        } else {
            Err(leftwm::errors::LeftErrorKind::StreamError().into())
        }
    }
}
