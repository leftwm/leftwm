//! Creates a pipe to listen for external commands.
use crate::layouts::Layout;
use crate::Command;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Holds pipe file location and a receiver.
#[derive(Debug)]
pub struct CommandPipe<CMD> {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<Command<CMD>>,
}

impl<CMD> Drop for CommandPipe<CMD> {
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

impl<CMD> CommandPipe<CMD>
where
    // TODO remove this constraint
    CMD: Send + 'static,
{
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

    pub async fn read_command(&mut self) -> Option<Command<CMD>> {
        self.rx.recv().await
    }
}

async fn read_from_pipe<CMD>(
    pipe_file: &Path,
    tx: &mpsc::UnboundedSender<Command<CMD>>,
) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        let cmd = parse_command(&line).ok()?;
        tx.send(cmd).ok()?;
    }

    Some(())
}

fn parse_command<CMD>(s: &str) -> std::result::Result<Command<CMD>, ()> {
    let head = *s.split(' ').collect::<Vec<&str>>().get(0).unwrap_or(&"");
    match head {
        "SoftReload" => Ok(Command::SoftReload),
        "ToggleFullScreen" => Ok(Command::ToggleFullScreen),
        "SwapScreens" => Ok(Command::SwapScreens),
        "MoveWindowToLastWorkspace" => Ok(Command::MoveWindowToLastWorkspace),
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
        _ => Err(()),
    }
}

// TODO
/*
fn build_load_theme(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "LoadTheme ");
    let path = Path::new(headless);
    if path.is_file() {
        Ok(Command<CMD>::LoadTheme(path.into()))
    } else {
        Err(())
    }
}
*/

fn build_toggle_scratchpad<CMD>(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "ToggleScratchPad ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let name = *parts.get(0).ok_or(())?;
    Ok(Command::ToggleScratchPad(name.to_string()))
}

fn build_send_window_to_tag<CMD>(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "SendWindowToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let tag_index: usize = parts.get(0).ok_or(())?.parse().map_err(|_| ())?;
    Ok(Command::SendWindowToTag(tag_index))
}

fn build_send_workspace_to_tag<CMD>(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "SendWorkspaceToTag ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let ws_index: usize = parts.get(0).ok_or(())?.parse().map_err(|_| ())?;
    let tag_index: usize = parts.get(1).ok_or(())?.parse().map_err(|_| ())?;
    Ok(Command::SendWorkspaceToTag(ws_index, tag_index))
}

fn build_set_layout<CMD>(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "SetLayout ");
    let parts: Vec<&str> = headless.split(' ').collect();
    let layout_name = *parts.get(0).ok_or(())?;
    let layout = match Layout::from_str(layout_name) {
        Ok(layout) => layout,
        Err(err) => {
            // TODO better global handling
            log::error!("{}", err);
            return Err(());
        }
    };
    Ok(Command::SetLayout(layout))
}

fn build_set_margin_multiplier<CMD>(raw: &str) -> std::result::Result<Command<CMD>, ()> {
    let headless = without_head(raw, "SetMarginMultiplier ");
    let parts: Vec<&str> = headless.split(' ').collect();
    if parts.len() != 1 {
        return Err(());
    }
    let margin_multiplier = f32::from_str(parts[0]).map_err(|_| ())?;
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
            pipe.write_all(b"SoftReload\n").await.unwrap();
            pipe.flush().await.unwrap();

            assert_eq!(
                Command<CMD>::SoftReload,
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
