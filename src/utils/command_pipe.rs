use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use xdg::BaseDirectories;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

type Queue = Arc<Mutex<VecDeque<ExternalCommand>>>;
type ResultQueue = Result<Queue>;

pub struct CommandPipe {
    queue: ResultQueue,
}

impl CommandPipe {
    pub fn new() -> CommandPipe {
        CommandPipe {
            queue: CommandPipe::build_listener(),
        }
    }

    pub fn read_command(&mut self) -> Option<ExternalCommand> {
        if let Ok(q) = &mut self.queue {
            let mut my_q = q.lock().unwrap();
            return my_q.pop_front();
        }
        None
    }

    fn build_listener() -> ResultQueue {
        let q: Queue = Arc::new(Mutex::new(VecDeque::new()));
        let base = BaseDirectories::with_prefix("leftwm")?;
        let pipe_file = base.place_runtime_file("commands.pipe")?;
        if !pipe_file.exists() {
            Command::new("mkfifo")
                .args(pipe_file.to_str())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .status()?;
        }
        let q2: Queue = q.clone();
        thread::spawn(move || loop {
            loop {
                let file = File::open(&pipe_file).unwrap();
                for line in BufReader::new(file).lines() {
                    if let Ok(l) = line {
                        if let Ok(cmd) = parse_command(l){
                            let mut my_q = q2.lock().unwrap();
                            my_q.push_back(cmd);
                        }
                    }
                }
            }
        });
        Ok(q)
    }

}

fn parse_command(s: String) -> std::result::Result<ExternalCommand, ()> {
    if s.starts_with("UnloadTheme") {
        return Ok(ExternalCommand::UnloadTheme);
    } else if s.starts_with("Reload") {
        return Ok(ExternalCommand::Reload);
    } else if s.starts_with("LoadTheme ") {
        return build_load_theme(s.clone());
    } else if s.starts_with("SendWorkspaceToTag ") {
        return build_goto_tag(s.clone());
    }
    Err(())
}

fn build_load_theme(mut raw: String) -> std::result::Result<ExternalCommand, ()> {
    crop_head(&mut raw, "LoadTheme ");
    let path = Path::new(&raw);
    if path.is_file() {
        Ok(ExternalCommand::LoadTheme(path.into()))
    } else {
        Err(())
    }
}

fn build_goto_tag(mut raw: String) -> std::result::Result<ExternalCommand, ()> {
    crop_head(&mut raw, "SendWorkspaceToTag ");
    let parts: Vec<&str> = raw.split(" ").collect();
    if parts.len() != 2 { return Err(()) }
    let ws_index = match parts[0].parse::<usize>() {
        Ok(x) => x,
        Err(_) => { return Err(()); }
    };
    let tag_index = match parts[1].parse::<usize>() {
        Ok(x) => x,
        Err(_) => { return Err(()); }
    };
    Ok( ExternalCommand::SendWorkspaceToTag(ws_index, tag_index) )
}

fn crop_head(s: &mut String, head: &str) {
    let pos = head.len();
    match s.char_indices().nth(pos) {
        Some((pos, _)) => {
            s.drain(..pos);
        }
        None => {
            s.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExternalCommand {
    LoadTheme(PathBuf),
    UnloadTheme,
    Reload,
    SendWorkspaceToTag(usize, usize),
}

