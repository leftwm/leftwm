use crate::{child_process::Nanny, config::Config};
use crate::{CommandPipe, DisplayServer, Manager, Mode, StateSocket, Window};
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Once};

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// # Panics
    /// This function panics if it can't create or write to the command file.
    pub async fn event_loop(mut self) {
        let socket_file = place_runtime_file("current_state.sock")
            .expect("ERROR: couldn't create current_state.sock");
        let mut state_socket = StateSocket::default();
        state_socket
            .listen(socket_file)
            .await
            .expect("ERROR: couldn't connect to current_state.sock");

        let file_name = CommandPipe::pipe_name();
        let pipe_file = place_runtime_file(&file_name)
            .unwrap_or_else(|_| panic!("ERROR: couldn't create {}", file_name.display()));
        let mut command_pipe = CommandPipe::new(pipe_file)
            .await
            .unwrap_or_else(|_| panic!("ERROR: couldn't connect to {}", file_name.display()));

        //start the current theme
        let after_first_loop: Once = Once::new();

        //main event loop
        let mut event_buffer = vec![];
        loop {
            if self.state.mode == Mode::Normal {
                state_socket.write_manager_state(&self.state).await.ok();
            }
            self.display_server.flush();

            let mut needs_update = false;
            tokio::select! {
                _ = self.display_server.wait_readable(), if event_buffer.is_empty() => {
                    event_buffer.append(&mut self.display_server.get_next_events());
                    continue;
                }
                // When a mouse button is pressed enter/motion notifies are blocked and only appear
                // once the button is released. This is to double check that we know which window
                // is currently focused.
                _ = timeout(100), if event_buffer.is_empty()
                    && self.state.focus_manager.behaviour.is_sloppy() => {
                    if let Some(verify_event) = self.display_server.generate_verify_focus_event() {
                        event_buffer.push(verify_event);
                    }
                    continue;
                }
                Some(cmd) = command_pipe.read_command(), if event_buffer.is_empty() => {
                    needs_update = self.command_handler(&cmd) || needs_update;
                }
                else => {
                    event_buffer
                        .drain(..)
                        .for_each(|event| needs_update = self.display_event_handler(event) || needs_update);
                }
            }

            // If we need to update the displayed state.
            if needs_update {
                self.update_windows();

                match self.state.mode {
                    // When (resizing / moving) only deal with the single window.
                    Mode::ResizingWindow(h) | Mode::MovingWindow(h) => {
                        let windows: Vec<&Window> = self
                            .state
                            .windows
                            .iter()
                            .filter(|w| w.handle == h)
                            .collect();
                        self.display_server.update_windows(windows);
                    }
                    _ => {
                        let windows: Vec<&Window> = self.state.windows.iter().collect();
                        self.display_server.update_windows(windows);
                    }
                }
            }

            //preform any actions requested by the handler
            while !self.state.actions.is_empty() {
                if let Some(act) = self.state.actions.pop_front() {
                    if let Some(event) = self.display_server.execute_action(act) {
                        event_buffer.push(event);
                    }
                }
            }

            //after the very first loop run the 'up' scripts (global and theme). we need the unix
            //socket to already exist.
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
                self.children.reap();
            }

            if self.reload_requested {
                state_socket.shutdown().await;
                break;
            }
        }
    }
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
