use std::io::Result;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct CommandPipe {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<ExternalCommand>,
}

impl Drop for CommandPipe {
    fn drop(&mut self) {
        self.rx.close();

        // Open fifo for write to unblock pending open for read operation that prevents tokio runtime
        // from shutting down.
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
            .open(self.pipe_file.clone())
            .ok();
    }
}

impl CommandPipe {
    /// Create and listen to the named pipe.
    pub async fn new(pipe_file: PathBuf) -> Result<Self> {
        fs::remove_file(pipe_file.as_path()).await.ok();
        if let Err(e) = nix::unistd::mkfifo(&pipe_file, nix::sys::stat::Mode::S_IRWXU) {
            log::error!("Failed to create new fifo {:?}", e);
        }

        let path = pipe_file.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while !tx.is_closed() {
                read_from_pipe(&path, &tx).await;
            }
            fs::remove_file(path).await.ok();
        });

        Ok(Self { pipe_file, rx })
    }

    pub async fn read_command(&mut self) -> Option<ExternalCommand> {
        self.rx.recv().await
    }
}

async fn read_from_pipe(
    pipe_file: &Path,
    tx: &mpsc::UnboundedSender<ExternalCommand>,
) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        let cmd = parse_command(line).ok()?;
        tx.send(cmd).ok()?;
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
    } else if s.starts_with("SetLayout ") {
        return build_set_layout(s);
    } else if s.starts_with("SwapScreens") {
        return Ok(ExternalCommand::SwapScreens);
    } else if s.starts_with("MoveWindowToLastWorkspace") {
        return Ok(ExternalCommand::MoveWindowToLastWorkspace);
    } else if s.starts_with("SnapWindowTile") {
        return Ok(ExternalCommand::SnapWindowTile);
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

fn build_set_layout(mut raw: String) -> std::result::Result<ExternalCommand, ()> {
    crop_head(&mut raw, "SetLayout ");
    let parts: Vec<&str> = raw.split(' ').collect();
    if parts.len() != 1 {
        return Err(());
    }
    let layout = String::from_str(parts[0]).map_err(|_| ())?;
    Ok(ExternalCommand::SetLayout(layout))
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
    SnapWindowTile,
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
    SetLayout(String),
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
        let mut command_pipe = CommandPipe::new(pipe_file.clone()).await.unwrap();

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

            assert_eq!(
                ExternalCommand::Reload,
                command_pipe.read_command().await.unwrap()
            );
        }
    }

    #[tokio::test]
    async fn pipe_cleanup() {
        let pipe_file = temp_path().await.unwrap();
        fs::remove_file(pipe_file.as_path()).await.unwrap();

        // Write to pipe.
        {
            let _command_pipe = CommandPipe::new(pipe_file.clone()).await.unwrap();
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

        assert!(!pipe_file.exists());
    }
}
