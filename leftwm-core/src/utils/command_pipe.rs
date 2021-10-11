//! Creates a pipe to listen for external commands.
use crate::layouts::Layout;
use crate::Command;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Holds pipe file location and a receiver.
#[derive(Debug)]
pub struct CommandPipe {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<Command>,
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
    pub async fn new(pipe_file: PathBuf) -> Result<Self, std::io::Error> {
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

    pub async fn read_command(&mut self) -> Option<Command> {
        self.rx.recv().await
    }
}

async fn read_from_pipe(pipe_file: &Path, tx: &mpsc::UnboundedSender<Command>) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        let cmd = match parse_command(&line) {
            Ok(cmd) => cmd,
            Err(err) => {
                log::error!("An error occurred while parsing the command: {}", err);
                return None;
            }
        };
        tx.send(cmd).ok()?;
    }

    Some(())
}

fn parse_command(s: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let head = *s.split(' ').collect::<Vec<&str>>().get(0).unwrap_or(&"");
    match head {
        "SoftReload" => Ok(Command::SoftReload),
        "ToggleFullScreen" => Ok(Command::ToggleFullScreen),
        "ToggleSticky" => Ok(Command::ToggleSticky),
        "SwapScreens" => Ok(Command::SwapScreens),
        "MoveWindowToLastWorkspace" => Ok(Command::MoveWindowToLastWorkspace),
        "MoveWindowToNextWorkspace" => Ok(Command::MoveWindowToNextWorkspace),
        "MoveWindowToPreviousWorkspace" => Ok(Command::MoveWindowToPreviousWorkspace),
        "FloatingToTile" => Ok(Command::FloatingToTile),
        "MoveWindowUp" => Ok(Command::MoveWindowUp),
        "MoveWindowDown" => Ok(Command::MoveWindowDown),
        "FocusWindowUp" => Ok(Command::FocusWindowUp),
        "MoveWindowTop" => Ok(Command::MoveWindowTop),
        "FocusWindowDown" => Ok(Command::FocusWindowDown),
        "FocusNextTag" => Ok(Command::FocusNextTag),
        "FocusPreviousTag" => Ok(Command::FocusPreviousTag),
        "FocusWorkspaceNext" => Ok(Command::FocusWorkspaceNext),
        "FocusWorkspacePrevious" => Ok(Command::FocusWorkspacePrevious),
        "NextLayout" => Ok(Command::NextLayout),
        "PreviousLayout" => Ok(Command::PreviousLayout),
        "RotateTag" => Ok(Command::RotateTag),
        "CloseWindow" => Ok(Command::CloseWindow),
        "ToggleScratchPad" => build_toggle_scratchpad(s),
        "SendWorkspaceToTag" => build_send_workspace_to_tag(s),
        "SendWindowToTag" => build_send_window_to_tag(s),
        "SetLayout" => build_set_layout(s),
        "SetMarginMultiplier" => build_set_margin_multiplier(s),
        _ => Ok(Command::Other(s.into())),
    }
}

fn build_toggle_scratchpad(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "ToggleScratchPad ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let name = *parts.get(0).ok_or("missing argument scratchpad's name")?;
    Ok(Command::ToggleScratchPad(name.to_string()))
}

fn build_send_window_to_tag(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "SendWindowToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let tag_index: usize = parts.get(0).ok_or("missing argument tag_index")?.parse()?;
    Ok(Command::SendWindowToTag(tag_index))
}

fn build_send_workspace_to_tag(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "SendWorkspaceToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let ws_index: usize = parts
        .get(0)
        .ok_or("missing argument workspace index")?
        .parse()?;
    let tag_index: usize = parts.get(1).ok_or("missing argument tag index")?.parse()?;
    Ok(Command::SendWorkspaceToTag(ws_index, tag_index))
}

fn build_set_layout(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "SetLayout ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let layout_name = *parts.get(0).ok_or("missing layout name")?;
    Ok(Command::SetLayout(Layout::from_str(layout_name)?))
}

fn build_set_margin_multiplier(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "SetMarginMultiplier ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let margin_multiplier = f32::from_str(parts.get(0).ok_or("missing argument multiplier")?)?;
    Ok(Command::SetMarginMultiplier(margin_multiplier))
}

fn without_head<'a, 'b>(s: &'a str, head: &'b str) -> &'a str {
    if !s.starts_with(head) {
        return s;
    }
    &s[head.len()..]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::helpers::test::temp_path;
    use tokio::io::AsyncWriteExt;
    use tokio::time;

    #[tokio::test]
    async fn read_good_command() {
        let pipe_file = temp_path().await.unwrap();
        let mut command_pipe = CommandPipe::new(pipe_file.clone()).await.unwrap();

        // Write some meaningful command to the pipe and close it.
        {
            let mut pipe = fs::OpenOptions::new()
                .write(true)
                .open(&pipe_file)
                .await
                .unwrap();
            pipe.write_all(b"SoftReload\n").await.unwrap();
            pipe.flush().await.unwrap();

            assert_eq!(
                Command::SoftReload,
                command_pipe.read_command().await.unwrap()
            );
        }
    }

    #[tokio::test]
    async fn read_bad_command() {
        let pipe_file = temp_path().await.unwrap();
        let mut command_pipe = CommandPipe::new(pipe_file.clone()).await.unwrap();

        // Write some custom command and close it.
        {
            let mut pipe = fs::OpenOptions::new()
                .write(true)
                .open(&pipe_file)
                .await
                .unwrap();
            pipe.write_all(b"Hello World\n").await.unwrap();
            pipe.flush().await.unwrap();

            assert_eq!(
                Command::Other("Hello World".to_string()),
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
                .open(&pipe_file)
                .await
                .unwrap();
            pipe.write_all(b"ToggleFullScreen\n").await.unwrap();
            pipe.flush().await.unwrap();
        }

        // Let the OS close the write end of the pipe before shutting down the listener.
        time::sleep(time::Duration::from_millis(100)).await;

        // NOTE: clippy is drunk
        {
            assert!(!pipe_file.exists());
        }
    }
}
