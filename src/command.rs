#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    Execute,
    CloseWindow,
    SwapTags,
    GotoTag,
    MoveToTag,
    MoveToLastWorkspace,
}
