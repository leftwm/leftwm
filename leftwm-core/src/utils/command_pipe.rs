//! Creates a pipe to listen for external commands.
use crate::models::{Handle, TagId};
use crate::utils::return_pipe::ReturnPipe;
use crate::{command, Command, ReleaseScratchPadOption};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use xdg::BaseDirectories;

/// Holds pipe file location and a receiver.
#[derive(Debug)]
pub struct CommandPipe<H: Handle> {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<Command<H>>,
}

impl<H: Handle> Drop for CommandPipe<H> {
    fn drop(&mut self) {
        use std::os::unix::fs::OpenOptionsExt;
        self.rx.close();

        // Open fifo for write to unblock pending open for read operation that prevents tokio runtime
        // from shutting down.
        if let Err(err) = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
            .open(&self.pipe_file)
        {
            eprintln!(
                "Failed to open {} when dropping CommandPipe: {err}",
                self.pipe_file.display()
            );
        }
    }
}

impl<H: Handle> CommandPipe<H> {
    /// Create and listen to the named pipe.
    /// # Errors
    ///
    /// Will error if unable to `mkfifo`, likely a filesystem issue
    /// such as inadequate permissions.
    pub async fn new(pipe_file: PathBuf) -> Result<Self, std::io::Error> {
        fs::remove_file(pipe_file.as_path()).await.ok();
        if let Err(e) = nix::unistd::mkfifo(&pipe_file, nix::sys::stat::Mode::S_IRWXU) {
            tracing::error!("Failed to create new fifo {:?}", e);
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

    pub async fn read_command(&mut self) -> Option<Command<H>> {
        self.rx.recv().await
    }
}

pub fn pipe_name() -> PathBuf {
    let display = env::var("DISPLAY")
        .ok()
        .and_then(|d| d.rsplit_once(':').map(|(_, r)| r.to_owned()))
        .unwrap_or_else(|| "0".to_string());

    PathBuf::from(format!("command-{display}.pipe"))
}

async fn read_from_pipe<H: Handle>(
    pipe_file: &Path,
    tx: &mpsc::UnboundedSender<Command<H>>,
) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        let cmd = match parse_command(&line) {
            Ok(cmd) => {
                if let Command::Other(_) = cmd {
                    cmd
                } else {
                    let file_name = ReturnPipe::pipe_name();
                    if let Ok(file_path) = BaseDirectories::with_prefix("leftwm") {
                        if let Some(file_path) = file_path.find_runtime_file(&file_name) {
                            if let Ok(mut file) = OpenOptions::new().append(true).open(file_path) {
                                if let Err(e) = writeln!(file, "OK: command executed successfully")
                                {
                                    tracing::error!("Unable to write to return pipe: {e}");
                                }
                            }
                        }
                    }
                    cmd
                }
            }
            Err(err) => {
                tracing::error!("An error occurred while parsing the command: {}", err);
                // return to stdout
                let file_name = ReturnPipe::pipe_name();
                if let Ok(file_path) = BaseDirectories::with_prefix("leftwm") {
                    if let Some(file_path) = file_path.find_runtime_file(file_name) {
                        if let Ok(mut file) = OpenOptions::new().append(true).open(file_path) {
                            if let Err(e) = writeln!(file, "ERROR: Error parsing command: {err}") {
                                tracing::error!("Unable to write error to return pipe: {e}");
                            }
                        }
                    }
                }

                return None;
            }
        };
        tx.send(cmd).ok()?;
    }

    Some(())
}

