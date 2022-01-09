use anyhow::{Context, Result};
use clap::{App, Arg};
use leftwm_core::CommandPipe;
use std::fs::OpenOptions;
use std::io::prelude::*;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("LeftWM Command")
        .author("Lex Childs <lex.childs@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Sends external commands to LeftWM")
        .arg(
            Arg::with_name("command")
                .help("The command to be sent. See 'list' flag.")
                // .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("list")
                .help("Print a list of available commands with their arguments.")
                .short("l")
                .long("list"),
        )
        .get_matches();

    let file_name = CommandPipe::pipe_name();
    let file_path = BaseDirectories::with_prefix("leftwm")?
        .find_runtime_file(&file_name)
        .with_context(|| format!("ERROR: Couldn't find {}", file_name.display()))?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .with_context(|| format!("ERROR: Couldn't open {}", file_name.display()))?;
    if let Some(commands) = matches.values_of("command") {
        for command in commands {
            if let Err(e) = writeln!(file, "{}", command) {
                eprintln!(" ERROR: Couldn't write to commands.pipe: {}", e);
            }
        }
    }

    let command_list = matches.occurrences_of("list") == 1;

    if command_list {
        println!(
            "
        Available Commands:

        Commands without arguments:

        UnloadTheme
        SoftReload
        ToggleFullScreen
        ToggleSticky
        SwapScreens
        MoveWindowToLastWorkspace
        MoveWindowToNextWorkspace
        MoveWindowToPreviousWorkspace
        FloatingToTile
        TileToFloating
        ToggleFloating
        MoveWindowUp
        MoveWindowDown
        MoveWindowTop
        FocusWindowUp
        FocusWindowDown
        FocusWindowTop
        FocusNextTag
        FocusPreviousTag
        FocusWorkspaceNext
        FocusWorkspacePrevious
        NextLayout
        PreviousLayout
        RotateTag
        CloseWindow

        Commands with arguments:
            Use quotations for the command and arguments, like this:
            leftwm-command \"<command> <args>\"

        LoadTheme              Args: <Path_to/theme.toml>
        ToggleScratchPad       Args: <ScratchpadName>
        SendWorkspaceToTag     Args: <workspaxe_index> <tag_index> (int)
        SendWindowToTag        Args: <tag_index> (int)
        SetLayout              Args: <LayoutName>
        SetMarginMultiplier    Args: <multiplier-value> (float)

        For more information please visit:
        https://github.com/leftwm/leftwm/wiki/External-Commands
         "
        );
    }
    Ok(())
}
