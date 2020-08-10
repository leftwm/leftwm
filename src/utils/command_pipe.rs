use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use x11_dl::xlib;
use xdg::BaseDirectories;

type Queue = Arc<Mutex<VecDeque<ExternalCommand>>>;
type ResultQueue = Result<Queue>;

pub struct CommandPipe {
    queue: ResultQueue,
}
impl Default for CommandPipe {
    fn default() -> Self {
        Self::new()
    }
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
            let mut xlib = xlib::Xlib::open().unwrap();
            let dpy = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
            assert!(!dpy.is_null(), "Null pointer in display");
            let root = unsafe { (xlib.XDefaultRootWindow)(dpy) };

            loop {
                let file = File::open(&pipe_file).unwrap();
                for line in BufReader::new(file).lines() {
                    if let Ok(l) = line {
                        if let Ok(cmd) = parse_command(l) {
                            let mut my_q = q2.lock().unwrap();
                            my_q.push_back(cmd);
                            create_unblocking_event(&mut xlib, dpy, root);
                        }
                    }
                }
            }
        });
        Ok(q)
    }
}

// the main event loop cannot process this external command until an event come in
// we ca generating a fake pointless event to unblock the event loop
fn create_unblocking_event(xlib: &mut xlib::Xlib, dpy: *mut xlib::Display, root: xlib::Window) {
    let mut current: xlib::Window = 0;
    let mut revert: i32 = 0;
    unsafe {
        (xlib.XGetInputFocus)(dpy, &mut current, &mut revert);
        (xlib.XSetInputFocus)(dpy, root, xlib::RevertToPointerRoot, xlib::CurrentTime);
        (xlib.XSetInputFocus)(dpy, current, revert, xlib::CurrentTime);
    }
}

fn parse_command(s: String) -> std::result::Result<ExternalCommand, ()> {
    if s.starts_with("UnloadTheme") {
        return Ok(ExternalCommand::UnloadTheme);
    } else if s.starts_with("Reload") {
        return Ok(ExternalCommand::Reload);
    } else if s.starts_with("LoadTheme ") {
        return build_load_theme(s);
    } else if s.starts_with("SendWorkspaceToTag ") {
        return build_send_workspace_to_tag(s);
    } else if s.starts_with("SendWindowToTag ") {
        return build_send_window_to_tag(s);
    } else if s.starts_with("SwapScreens") {
        return Ok(ExternalCommand::SwapScreens);
    } else if s.starts_with("MoveWindowToLastWorkspace") {
        return Ok(ExternalCommand::MoveWindowToLastWorkspace);
    } else if s.starts_with("MoveWindowUp") {
        return Ok(ExternalCommand::MoveWindowUp);
    } else if s.starts_with("MoveWindowDown") {
        return Ok(ExternalCommand::MoveWindowDown);
    } else if s.starts_with("FocusWindowUp") {
        return Ok(ExternalCommand::FocusWindowUp);
    } else if s.starts_with("FocusWindowDown") {
        return Ok(ExternalCommand::FocusWindowDown);
    } else if s.starts_with("FocusNextTag") {
        return Ok(ExternalCommand::FocusNextTag);
    } else if s.starts_with("FocusPreviousTag") {
        return Ok(ExternalCommand::FocusPreviousTag);
    } else if s.starts_with("FocusWorkspaceNext") {
        return Ok(ExternalCommand::FocusWorkspaceNext);
    } else if s.starts_with("FocusWorkspacePrevious") {
        return Ok(ExternalCommand::FocusWorkspacePrevious);
    } else if s.starts_with("NextLayout") {
        return Ok(ExternalCommand::NextLayout);
    } else if s.starts_with("PreviousLayout") {
        return Ok(ExternalCommand::PreviousLayout);
    } else if s.starts_with("CloseWindow") {
        return Ok(ExternalCommand::CloseWindow);
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

fn build_send_window_to_tag(mut raw: String) -> std::result::Result<ExternalCommand, ()> {
    crop_head(&mut raw, "SendWindowToTag ");
    let parts: Vec<&str> = raw.split(' ').collect();
    if parts.len() != 1 {
        return Err(());
    }
    let tag_index = match parts[0].parse::<usize>() {
        Ok(x) => x,
        Err(_) => {
            return Err(());
        }
    };
    Ok(ExternalCommand::SendWindowToTag(tag_index))
}

fn build_send_workspace_to_tag(mut raw: String) -> std::result::Result<ExternalCommand, ()> {
    crop_head(&mut raw, "SendWorkspaceToTag ");
    let parts: Vec<&str> = raw.split(' ').collect();
    if parts.len() != 2 {
        return Err(());
    }
    let ws_index = match parts[0].parse::<usize>() {
        Ok(x) => x,
        Err(_) => {
            return Err(());
        }
    };
    let tag_index = match parts[1].parse::<usize>() {
        Ok(x) => x,
        Err(_) => {
            return Err(());
        }
    };
    Ok(ExternalCommand::SendWorkspaceToTag(ws_index, tag_index))
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
    SendWindowToTag(usize),
    SwapScreens,
    MoveWindowToLastWorkspace,
    MoveWindowUp,
    MoveWindowDown,
    FocusWindowUp,
    FocusWindowDown,
    FocusNextTag,
    FocusPreviousTag,
    FocusWorkspaceNext,
    FocusWorkspacePrevious,
    CloseWindow,
    NextLayout,
    PreviousLayout,
}