fn parse_command<H: Handle>(s: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let (head, rest) = s.split_once(' ').unwrap_or((s, ""));
    match head {
        // Move Window
        "MoveWindowDown" => Ok(Command::MoveWindowDown),
        "MoveWindowTop" => build_move_window_top(rest),
        "SwapWindowTop" => build_swap_window_top(rest),
        "MoveWindowUp" => Ok(Command::MoveWindowUp),
        "MoveWindowToNextTag" => build_move_window_to_next_tag(rest),
        "MoveWindowToPreviousTag" => build_move_window_to_previous_tag(rest),
        "MoveWindowToLastWorkspace" => Ok(Command::MoveWindowToLastWorkspace),
        "MoveWindowToNextWorkspace" => Ok(Command::MoveWindowToNextWorkspace),
        "MoveWindowToPreviousWorkspace" => Ok(Command::MoveWindowToPreviousWorkspace),
        "SendWindowToTag" => build_send_window_to_tag(rest),
        // Focus Navigation
        "FocusWindowDown" => Ok(Command::FocusWindowDown),
        "FocusWindowTop" => build_focus_window_top(rest),
        "FocusWindowUp" => Ok(Command::FocusWindowUp),
        "FocusNextTag" => build_focus_next_tag(rest),
        "FocusPreviousTag" => build_focus_previous_tag(rest),
        "FocusWorkspaceNext" => Ok(Command::FocusWorkspaceNext),
        "FocusWorkspacePrevious" => Ok(Command::FocusWorkspacePrevious),
        // Layout
        "DecreaseMainWidth" | "DecreaseMainSize" => build_decrease_main_size(rest), // 'DecreaseMainWidth' deprecated
        "IncreaseMainWidth" | "IncreaseMainSize" => build_increase_main_size(rest), // 'IncreaseMainWidth' deprecated
        "DecreaseMainCount" => Ok(Command::DecreaseMainCount()),
        "IncreaseMainCount" => Ok(Command::IncreaseMainCount()),
        "NextLayout" => Ok(Command::NextLayout),
        "PreviousLayout" => Ok(Command::PreviousLayout),
        "RotateTag" => Ok(Command::RotateTag),
        "SetLayout" => build_set_layout(rest),
        "SetMarginMultiplier" => build_set_margin_multiplier(rest),
        // Scratchpad
        "ToggleScratchPad" => build_toggle_scratchpad(rest),
        "AttachScratchPad" => build_attach_scratchpad(rest),
        "ReleaseScratchPad" => Ok(build_release_scratchpad(rest)),
        "NextScratchPadWindow" => Ok(Command::NextScratchPadWindow {
            scratchpad: rest.to_owned().into(),
        }),
        "PrevScratchPadWindow" => Ok(Command::PrevScratchPadWindow {
            scratchpad: rest.to_owned().into(),
        }),
        // Floating
        "FloatingToTile" => Ok(Command::FloatingToTile),
        "TileToFloating" => Ok(Command::TileToFloating),
        "ToggleFloating" => Ok(Command::ToggleFloating),
        // Workspace/Tag
        "GoToTag" => build_go_to_tag(rest),
        "ReturnToLastTag" => Ok(Command::ReturnToLastTag),
        "SendWorkspaceToTag" => build_send_workspace_to_tag(rest),
        "SwapScreens" => Ok(Command::SwapScreens),
        "ToggleFullScreen" => Ok(Command::ToggleFullScreen),
        "ToggleMaximized" => Ok(Command::ToggleMaximized),
        "ToggleSticky" => Ok(Command::ToggleSticky),
        "ToggleAbove" => Ok(Command::ToggleAbove),
        // General
        "CloseWindow" => Ok(Command::CloseWindow),
        "CloseAllOtherWindows" => Ok(Command::CloseAllOtherWindows),
        "SoftReload" => Ok(Command::SoftReload),
        _ => Ok(Command::Other(s.into())),
    }
}

fn build_attach_scratchpad<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let name = if raw.is_empty() {
        return Err("missing argument scratchpad's name".into());
    } else {
        raw
    };
    Ok(Command::AttachScratchPad {
        scratchpad: name.into(),
        window: None,
    })
}

fn build_release_scratchpad<H: Handle>(raw: &str) -> Command<H> {
    if raw.is_empty() {
        Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::None,
            tag: None,
        }
    } else if let Ok(tag_id) = usize::from_str(raw) {
        Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::None,
            tag: Some(tag_id),
        }
    } else {
        Command::ReleaseScratchPad {
            window: ReleaseScratchPadOption::ScratchpadName(raw.into()),
            tag: None,
        }
    }
}

fn build_toggle_scratchpad<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let name = if raw.is_empty() {
        return Err("missing argument scratchpad's name".into());
    } else {
        raw
    };
    Ok(Command::ToggleScratchPad(name.into()))
}

