use crate::models::Manager;
use crate::models::dto::*;
use bytes::{BufMut, BytesMut};
use futures::future::{self, Either};
use futures::sync::mpsc;
use futures::{Future, Stream};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::io;
use tokio::prelude::*;
use tokio_uds::{UnixListener, UnixStream};

use uuid::Uuid;
use xdg::BaseDirectories;

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;
type Server = Result<Arc<Mutex<Shared>>, Box<std::error::Error>>;

pub struct StateSocket {
    server: Server,
    last_state: Arc<Mutex<String>>, //last_state: String
}

struct Shared {
    peers: HashMap<Uuid, Tx>,
}
impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }
}

impl Default for StateSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl StateSocket {
    pub fn new() -> StateSocket {
        let last_state = Arc::new(Mutex::new("".to_owned()));
        StateSocket {
            server: StateSocket::build_listener(last_state.clone()),
            last_state,
        }
    }

    fn build_listener(last_state: Arc<Mutex<String>>) -> Server {
        let base = BaseDirectories::with_prefix("leftwm")?;
        let socket_file = base.place_runtime_file("current_state.sock")?;
        let state = Arc::new(Mutex::new(Shared::new()));
        let return_state = state.clone();
        thread::spawn(move || loop {
            let thread_state = state.clone();
            let thread_last_state = last_state.clone();
            let listener = match UnixListener::bind(&socket_file) {
                Ok(m) => m,
                Err(_) => {
                    fs::remove_file(&socket_file).unwrap();
                    UnixListener::bind(&socket_file).unwrap()
                }
            };
            let server = listener
                .incoming()
                .map_err(|e| eprintln!("accept failed = {:?}", e))
                .for_each(move |sock| {
                    process(sock, thread_state.clone(), thread_last_state.clone());
                    Ok(())
                });
            tokio::run(server);
        });
        Ok(return_state)
    }

    pub fn write_manager_state(&mut self, manager: &Manager) -> Result<(), Box<std::error::Error>> {
        let state: ManagerState = manager.into();
        let mut json = serde_json::to_string(&state)?;
        json.push_str("\n");
        let mut lc = self.last_state.lock().unwrap();
        if json != *lc {
            if let Ok(server) = &self.server {
                for (_, tx) in server.lock().unwrap().peers.iter() {
                    tx.unbounded_send(json.clone()).unwrap();
                }
            }
            *lc = json;
        }
        Ok(())
    }
}

struct Lines {
    socket: UnixStream,
    wr: BytesMut,
}

impl Lines {
    fn new(socket: UnixStream) -> Self {
        Lines {
            socket,
            wr: BytesMut::new(),
        }
    }

    fn buffer(&mut self, line: &[u8]) {
        self.wr.reserve(line.len());
        self.wr.put(line);
    }

    fn poll_flush(&mut self) -> Poll<(), io::Error> {
        while !self.wr.is_empty() {
            let n = try_ready!(self.socket.poll_write(&self.wr));
            assert!(n > 0);
            let _ = self.wr.split_to(n);
        }
        Ok(Async::Ready(()))
    }
}

impl Stream for Lines {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<()>, Self::Error> {
        Ok(Async::Ready(Some(())))
    }
}

fn process(socket: UnixStream, state: Arc<Mutex<Shared>>, last_state: Arc<Mutex<String>>) {
    let lines = Lines::new(socket);

    let connection = lines
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(|(name, lines)| {
            match name {
                Some(_) => {}
                None => {
                    return Either::A(future::ok(()));
                }
            };
            let peer = Peer::new(state, lines, last_state);
            Either::B(peer)
        })
        .map_err(|_e| {});
    tokio::spawn(connection);
}

struct Peer {
    lines: Lines,
    state: Arc<Mutex<Shared>>,
    rx: Rx,
    id: Uuid,
}

impl Peer {
    fn new(state: Arc<Mutex<Shared>>, lines: Lines, last_state: Arc<Mutex<String>>) -> Peer {
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::unbounded();
        let json = last_state.lock().unwrap().clone();
        tx.unbounded_send(json).unwrap();
        state.lock().unwrap().peers.insert(id, tx);
        Peer {
            lines,
            state,
            rx,
            id,
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.state.lock().unwrap().peers.remove(&self.id);
    }
}

impl Future for Peer {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<(), io::Error> {
        //while there are lines to read
        while let Async::Ready(Some(s)) = self.rx.poll().unwrap() {
            self.lines.buffer(&s.as_bytes());
        }
        let _ = self.lines.poll_flush();
        Ok(Async::NotReady)
    }
}
