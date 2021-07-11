//! Tasks for `LeftWM` to execute and track
use serde::{Deserialize, Serialize};
use toml::value::Table;

/// The information needed to generate and run a Task for leftwm.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Component {
    /// The command itself. Either use full path /path/to/bin, or the binary
    /// name itself here. Do not add unnecessary information, such as
    /// "firefox --headless", instead use `args` to pass information to the
    /// binary or script.
    /// `LeftWM` will run the command from within the
    /// ~/.config/leftwm/themes/current/ directory, so any scripts there may be
    /// called with command = sh or command = bash and args = ["up"].
    pub command: String,
    /// Any arguments that ought to be passed to the command.
    /// `LeftWM` will run the command from within the
    /// ~/.config/leftwm/themes/current/ directory, so any scripts there may be
    /// called with command = sh or command = bash and args = ["up"].
    pub args: Option<Vec<String>>,
    /// If a group is set, only the first command which is both in that group
    /// and in the path (installed) will be run. LeftWM should log an error
    /// if no commands in a group are run.
    pub group: Option<String>,
}

/// Tasks for `LeftWM` to manage on behalf of a theme.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Task {
    /// Operation that pertains to starting this particular task
    pub up: Option<Component>,
    /// Operation that pertains to ending this particular task
    pub down: Option<Component>,
    /// `LeftWM` disregards install, that's for `leftwm-theme` to deal with.
    pub install: Option<Table>,
}