fn build_go_to_tag<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let headless = without_head(raw, "GoToTag ");
    let mut parts = headless.split(' ');
    let tag: TagId = parts
        .next()
        .ok_or("missing argument tag_id")?
        .parse()
        .or(Err("argument tag_id was missing or not a valid tag number"))?;
    let swap: bool = match parts.next().ok_or("missing argument swap")?.parse() {
        Ok(b) => b,
        Err(_) => Err("argument swap was not true or false")?,
    };
    Ok(Command::GoToTag { tag, swap })
}

fn build_send_window_to_tag<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let tag_id = if raw.is_empty() {
        return Err("missing argument tag_id".into());
    } else {
        match TagId::from_str(raw) {
            Ok(tag) => tag,
            Err(_) => Err("argument tag_id was not a valid tag number")?,
        }
    };
    Ok(Command::SendWindowToTag {
        window: None,
        tag: tag_id,
    })
}

fn build_send_workspace_to_tag<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    if raw.is_empty() {
        return Err("missing argument workspace index".into());
    }
    let mut parts: std::str::Split<'_, char> = raw.split(' ');
    let ws_index: usize = match parts
        .next()
        .expect("split() always returns an array of at least 1 element")
        .parse()
    {
        Ok(ws) => ws,
        Err(_) => Err("argument workspace index was not a valid workspace number")?,
    };
    let tag_index: usize = match parts.next().ok_or("missing argument tag index")?.parse() {
        Ok(tag) => tag,
        Err(_) => Err("argument tag index was not a valid tag number")?,
    };
    Ok(Command::SendWorkspaceToTag(ws_index, tag_index))
}

fn build_set_layout<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let layout_name = if raw.is_empty() {
        return Err("missing layout name".into());
    } else {
        raw
    };
    Ok(Command::SetLayout(String::from(layout_name)))
}

fn build_set_margin_multiplier<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let margin_multiplier = if raw.is_empty() {
        return Err("missing argument multiplier".into());
    } else {
        f32::from_str(raw)?
    };
    Ok(Command::SetMarginMultiplier(margin_multiplier))
}

fn build_focus_window_top<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let swap = if raw.is_empty() {
        false
    } else {
        match bool::from_str(raw) {
            Ok(bl) => bl,
            Err(_) => Err("Argument swap was not true or false")?,
        }
    };
    Ok(Command::FocusWindowTop { swap })
}

fn build_move_window_top<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let swap = if raw.is_empty() {
        true
    } else {
        match bool::from_str(raw) {
            Ok(bl) => bl,
            Err(_) => Err("Argument swap was not true or false")?,
        }
    };
    Ok(Command::MoveWindowTop { swap })
}

fn build_swap_window_top<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let swap = if raw.is_empty() {
        true
    } else {
        match bool::from_str(raw) {
            Ok(bl) => bl,
            Err(_) => Err("Argument swap was not true or false")?,
        }
    };
    Ok(Command::SwapWindowTop { swap })
}

fn build_move_window_to_next_tag<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let follow = if raw.is_empty() {
        true
    } else {
        match bool::from_str(raw) {
            Ok(bl) => bl,
            Err(_) => Err("Argument follow was not true or false")?,
        }
    };
    Ok(Command::MoveWindowToNextTag { follow })
}

fn build_move_window_to_previous_tag<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let follow = if raw.is_empty() {
        true
    } else {
        match bool::from_str(raw) {
            Ok(bl) => bl,
            Err(_) => Err("Argument follow was not true or false")?,
        }
    };
    Ok(Command::MoveWindowToPreviousTag { follow })
}

fn build_increase_main_size<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let mut parts = raw.split(' ');
    let change: i32 = match parts.next().ok_or("missing argument change")?.parse() {
        Ok(num) => num,
        Err(_) => Err("argument change was missing or invalid")?,
    };
    Ok(Command::IncreaseMainSize(change))
}

fn build_decrease_main_size<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    let mut parts = raw.split(' ');
    let change: i32 = match parts.next().ok_or("missing argument change")?.parse() {
        Ok(num) => num,
        Err(_) => Err("argument change was missing or invalid")?,
    };
    Ok(Command::DecreaseMainSize(change))
}

