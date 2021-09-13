//! Save and restore manager state.

use crate::config::Config;
use crate::errors::Result;
use crate::Manager;

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
    fn save(&self, manager: &Manager, config: &dyn Config) -> Result<()>;

    /// Load saved state if it exists.
    fn load(&self, manager: &mut Manager, config: &dyn Config);
}
