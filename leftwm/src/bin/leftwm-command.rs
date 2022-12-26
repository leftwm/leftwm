use anyhow::{Context, Result};
use clap::{arg, command};
use leftwm_core::CommandPipe;
use std::fs::OpenOptions;
use std::io::prelude::*;
use xdg::BaseDirectories;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = get_command().get_matches();

    let file_name = CommandPipe::pipe_name();
    let file_path = BaseDirectories::with_prefix("leftwm")?
        .find_runtime_file(&file_name)
        .with_context(|| format!("ERROR: Couldn't find {}", file_name.display()))?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .with_context(|| format!("ERROR: Couldn't open {}", file_name.display()))?;
    if let Some(commands) = matches.get_many::<String>("COMMAND") {
        for command in commands {
            if let Err(e) = writeln!(file, "{command}") {
                eprintln!(" ERROR: Couldn't write to commands.pipe: {e}");
            }
        }
    }

    if matches.get_flag("list") {
        print_commandlist();
    }
    Ok(())
}

fn get_command() -> clap::Command {
    command!("LeftWM Command")
        .about("Sends external commands to LeftWM")
        .help_template(leftwm::utils::get_help_template())
        .args(&[
            arg!(-l --list "Print a list of available commands with their arguments."),
            arg!([COMMAND] ... "The command to be sent. See 'list' flag."),
        ])
}

fn print_commandlist() {
    println!(
        "
        Available Commands:

        Commands without arguments:

        UnloadTheme
        SoftReload
        ToggleFullScreen
        ToggleSticky
        SwapScreens
        MoveWindowToNextTag
        MoveWindowToPreviousTag
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
        ReturnToLastTag
        CloseWindow

        Commands with arguments:
            Use quotations for the command and arguments, like this:
            leftwm-command \"<command> <args>\"

        LoadTheme              Args: <Path_to/theme.ron>
            Note: `theme.toml` will be deprecated but stays for backwards compatibility for a while 
        AttachScratchPad       Args: <ScratchpadName>
        ReleaseScratchPad      Args: <tag_index> or <ScratchpadName>
        NextScratchPadWindow   Args: <ScratchpadName>
        PrevScratchPadWindow   Args: <ScratchpadName>
        ToggleScratchPad       Args: <ScratchpadName>
        SendWorkspaceToTag     Args: <workspaxe_index> <tag_index> (int)
        SendWindowToTag        Args: <tag_index> (int)
        SetLayout              Args: <LayoutName>
        SetMarginMultiplier    Args: <multiplier-value> (float)
        FocusWindow            Args: <WindowClass> or <visible-window-index> (int)

        For more information please visit:
        https://github.com/leftwm/leftwm/wiki/External-Commands
         "
    );
}