fn build_focus_next_tag<H: Handle>(raw: &str) -> Result<Command<H>, Box<dyn std::error::Error>> {
    match raw {
        "ignore_empty" | "goto_used" => Ok(Command::FocusNextTag {
            behavior: command::FocusDeltaBehavior::IgnoreEmpty,
        }),
        "ignore_used" | "goto_empty" => Ok(Command::FocusNextTag {
            behavior: command::FocusDeltaBehavior::IgnoreUsed,
        }),
        "default" | "" => Ok(Command::FocusNextTag {
            behavior: command::FocusDeltaBehavior::Default,
        }),
        _ => Err(Box::new(InvalidFocusDeltaBehaviorError {
            attempted_value: raw.to_owned(),
            command: Command::<H>::FocusNextTag {
                behavior: command::FocusDeltaBehavior::Default,
            },
        })),
    }
}

fn build_focus_previous_tag<H: Handle>(
    raw: &str,
) -> Result<Command<H>, Box<dyn std::error::Error>> {
    match raw {
        "ignore_empty" | "goto_used" => Ok(Command::FocusPreviousTag {
            behavior: command::FocusDeltaBehavior::IgnoreEmpty,
        }),
        "ignore_used" | "goto_empty" => Ok(Command::FocusPreviousTag {
            behavior: command::FocusDeltaBehavior::IgnoreUsed,
        }),

        "default" | "" => Ok(Command::FocusPreviousTag {
            behavior: command::FocusDeltaBehavior::Default,
        }),
        _ => Err(Box::new(InvalidFocusDeltaBehaviorError {
            attempted_value: raw.to_owned(),
            command: Command::<H>::FocusPreviousTag {
                behavior: command::FocusDeltaBehavior::Default,
            },
        })),
    }
}

fn without_head<'a>(s: &'a str, head: &'a str) -> &'a str {
    if !s.starts_with(head) {
        return s;
    }
    &s[head.len()..]
}

#[derive(Debug)]
struct InvalidFocusDeltaBehaviorError<H: Handle> {
    attempted_value: String,
    command: Command<H>,
}

impl<H: Handle> Error for InvalidFocusDeltaBehaviorError<H> {}

impl<H: Handle> fmt::Display for InvalidFocusDeltaBehaviorError<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.command {
            Command::FocusNextTag { .. } => write!(
                f,
                "Invalid behavior for FocusNextTag: {}",
                &self.attempted_value
            ),
            Command::FocusPreviousTag { .. } => write!(
                f,
                "Invalid behavior for FocusPreviousTag: {}",
                &self.attempted_value
            ),
            _ => write!(f, "Invalid behavior: {}", &self.attempted_value),
        }
    }
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

    #[test]
    fn build_move_window_to_next_tag_without_parameter() {
        assert_eq!(
            build_move_window_to_next_tag("").unwrap(),
            Command::MoveWindowToNextTag { follow: true }
        );
    }

    #[test]
    fn build_move_window_to_previous_tag_without_parameter() {
        assert_eq!(
            build_move_window_to_previous_tag("").unwrap(),
            Command::MoveWindowToPreviousTag { follow: true }
        );
    }

    #[test]
    fn build_focus_next_tag_without_parameter() {
        assert_eq!(
            build_focus_next_tag("").unwrap(),
            Command::FocusNextTag {
                behavior: command::FocusDeltaBehavior::Default
            }
        );
    }

    #[test]
    fn build_focus_previous_tag_without_parameter() {
        assert_eq!(
            build_focus_previous_tag("").unwrap(),
            Command::FocusPreviousTag {
                behavior: command::FocusDeltaBehavior::Default
            }
        );
    }

    #[test]
    fn build_focus_next_tag_with_invalid() {
        assert_eq!(
            build_focus_next_tag("gurke").unwrap_err().to_string(),
            (InvalidFocusDeltaBehaviorError {
                attempted_value: String::from("gurke"),
                command: Command::FocusNextTag {
                    behavior: command::FocusDeltaBehavior::Default,
                }
            })
            .to_string()
        );
    }

    #[test]
    fn build_focus_previous_tag_with_invalid() {
        assert_eq!(
            build_focus_previous_tag("gurke").unwrap_err().to_string(),
            (InvalidFocusDeltaBehaviorError {
                attempted_value: String::from("gurke"),
                command: Command::FocusPreviousTag {
                    behavior: command::FocusDeltaBehavior::Default,
                }
            })
            .to_string()
        );
    }
}
