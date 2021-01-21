use std::collections::VecDeque;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

type Queue = Arc<Mutex<VecDeque<ExternalCommand>>>;

#[derive(Debug, Default)]
pub struct CommandPipe {
    queue: Queue,
    listener: Option<tokio::task::JoinHandle<()>>,
    pipe_file: PathBuf,
}

impl Drop for CommandPipe {
    fn drop(&mut self) {
        if !std::thread::panicking() && self.listener.is_some() {
            panic!("CommandPipe has to be shutdown explicitly before drop");
        }
    }
}

impl CommandPipe {
    /// Create and listen to the named pipe.
    pub async fn listen(&mut self, pipe_file: PathBuf) -> Result<()> {
        self.pipe_file = pipe_file;
        let listener = self.build_listener().await?;
        self.listener = Some(listener);
        Ok(())
    }

    /// Explicitly shutdown `CommandPipe` to perform cleanup.
    pub async fn shutdown(&mut self) {
        use std::os::unix::fs::OpenOptionsExt;
        if let Some(listener) = self.listener.take() {
            listener.abort();
            listener.await.ok();

            // Open fifo for write to unblock pending open for read operation that prevents tokio runtime
            // from shutting down.
            std::fs::OpenOptions::new()
                .write(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(self.pipe_file.clone())
                .ok();

            fs::remove_file(self.pipe_file.as_path()).await.ok();
        }
    }

    pub async fn read_command(&mut self) -> Option<ExternalCommand> {
        self.queue.lock().await.pop_front()
    }

    async fn build_listener(&self) -> Result<tokio::task::JoinHandle<()>> {
        let queue = self.queue.clone();
        let pipe_file = self.pipe_file.clone();
        fs::remove_file(pipe_file.clone()).await.ok();
        if let Err(e) = nix::unistd::mkfifo(&pipe_file, nix::sys::stat::Mode::S_IRWXU) {
            log::error!("Failed to create new fifo {:?}", e);
        }

        Ok(tokio::spawn(async move {
            loop {
                read_from_pipe(&queue, &pipe_file).await;
            }
        }))
    }
}

async fn read_from_pipe(queue: &Queue, pipe_file: &PathBuf) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        let cmd = parse_command(line).ok()?;
        queue.lock().await.push_back(cmd);
    }

    Some(())
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
    } else if s.starts_with("MoveWindowTop") {
        return Ok(ExternalCommand::MoveWindowTop);
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

#[derive(Debug, Clone, PartialEq)]
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
    MoveWindowTop,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::helpers::test::temp_path;
    use tokio::io::AsyncWriteExt;
    use tokio::time;

    #[tokio::test]
    async fn read_command() {
        let pipe_file = temp_path().await.unwrap();
        let mut command_pipe = CommandPipe::default();
        command_pipe.listen(pipe_file.clone()).await.unwrap();

        // Open pipe for writing and write some garbage. Then close the pipe.
        {
            let mut pipe = fs::OpenOptions::new()
                .write(true)
                .open(pipe_file.clone())
                .await
                .unwrap();
            pipe.write_all(vec![0x11, 0x22].as_ref()).await.unwrap();
            pipe.flush().await.unwrap();
        }

        // Let the OS close the pipe.
        time::sleep(time::Duration::from_millis(100)).await;

        // Write some meaningful command to the pipe and close it.
        {
            let mut pipe = fs::OpenOptions::new()
                .write(true)
                .open(pipe_file.clone())
                .await
                .unwrap();
            pipe.write_all(b"Reload\n").await.unwrap();
            pipe.flush().await.unwrap();

            let mut command = None;
            while command.is_none() {
                command = command_pipe.read_command().await;
                time::sleep(time::Duration::from_millis(100)).await;
            }

            assert_eq!(ExternalCommand::Reload, command.unwrap());
        }

        command_pipe.shutdown().await;
    }

    #[tokio::test]
    async fn pipe_cleanup() {
        let pipe_file = temp_path().await.unwrap();
        fs::remove_file(pipe_file.as_path()).await.unwrap();
        let mut command_pipe = CommandPipe::default();
        command_pipe.listen(pipe_file.clone()).await.unwrap();

        // Write to pipe.
        {
            let mut pipe = fs::OpenOptions::new()
                .write(true)
                .open(pipe_file.clone())
                .await
                .unwrap();
            pipe.write_all(b"UnloadTheme\n").await.unwrap();
            pipe.flush().await.unwrap();
        }

        // Let the OS close the write end of the pipe before shutting down the listener.
        time::sleep(time::Duration::from_millis(100)).await;

        command_pipe.shutdown().await;
        assert!(!pipe_file.exists());
    }
}
