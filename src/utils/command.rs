#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    OpenTerminal,
    CloseWindow,
    SwapWorkspaces,
    GotoWorkspace,
    MovetoWorkspace,
    //GotoWorkspace(i32),
    //MovetoWorkspace(i32),
}
