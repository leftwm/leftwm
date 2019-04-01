use crate::models::Manager;
use std::fs;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use xdg::BaseDirectories;

type StateStream = Result<Sender<ManagerState>, Box<std::error::Error>>;
pub struct StateSocket {
    state_stream: StateStream,
}

impl StateSocket {
    pub fn new() -> StateSocket {
        StateSocket {
            state_stream: StateSocket::build_listener(),
        }
    }

    fn build_listener() -> StateStream {
        let base = BaseDirectories::with_prefix("leftwm")?;
        let socket_file = base.place_runtime_file("current_state.sock")?;
        let (tx, mut rx): (Sender<ManagerState>, Receiver<ManagerState>) = mpsc::channel();
        let listener = match UnixListener::bind(&socket_file) {
            Ok(m) => m,
            Err(_) => {
                fs::remove_file(&socket_file).unwrap();
                UnixListener::bind(&socket_file).unwrap()
            }
        };
        thread::spawn(move || loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    let _ = socket_writer(socket, &mut rx);
                }
                Err(_) => {}
            }
        });
        Ok(tx)
    }

    pub fn write_manager_state(&mut self, manager: &Manager) {
        let state: ManagerState = manager.into();
        if let Ok(stream) = &self.state_stream {
            let _ = stream.send(state);
        }
    }
}

fn socket_writer(
    mut stream: UnixStream,
    rx: &mut Receiver<ManagerState>,
) -> Result<(), Box<std::error::Error>> {
    loop {
        let state: ManagerState = rx.recv()?;
        let mut json = serde_json::to_string(&state)?;
        json.push_str("\n");
        stream.write_all(json.as_bytes())?;
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ManagerState {
    pub window_title: Option<String>,
    pub desktop_names: Vec<String>,
    pub viewports: Vec<String>,
    pub active_desktop: Vec<String>,
}
impl From<&Manager> for ManagerState {
    fn from(manager: &Manager) -> Self {
        let mut viewports = vec![];
        for ws in &manager.workspaces {
            viewports.extend_from_slice(&ws.tags);
        }
        let active_desktop = match manager.focused_workspace() {
            Some(ws) => ws.tags.clone(),
            None => vec!["".to_owned()],
        };
        let window_title = match manager.focused_window() {
            Some(win) => win.name.clone(),
            None => None,
        };
        ManagerState {
            window_title,
            desktop_names: manager.tags.clone(),
            viewports,
            active_desktop,
        }
    }
}
