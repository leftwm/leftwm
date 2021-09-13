//! Save and restore manager state.
mod common;

use crate::errors::Result;
use crate::Manager;
use common::config::Config;

pub trait State {
    /// Write current state to a file.
    /// It will be used to restore the state after soft reload.
    ///
    /// # Errors
    ///
    /// Will return error if unable to create `state_file` or
    /// if unable to serialize the text.
    /// May be caused by inadequate permissions, not enough
    /// space on drive, or other typical filesystem issues.
    fn save(&self, manager: &Manager, config: &Config) -> Result<()>;

    /// Load saved state if it exists.
    fn load(&self, manager: &mut Manager, config: &Config);
}
