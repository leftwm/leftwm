//! Creates a pipe to listen for external commands.
use std::io::Result;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Holds pipe file location and a receiver.
#[derive(Debug)]
pub struct CommandPipe {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<ExternalCommand>,
}

impl Drop for CommandPipe {
    fn drop(&mut self) {
        use std::os::unix::fs::OpenOptionsExt;
        self.rx.close();

        // Open fifo for write to unblock pending open for read operation that prevents tokio runtime
        // from shutting down.
        std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
            .open(self.pipe_file.clone())
            .ok();
    }
}

impl CommandPipe {
    /// Create and listen to the named pipe.
    /// # Errors
    ///
    /// Will error if unable to `mkfifo`, likely a filesystem issue
    /// such as inadequate permissions.
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
        let cmd = parse_command(&line).ok()?;
        tx.send(cmd).ok()?;
    }

    Some(())
}

fn parse_command(s: &str) -> std::result::Result<ExternalCommand, ()> {
    let head = *s.split(' ').collect::<Vec<&str>>().get(0).unwrap_or(&"");
    match head {
        "Reload" => Ok(ExternalCommand::Reload),
        "ToggleFullScreen" => Ok(ExternalCommand::ToggleFullScreen),
        "SwapScreens" => Ok(ExternalCommand::SwapScreens),
        "MoveWindowToLastWorkspace" => Ok(ExternalCommand::MoveWindowToLastWorkspace),
        "FloatingToTile" => Ok(ExternalCommand::FloatingToTile),
        "MoveWindowUp" => Ok(ExternalCommand::MoveWindowUp),
        "MoveWindowDown" => Ok(ExternalCommand::MoveWindowDown),
        "FocusWindowUp" => Ok(ExternalCommand::FocusWindowUp),
        "MoveWindowTop" => Ok(ExternalCommand::MoveWindowTop),
        "FocusWindowDown" => Ok(ExternalCommand::FocusWindowDown),
        "FocusNextTag" => Ok(ExternalCommand::FocusNextTag),
        "FocusPreviousTag" => Ok(ExternalCommand::FocusPreviousTag),
        "FocusWorkspaceNext" => Ok(ExternalCommand::FocusWorkspaceNext),
        "FocusWorkspacePrevious" => Ok(ExternalCommand::FocusWorkspacePrevious),
        "NextLayout" => Ok(ExternalCommand::NextLayout),
        "PreviousLayout" => Ok(ExternalCommand::PreviousLayout),
        "RotateTag" => Ok(ExternalCommand::RotateTag),
        "CloseWindow" => Ok(ExternalCommand::CloseWindow),
        "ToggleScratchPad" => build_toggle_scratchpad(s),
        "SendWorkspaceToTag" => build_send_workspace_to_tag(s),
        "SendWindowToTag" => build_send_window_to_tag(s),
        "SetLayout" => build_set_layout(s),
        "SetMarginMultiplier" => build_set_margin_multiplier(s),
        _ => Err(()),
    }
}

// TODO
/*
fn build_load_theme(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "LoadTheme ");
    let path = Path::new(headless);
    if path.is_file() {
        Ok(ExternalCommand::LoadTheme(path.into()))
    } else {
        Err(())
    }
}
*/

fn build_toggle_scratchpad(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "ToggleScratchPad ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let name = *parts.get(0).ok_or(())?;
    Ok(ExternalCommand::ToggleScratchPad(name.to_string()))
}

fn build_send_window_to_tag(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "SendWindowToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let tag_index: usize = parts.get(0).ok_or(())?.parse().map_err(|_| ())?;
    Ok(ExternalCommand::SendWindowToTag(tag_index))
}

fn build_send_workspace_to_tag(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "SendWorkspaceToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let ws_index: usize = parts.get(0).ok_or(())?.parse().map_err(|_| ())?;
    let tag_index: usize = parts.get(1).ok_or(())?.parse().map_err(|_| ())?;
    Ok(ExternalCommand::SendWorkspaceToTag(ws_index, tag_index))
}

fn build_set_layout(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "SetLayout ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let layout_name = *parts.get(0).ok_or(())?;
    let layout = String::from_str(layout_name).map_err(|_| ())?;
    Ok(ExternalCommand::SetLayout(layout))
}

fn build_set_margin_multiplier(raw: &str) -> std::result::Result<ExternalCommand, ()> {
    let headless = without_head(raw, "SetMarginMultiplier ");
    let parts: Vec<&str> = headless.split(' ').collect();
    if parts.len() != 1 {
        return Err(());
    }
    let margin_multiplier = String::from_str(parts[0]).map_err(|_| ())?;
    Ok(ExternalCommand::SetMarginMultiplier(margin_multiplier))
}

fn without_head<'a, 'b>(s: &'a str, head: &'b str) -> &'a str {
    if !s.starts_with(head) {
        return s;
    }
    &s[head.len()..]
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalCommand {
    Reload,
    ToggleScratchPad(String),
    ToggleFullScreen,
    SendWorkspaceToTag(usize, usize),
    SendWindowToTag(usize),
    SwapScreens,
    MoveWindowToLastWorkspace,
    FloatingToTile,
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
    RotateTag,
    SetLayout(String),
    SetMarginMultiplier(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::helpers::test::temp_path;
    use tokio::io::AsyncWriteExt;
    use tokio::time;

    #[test]
    fn read_command() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(read_command_async());
    }
    async fn read_command_async() {
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

    #[test]
    fn pipe_cleanup() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(pipe_cleanup_async());
    }
    async fn pipe_cleanup_async() {
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
