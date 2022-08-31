use crate::{child_process::Nanny, config::Config};
use crate::{CommandPipe, DisplayEvent, DisplayServer, Manager, Mode, StateSocket, Window, Command};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::atomic::Ordering;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    #[error("Couldn't create the file: '{0}'")]
    CreateFile(String),

    #[error("Couldn't connect to file: '{0}'")]
    ConnectToFile(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EventResponse {
    None,
    DisplayRefreshNeeded,
}

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    pub async fn event_loop(mut self) -> Result<(), Error> {
        let mut state_socket = get_state_socket().await?;
        let mut command_pipe = get_command_pipe().await?;

        // Start the current theme.
        let after_first_loop: Once = Once::new();

        // Main event loop.
        let mut event_buffer = vec![];
        while self.should_keep_running(&mut state_socket).await {
            self.update_manager_state(&mut state_socket).await;
            self.display_server.flush(); // is that needed?

            let response: EventResponse = tokio::select! {
                _ = self.display_server.wait_readable(), if event_buffer.is_empty() => {
                    event_buffer.append(&mut self.display_server.get_next_events());
                    EventResponse::None
                },
                // When a mouse button is pressed or enter/motion notifies are blocked and only appear
                // once the button is released. This is to double check that we know which window
                // is currently focused.
                // _ = timeout(100), if event_buffer.is_empty()
                //     && self.state.focus_manager.sloppy_mouse_follows_focus
                //     && self.state.focus_manager.behaviour.is_sloppy() => {
                //     if let Some(verify_event) = self.display_server.generate_verify_focus_event() {
                //         event_buffer.push(verify_event);
                //     }
                //     EventResponse::None
                // },
                Some(cmd) = command_pipe.read_command(), if event_buffer.is_empty() => self.execute_command(&cmd),
                else => self.execute_display_events(&mut event_buffer),
            };

            match response {
                EventResponse::None => (),
                EventResponse::DisplayRefreshNeeded => self.refresh_display(),
            };

            // Perform any actions requested by the handler.
            while !self.state.actions.is_empty() {
                if let Some(act) = self.state.actions.pop_front() {
                    if let Some(event) = self.display_server.execute_action(act) {
                        event_buffer.push(event);
                    }
                }
            }

            // After the very first loop run the 'up' scripts (global and theme). As we need the unix
            // socket to already exist.
            after_first_loop.call_once(|| {
                match Nanny::run_global_up_script() {
                    Ok(child) => {
                        child.map(|child| self.children.insert(child));
                    }
                    Err(err) => log::error!("Global up script faild: {}", err),
                }
                match Nanny::boot_current_theme() {
                    Ok(child) => {
                        child.map(|child| self.children.insert(child));
                    }
                    Err(err) => log::error!("Theme loading failed: {}", err),
                }

                self.config.load_state(&mut self.state);
            });

            if self.reap_requested.swap(false, Ordering::SeqCst) {
                self.children.remove_finished_children();
            }
        }

        Ok(())
    }

    async fn update_manager_state(&self, state_socket: &mut StateSocket) {
        if self.state.mode == Mode::Normal {
            state_socket.write_manager_state(&self.state).await.ok();
        }
    }

    async fn should_keep_running(&self, state_socket: &mut StateSocket) -> bool {
        if self.reload_requested {
            state_socket.shutdown().await;
            false
        } else {
            true
        }
    }

    fn execute_display_events(&mut self, event_buffer: &mut Vec<DisplayEvent>) -> EventResponse {
        let mut display_needs_refresh = false;

        event_buffer.drain(..).for_each(|event: DisplayEvent| {
            display_needs_refresh = self.display_event_handler(event) || display_needs_refresh
        });

        if display_needs_refresh {
            EventResponse::DisplayRefreshNeeded
        } else {
            EventResponse::None
        }
    }

    fn refresh_display(&mut self) {
        self.update_windows();

        match self.state.mode {
            // When (resizing / moving) only deal with the single window.
            Mode::ResizingWindow(h) | Mode::MovingWindow(h) => {
                if let Some(window) = self.state.windows.iter().find(|w| w.handle == h) {
                    self.display_server.update_windows(vec![window]);
                }
            }
            _ => {
                let windows: Vec<&Window> = self.state.windows.iter().collect();
                self.display_server.update_windows(windows);
            }
        }
    }

    fn execute_command(&mut self, command: &Command) -> EventResponse {
        if self.command_handler(command) {
            EventResponse::DisplayRefreshNeeded
        } else {
            EventResponse::None
        }
    }
}

async fn get_state_socket() -> Result<StateSocket, Error> {
    let socket_filename = String::from("current_state.sock");
    let socket_file = place_runtime_file(socket_filename.clone())
        .map_err(|_| Error::CreateFile(socket_filename.clone()))?;

    let mut state_socket = StateSocket::default();

    state_socket
        .listen(socket_file)
        .await
        .map_err(|_| Error::ConnectToFile(socket_filename))?;

    Ok(state_socket)
}

async fn get_command_pipe() -> Result<CommandPipe, Error> {
    let file_name = CommandPipe::pipe_name();
    let file_path = file_name.to_str().unwrap().to_string();

    let pipe_file =
        place_runtime_file(&file_name).map_err(|_| Error::CreateFile(file_path.clone()))?;

    CommandPipe::new(pipe_file)
        .await
        .map_err(|_| Error::ConnectToFile(file_path))
}

fn place_runtime_file<P>(path: P) -> std::io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    xdg::BaseDirectories::with_prefix("leftwm")?.place_runtime_file(path)
}

async fn timeout(mills: u64) {
    use tokio::time::{sleep, Duration};
    sleep(Duration::from_millis(mills)).await;
}
