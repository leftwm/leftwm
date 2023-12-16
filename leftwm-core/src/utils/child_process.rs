//! Starts programs in autostart, runs global 'up' script, and boots theme. Provides function to
//! boot other desktop files also.
use crate::errors::Result;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::iter::{Extend, FromIterator};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{atomic::AtomicBool, Arc};
use xdg::BaseDirectories;

pub type ChildID = u32;

#[derive(Default)]
pub struct Nanny {}

impl Nanny {
    /// Retrieve the path to the config directory. Tries to create it if it does not exist.
    ///
    /// # Errors
    ///
    /// Will error if unable to open or create the config directory.
    /// Could be caused by inadequate permissions.
    fn get_config_dir() -> Result<PathBuf> {
        BaseDirectories::with_prefix("leftwm")?
            .create_config_directory("")
            .map_err(Into::into)
    }

    /// Runs a script if it exits
    fn run_script(path: &Path) -> Result<Child> {
        Command::new(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(Into::into)
    }

    /// Runs the 'up' script in the config directory, if there is one.
    ///
    /// # Errors
    ///
    /// Will error if unable to open current config directory.
    /// Could be caused by inadequate permissions.
    pub fn run_global_up_script() -> Result<Child> {
        let mut path = Self::get_config_dir()?;
        let mut scripts = Self::get_files_in_path_with_ext(&path, "up")?;

        while let Some(Reverse(script)) = scripts.pop() {
            if let Err(e) = Self::run_script(&script) {
                tracing::error!("Unable to run script {script:?}, error: {e}");
            }
        }

        path.push("up");
        Self::run_script(&path)
    }

    /// Returns a min-heap of files with the specified extension.
    ///
    /// # Errors
    ///
    /// Comes from `std::fs::read_dir()`.
    fn get_files_in_path_with_ext(
        path: impl AsRef<Path>,
        ext: impl AsRef<OsStr>,
    ) -> Result<BinaryHeap<Reverse<PathBuf>>> {
        let dir = fs::read_dir(&path)?;

        let mut files = BinaryHeap::new();

        for entry in dir.flatten() {
            let file = entry.path();

            if let Some(extension) = file.extension() {
                if extension == ext.as_ref() {
                    files.push(Reverse(file));
                }
            }
        }

        Ok(files)
    }

    /// Runs the 'up' script of the current theme, if there is one.
    ///
    /// # Errors
    ///
    /// Will error if unable to open current theme directory.
    /// Could be caused by inadequate permissions.
    pub fn boot_current_theme() -> Result<Child> {
        let mut path = Self::get_config_dir()?;
        path.push("themes");
        path.push("current");
        path.push("up");
        Self::run_script(&path)
    }
}

/// A struct managing children processes.
#[derive(Debug, Default)]
pub struct Children {
    inner: HashMap<ChildID, Child>,
}

impl Children {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Insert a `Child` in the `Children`.
    ///
    /// # Returns
    /// - `true` if `child` is a new child-process
    /// - `false` if `child` is already known
    pub fn insert(&mut self, child: Child) -> bool {
        self.inner.insert(child.id(), child).is_none()
    }

    /// Merge another `Children` into this `Children`.
    pub fn merge(&mut self, reaper: Self) {
        self.inner.extend(reaper.inner);
    }

    /// Remove all children precosses which finished
    pub fn remove_finished_children(&mut self) {
        self.inner
            .retain(|_, child| child.try_wait().map_or(true, |ret| ret.is_none()));
    }
}

impl FromIterator<Child> for Children {
    fn from_iter<T: IntoIterator<Item = Child>>(iter: T) -> Self {
        Self {
            inner: iter.into_iter().map(|child| (child.id(), child)).collect(),
        }
    }
}

impl Extend<Child> for Children {
    fn extend<T: IntoIterator<Item = Child>>(&mut self, iter: T) {
        self.inner
            .extend(iter.into_iter().map(|child| (child.id(), child)));
    }
}

/// Register the `SIGCHLD` signal handler. Once the signal is received,
/// the flag will be set true. User needs to manually clear the flag.
pub fn register_child_hook(flag: Arc<AtomicBool>) {
    _ = signal_hook::flag::register(signal_hook::consts::signal::SIGCHLD, flag)
        .map_err(|err| tracing::error!("Cannot register SIGCHLD signal handler: {:?}", err));
}

/// Sends command to shell for execution
/// Assumes STDIN/STDERR/STDOUT unwanted.
pub fn exec_shell(command: &str, children: &mut Children) -> Option<ChildID> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let pid = child.id();
    children.insert(child);
    Some(pid)
}
