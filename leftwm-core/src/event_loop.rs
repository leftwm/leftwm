use crate::models::Handle;
use crate::{child_process::Nanny, config::Config};
use crate::{
    Command, CommandPipe, DisplayEvent, DisplayServer, Manager, Mode, StateSocket, Window,
};
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Once};

use tracing::error;

/// Errors which can appear while running the event loop.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    #[error("Couldn't create the file: '{0}'")]
    CreateFile(PathBuf),

    #[error("Couldn't connect to file: '{0}'")]
    ConnectToFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EventResponse {
    None,
    DisplayRefreshNeeded,
}

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    /// Starts the event loop of leftwm
    ///
    /// # Errors
    /// `EventResponse` if the initialisation of the command pipe or/and the state socket failed.
    pub async fn start_event_loop(mut self) -> Result<(), Error> {
        let state_socket = get_state_socket().await?;
        let command_pipe = get_command_pipe().await?;

        self.call_up_scripts();
        self.event_loop(state_socket, command_pipe).await
    }

    async fn event_loop(
        &mut self,
        mut state_socket: StateSocket,
        mut command_pipe: CommandPipe<H>,
    ) -> Result<(), Error> {
        let after_first_loop: Once = Once::new();
        let mut event_buffer: Vec<DisplayEvent<H>> = vec![];
        while self.should_keep_running(&mut state_socket).await {
            self.update_manager_state(&mut state_socket).await;
            self.display_server.flush();

            let response: EventResponse = tokio::select! {
                () = self.display_server.wait_readable(), if event_buffer.is_empty() => {
                    self.add_events(&mut event_buffer);
                    continue;
                }
                // When a mouse button is pressed or enter/motion notifies are blocked and only appear
                // once the button is released. This is to double check that we know which window
                // is currently focused.
                () = timeout(100), if event_buffer.is_empty()
                    && self.state.focus_manager.sloppy_mouse_follows_focus
                    && self.state.focus_manager.behaviour.is_sloppy() => {
                        self.refresh_focus(&mut event_buffer);
                        continue;
                    }
                Some::<Command<H>>(cmd) = command_pipe.read_command(), if event_buffer.is_empty() => self.execute_command(&cmd),
                else => self.execute_display_events(&mut event_buffer),
            };

            match response {
                EventResponse::None => (),
                EventResponse::DisplayRefreshNeeded => self.refresh_display(),
            };

            self.execute_actions(&mut event_buffer);

            // We need to run once through all of the loop to properly initialize the state
            // before we can restore the previous state
            after_first_loop.call_once(|| {
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

    fn execute_display_events(&mut self, event_buffer: &mut Vec<DisplayEvent<H>>) -> EventResponse {
        let mut display_needs_refresh = false;

        event_buffer.drain(..).for_each(|event: DisplayEvent<H>| {
            display_needs_refresh = self.display_event_handler(event) || display_needs_refresh;
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
                let windows: Vec<&Window<H>> = self.state.windows.iter().collect();
                self.display_server.update_windows(windows);
            }
        }
    }

    fn execute_command(&mut self, command: &Command<H>) -> EventResponse {
        if self.command_handler(command) {
            EventResponse::DisplayRefreshNeeded
        } else {
            EventResponse::None
        }
    }

    fn add_events(&mut self, event_buffer: &mut Vec<DisplayEvent<H>>) -> EventResponse {
        event_buffer.append(&mut self.display_server.get_next_events());
        EventResponse::None
    }

    fn refresh_focus(&self, event_buffer: &mut Vec<DisplayEvent<H>>) -> EventResponse {
        if let Some(verify_event) = self.display_server.generate_verify_focus_event() {
            event_buffer.push(verify_event);
        }
        EventResponse::None
    }

    // Perform any actions requested by the handler.
    fn execute_actions(&mut self, event_buffer: &mut Vec<DisplayEvent<H>>) {
        while !self.state.actions.is_empty() {
            if let Some(act) = self.state.actions.pop_front() {
                if let Some(event) = self.display_server.execute_action(act) {
                    event_buffer.push(event);
                }
            }
        }
    }

    fn call_up_scripts(&mut self) {
        match Nanny::run_global_up_script() {
            Ok(child) => {
                self.children.insert(child);
            }
            Err(err) => tracing::warn!("Global up script faild: {}", err),
        }
        match Nanny::boot_current_theme() {
            Ok(child) => {
                self.children.insert(child);
            }
            Err(err) => tracing::warn!("Theme loading failed: {}", err),
        }
    }
}

async fn get_state_socket() -> Result<StateSocket, Error> {
    let socket_filename = Path::new("current_state.sock");
    let socket_file = place_runtime_file(socket_filename)
        .map_err(|_| Error::CreateFile(socket_filename.into()))?;

    let mut state_socket = StateSocket::default();

    state_socket
        .listen(socket_file)
        .await
        .map_err(|_| Error::ConnectToFile(socket_filename.into()))?;

    Ok(state_socket)
}

async fn get_command_pipe<H: Handle>() -> Result<CommandPipe<H>, Error> {
    let file_name = crate::pipe_name();

    let pipe_file =
        place_runtime_file(&file_name).map_err(|_| Error::CreateFile(file_name.clone()))?;

    CommandPipe::new(pipe_file)
        .await
        .map_err(|_| Error::ConnectToFile(file_name))
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
