use crate::errors::{LeftError, Result};
use crate::models::Handle;
use crate::models::dto::ManagerState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
struct State {
    peers: Vec<Option<UnixStream>>,
    last_state: String,
}

#[derive(Debug, Default)]
pub struct StateSocket {
    state: Arc<Mutex<State>>,
    listener: Option<tokio::task::JoinHandle<()>>,
    socket_file: PathBuf,
}

impl Drop for StateSocket {
    fn drop(&mut self) {
        assert!(
            std::thread::panicking() || self.listener.is_none(),
            "StateSocket has to be shutdown explicitly before drop"
        );
    }
}

impl StateSocket {
    /// Bind to Unix socket and listen.
    /// # Errors
    ///
    /// Will error if `build_listener()` cannot be unwrapped or awaited.
    /// As in `build_listener()`, this is likely a filesystem issue,
    /// such as incorrect permissions or a non-existant file.
    pub async fn listen(&mut self, socket_file: PathBuf) -> Result<()> {
        self.socket_file = socket_file;
        let listener = self.build_listener().await?;
        self.listener = Some(listener);
        Ok(())
    }

    /// Explicitly shutdown `StateSocket` to perform cleanup.
    pub async fn shutdown(&mut self) {
        if let Some(listener) = self.listener.take() {
            listener.abort();
            listener.await.ok();
            fs::remove_file(self.socket_file.as_path()).await.ok();
        }
    }

    /// # Errors
    /// Will return Err if a mut ref to the peer is unavailable.
    /// Will return error if state cannot be serialized
    pub async fn write_manager_state<H: Handle>(&mut self, raw_state: &crate::state::State<H>) -> Result<()> {
        if self.listener.is_some() {
            let state: ManagerState = raw_state.into();
            let mut json = serde_json::to_string(&state)?;
            json.push('\n');
            let mut state = self.state.lock().await;

            let state_changed = json != state.last_state;
            if state_changed {
                state.peers.retain(std::option::Option::is_some);
                for peer in &mut state.peers {
                    if peer
                        .as_mut()
                        .ok_or(LeftError::StreamError)?
                        .write_all(json.as_bytes())
                        .await
                        .is_err()
                    {
                        peer.take();
                    }
                }
                state.last_state = json;
            }
        }
        Ok(())
    }

    async fn build_listener(&self) -> Result<tokio::task::JoinHandle<()>> {
        let state = self.state.clone();
        let listener = if let Ok(m) = UnixListener::bind(&self.socket_file) {
            m
        } else {
            fs::remove_file(&self.socket_file).await?;
            UnixListener::bind(&self.socket_file)?
        };

        Ok(tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut peer, _)) => {
                        let mut state = state.lock().await;
                        if peer.write_all(state.last_state.as_bytes()).await.is_ok() {
                            state.peers.push(Some(peer));
                        }
                    }
                    Err(e) => tracing::error!("Accept failed = {:?}", e),
                }
            }
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::helpers::test::temp_path;
    use crate::Manager;
    use tokio::io::{AsyncBufReadExt, BufReader};

    #[tokio::test]
    async fn multiple_peers() {
        let manager = Manager::new_test(vec![]);
        let state = &manager.state;

        let socket_file = temp_path().await.unwrap();
        let mut state_socket = StateSocket::default();
        state_socket.listen(socket_file.clone()).await.unwrap();
        state_socket.write_manager_state(state).await.unwrap();

        assert_eq!(
            serde_json::to_string(&Into::<ManagerState>::into(state)).unwrap(),
            BufReader::new(UnixStream::connect(socket_file.clone()).await.unwrap())
                .lines()
                .next_line()
                .await
                .expect("Read next line")
                .unwrap()
        );

        assert_eq!(
            serde_json::to_string(&Into::<ManagerState>::into(state)).unwrap(),
            BufReader::new(UnixStream::connect(socket_file.clone()).await.unwrap())
                .lines()
                .next_line()
                .await
                .expect("Read next line")
                .unwrap()
        );

        assert_eq!(
            serde_json::to_string(&Into::<ManagerState>::into(state)).unwrap(),
            BufReader::new(UnixStream::connect(socket_file).await.unwrap())
                .lines()
                .next_line()
                .await
                .expect("Read next line")
                .unwrap()
        );

        state_socket.shutdown().await;
    }

    #[tokio::test]
    async fn get_update() {
        let manager = Manager::new_test(vec![]);
        let state = &manager.state;

        let socket_file = temp_path().await.unwrap();
        let mut state_socket = StateSocket::default();
        state_socket.listen(socket_file.clone()).await.unwrap();
        state_socket.write_manager_state(state).await.unwrap();

        let mut lines = BufReader::new(UnixStream::connect(socket_file).await.unwrap()).lines();

        assert_eq!(
            serde_json::to_string(&Into::<ManagerState>::into(state)).unwrap(),
            lines.next_line().await.expect("Read next line").unwrap()
        );

        // Fake state update.
        state_socket.state.lock().await.last_state = String::default();
        state_socket.write_manager_state(state).await.unwrap();

        assert_eq!(
            serde_json::to_string(&Into::<ManagerState>::into(state)).unwrap(),
            lines.next_line().await.expect("Read next line").unwrap()
        );

        state_socket.shutdown().await;
    }

    #[tokio::test]
    async fn socket_cleanup() {
        let socket_file = temp_path().await.unwrap();
        let mut state_socket = StateSocket::default();
        state_socket.listen(socket_file.clone()).await.unwrap();
        state_socket.shutdown().await;
        assert!(!socket_file.exists());
    }

    #[tokio::test]
    async fn socket_already_bound() {
        let socket_file = temp_path().await.unwrap();
        let mut old_socket = StateSocket::default();
        old_socket.listen(socket_file.clone()).await.unwrap();
        assert!(socket_file.exists());
        let mut state_socket = StateSocket::default();
        state_socket.listen(socket_file.clone()).await.unwrap();
        state_socket.shutdown().await;
        assert!(!socket_file.exists());
        old_socket.shutdown().await;
    }
}
