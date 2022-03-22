//! Creates a pipe to listen for external commands.
use crate::layouts::Layout;
use crate::models::TagId;
use crate::Command;
use std::env;
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

    pub fn pipe_name() -> PathBuf {
        let display = env::var("DISPLAY")
            .ok()
            .and_then(|d| d.rsplit_once(':').map(|(_, r)| r.to_owned()))
            .unwrap_or_else(|| "0".to_string());

        PathBuf::from(format!("command-{}.pipe", display))
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
    let (head, rest) = s.split_once(' ').unwrap_or((s, ""));
    match head {
        "SoftReload" => Ok(Command::SoftReload),
        "ToggleFullScreen" => Ok(Command::ToggleFullScreen),
        "ToggleSticky" => Ok(Command::ToggleSticky),
        "SwapScreens" => Ok(Command::SwapScreens),
        "MoveWindowToLastWorkspace" => Ok(Command::MoveWindowToLastWorkspace),
        "MoveWindowToNextWorkspace" => Ok(Command::MoveWindowToNextWorkspace),
        "MoveWindowToPreviousWorkspace" => Ok(Command::MoveWindowToPreviousWorkspace),
        "FloatingToTile" => Ok(Command::FloatingToTile),
        "TileToFloating" => Ok(Command::TileToFloating),
        "ToggleFloating" => Ok(Command::ToggleFloating),
        "MoveWindowUp" => Ok(Command::MoveWindowUp),
        "MoveWindowDown" => Ok(Command::MoveWindowDown),
        "MoveWindowTop" => build_move_window_top(rest),
        "FocusWindowUp" => Ok(Command::FocusWindowUp),
        "FocusWindowDown" => Ok(Command::FocusWindowDown),
        "FocusWindowTop" => build_focus_window_top(rest),
        "FocusNextTag" => Ok(Command::FocusNextTag),
        "FocusPreviousTag" => Ok(Command::FocusPreviousTag),
        "FocusWorkspaceNext" => Ok(Command::FocusWorkspaceNext),
        "FocusWorkspacePrevious" => Ok(Command::FocusWorkspacePrevious),
        "NextLayout" => Ok(Command::NextLayout),
        "PreviousLayout" => Ok(Command::PreviousLayout),
        "RotateTag" => Ok(Command::RotateTag),
        "CloseWindow" => Ok(Command::CloseWindow),
        "ToggleScratchPad" => build_toggle_scratchpad(rest),
        "SendWorkspaceToTag" => build_send_workspace_to_tag(rest),
        "SendWindowToTag" => build_send_window_to_tag(rest),
        "SetLayout" => build_set_layout(rest),
        "SetMarginMultiplier" => build_set_margin_multiplier(rest),
        "CloseAllOtherWindows" => Ok(Command::CloseAllOtherWindows),
        _ => Ok(Command::Other(s.into())),
    }
}

fn build_toggle_scratchpad(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let name = if raw.is_empty() {
        return Err("missing argument scratchpad's name".into());
    } else {
        raw
    };
    Ok(Command::ToggleScratchPad(name.to_string()))
}

fn build_send_window_to_tag(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let tag_id = if raw.is_empty() {
        return Err("missing argument tag_id".into());
    } else {
        TagId::from_str(raw)?
    };
    Ok(Command::SendWindowToTag {
        window: None,
        tag: tag_id,
    })
}

fn build_send_workspace_to_tag(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    if raw.is_empty() {
        return Err("missing argument workspace index".into());
    }
    let mut parts = raw.split(' ');
    let ws_index: usize = parts
        .next()
        .expect("split() always returns an array of at least 1 element")
        .parse()?;
    let tag_index: usize = parts.next().ok_or("missing argument tag index")?.parse()?;
    Ok(Command::SendWorkspaceToTag(ws_index, tag_index))
}

fn build_set_layout(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let layout_name = if raw.is_empty() {
        return Err("missing layout name".into());
    } else {
        raw
    };
    Ok(Command::SetLayout(Layout::from_str(layout_name)?))
}

fn build_set_margin_multiplier(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let margin_multiplier = if raw.is_empty() {
        return Err("missing argument multiplier".into());
    } else {
        f32::from_str(raw)?
    };
    Ok(Command::SetMarginMultiplier(margin_multiplier))
}

fn build_focus_window_top(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let swap = if raw.is_empty() {
        false
    } else {
        bool::from_str(raw)?
    };
    Ok(Command::FocusWindowTop { swap })
}

fn build_move_window_top(raw: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let swap = if raw.is_empty() {
        true
    } else {
        bool::from_str(raw)?
    };
    Ok(Command::MoveWindowTop { swap })
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

    #[test]
    fn build_toggle_scratchpad_without_parameter() {
        assert!(build_toggle_scratchpad("").is_err());
    }

    #[test]
    fn build_send_window_to_tag_without_parameter() {
        assert!(build_send_window_to_tag("").is_err());
    }

    #[test]
    fn build_send_workspace_to_tag_without_parameter() {
        assert!(build_send_workspace_to_tag("").is_err());
    }

    #[test]
    fn build_set_layout_without_parameter() {
        assert!(build_set_layout("").is_err());
    }

    #[test]
    fn build_set_margin_multiplier_without_parameter() {
        assert!(build_set_margin_multiplier("").is_err());
    }

    #[test]
    fn build_move_window_top_without_parameter() {
        assert_eq!(
            build_move_window_top("").unwrap(),
            Command::MoveWindowTop { swap: true }
        );
    }

    #[test]
    fn build_focus_window_top_without_parameter() {
        assert_eq!(
            build_focus_window_top("").unwrap(),
            Command::FocusWindowTop { swap: false }
        );
    }
}
