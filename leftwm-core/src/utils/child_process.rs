//! Starts programs in autostart, runs global 'up' script, and boots theme. Provides function to
//! boot other desktop files also.
use crate::errors::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::iter::{Extend, FromIterator};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{atomic::AtomicBool, Arc};
use xdg::BaseDirectories;

type ChildID = u32;

#[derive(Default)]
pub struct Nanny {}

impl Nanny {
    /// As [Desktop Application Autostart Specification](https://specifications.freedesktop.org/autostart-spec/autostart-spec-latest.html) describe,
    /// some applications placing an application's `.desktop` file in one of the *Autostart Directories*
    /// could be automatically launched during startup of the user's desktop environment after the user has logged in.
    ///
    /// The *Autostart Directories* are `$XDG_CONFIG_DIRS/autostart` and `$XDG_CONFIG_HOME/autostart`.
    /// `$XDG_CONFIG_DIRS` and `$XDG_CONFIG_HOME` can be found in
    /// [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/).
    ///
    ///
    /// There are some principles about autostart file:
    /// 1. An application `.desktop` file must have the format as defined in the [Desktop Entry Specification](http://standards.freedesktop.org/desktop-entry-spec/)
    /// 2. If two files have the same filename in `$XDG_CONFIG_DIRS/autostart` and `$XDG_CONFIG_HOME/autostart`,
    /// e.g. `foo.desktop`, `$XDG_CONFIG_DIRS/autostart/foo.desktop` will be ignored.
    ///
    /// `Autostart Entry` will be ignored when:
    /// 1. the `.desktop` file has the `Hidden` key set to true.
    /// 2. string identifying the desktop environments not in `OnlyShowIn`
    /// 3. string identifying the desktop environments in `NotShowIn`
    ///
    /// The string identifying the desktop environments means `$XDG_CURRENT_DESKTOP`,
    /// you can find some from [Registered `OnlyShowIn` Environments](https://specifications.freedesktop.org/menu-spec/latest/apb.html).  
    /// `LeftWM` use **`LeftWM`** as identification (case-sensitive).
    #[must_use]
    pub fn autostart() -> Children {
        BaseDirectories::new()
            .map(|xdg_dir| {
                xdg_dir
                    .list_config_files_once("autostart")
                    .iter()
                    .filter(|path| path.extension() == Some(std::ffi::OsStr::new("desktop")))
                    .filter_map(|file| boot_desktop_file(file).ok())
                    .collect::<Children>()
            })
            .unwrap_or_default()
    }

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
    fn run_script(path: &Path) -> Result<Option<Child>> {
        if path.is_file() {
            Command::new(&path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .map(Some)
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    /// Runs the 'up' script in the config directory, if there is one.
    ///
    /// # Errors
    ///
    /// Will error if unable to open current config directory.
    /// Could be caused by inadequate permissions.
    pub fn run_global_up_script() -> Result<Option<Child>> {
        let mut path = Self::get_config_dir()?;
        path.push("up");
        Self::run_script(&path)
    }

    /// Runs the 'up' script of the current theme, if there is one.
    ///
    /// # Errors
    ///
    /// Will error if unable to open current theme directory.
    /// Could be caused by inadequate permissions.
    pub fn boot_current_theme() -> Result<Option<Child>> {
        let mut path = Self::get_config_dir()?;
        path.push("themes");
        path.push("current");
        path.push("up");
        Self::run_script(&path)
    }
}

#[derive(Debug, thiserror::Error)]
enum EntryBootError {
    #[error("execute failed: {0}")]
    Execute(#[from] std::io::Error),

    #[error("invalid desktop (current {current:?})")]
    NotForThisDesktop { current: String },

    #[error("entry hidden")]
    Hidden,

    #[error("no exec")]
    NoExec,
}

fn boot_desktop_file(path: &Path) -> std::result::Result<Child, EntryBootError> {
    let entry = DesktopEntry::parse_file(path)?;
    let env_curr_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    if let Some(only_show_in) = entry.only_show_in {
        if !only_show_in.contains(&env_curr_desktop) {
            return Err(EntryBootError::NotForThisDesktop {
                current: env_curr_desktop,
            });
        }
    }
    if let Some(not_show_in) = entry.not_show_in {
        if not_show_in.contains(&env_curr_desktop) {
            return Err(EntryBootError::NotForThisDesktop {
                current: env_curr_desktop,
            });
        }
    }

    if entry.hidden {
        return Err(EntryBootError::Hidden);
    }

    if entry.exec.is_none() {
        return Err(EntryBootError::NoExec);
    }
    let wd = entry
        .path
        .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_else(|| PathBuf::from(".")));

    Command::new("sh")
        .current_dir(wd)
        .arg("-c")
        .arg(entry.exec.unwrap())
        .spawn()
        .map_err(EntryBootError::Execute)
}

/// Refer to [Recognized desktop entry keys](https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s06.html)
#[derive(Debug, Default)]
struct DesktopEntry {
    // TryExec: Option<String>,
    exec: Option<String>,
    path: Option<PathBuf>,
    only_show_in: Option<HashSet<String>>,
    not_show_in: Option<HashSet<String>>,
    hidden: bool,
}

impl DesktopEntry {
    fn parse_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse(content.as_str()))
    }
    fn parse(content: &str) -> Self {
        let mut in_main_section = false;
        let mut entry: Self = Default::default();
        for mut line in content.lines() {
            line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                if line == "[Desktop Entry]" {
                    in_main_section = true;
                    continue;
                }
                in_main_section = false;
            }

            if !in_main_section {
                continue;
            }

            if let Some((key, value)) = Self::split_line(line) {
                match key {
                    "Exec" => entry.exec = Some(value.to_string()),
                    "Path" => entry.path = Some(PathBuf::from(value)),
                    "OnlyShowIn" => entry.only_show_in = Some(Self::split_to_set(value)),
                    "NotShowIn" => entry.not_show_in = Some(Self::split_to_set(value)),
                    "Hidden" => entry.hidden = Self::str_bool(value).unwrap_or_default(),
                    _ => {}
                }
            }
        }
        entry
    }

    fn split_line(line: &str) -> Option<(&str, &str)> {
        line.find('=')?; //Check we have an equals, if we don't return None
        line.split_once('=')
    }
    fn split_to_set(value: &str) -> HashSet<String> {
        value
            .split(';')
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }
                Some(s.to_string())
            })
            .collect::<HashSet<String>>()
    }
    fn str_bool(value: &str) -> Option<bool> {
        value.to_lowercase().parse::<bool>().ok()
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
        self.inner.extend(reaper.inner.into_iter());
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
            inner: iter
                .into_iter()
                .map(|child| (child.id(), child))
                .collect::<HashMap<_, _>>(),
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
    let _ = signal_hook::flag::register(signal_hook::consts::signal::SIGCHLD, flag)
        .map_err(|err| log::error!("Cannot register SIGCHLD signal handler: {:?}", err));
}

/// Sends command to shell for execution
/// Assumes STDIN/STDOUT unwanted.
pub fn exec_shell(command: &str, children: &mut Children) -> Option<u32> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .ok()?;
    let pid = child.id();
    children.insert(child);
    Some(pid)
}

#[cfg(test)]
mod tests {

    use super::DesktopEntry;

    #[test]
    fn test_parse() {
        let content = r###"
            [Desktop Action Gallery]
        Exec=fooview --gallery
        Name=Browse Gallery
                [Desktop Entry]
        #comment
        Name=Optimus Manager
        Name[zh_CN]=Optimus \u{7ba1}\u{7406}\u{5668}
        Comment=A program to handle GPU switching on Optimus laptops
        Comment[ru]=\u{41f}\u{440}\u{43e}\u{433}\u{440}\u{430}\u{43c}\u{43c}\u{430} \u{434}\u{43b}\u{44f} \u{443}\u{43f}\u{440}\u{430}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{44f} \u{43f}\u{435}\u{440}\u{435}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{438}\u{435}\u{43c} \u{433}\u{440}\u{430}\u{444}\u{438}\u{447}\u{435}\u{441}\u{43a}\u{438}\u{445} \u{43f}\u{440}\u{43e}\u{446}\u{435}\u{441}\u{441}\u{43e}\u{440}\u{43e}\u{432} \u{43d}\u{430} \u{43d}\u{43e}\u{443}\u{442}\u{431}\u{443}\u{43a}\u{430}\u{445} c Optimus
        Comment[zh_CN]=\u{5904}\u{7406}\u{53cc}\u{663e}\u{5361}\u{7b14}\u{8bb0}\u{672c}\u{7535}\u{8111} GPU \u{5207}\u{6362}\u{7684}\u{7a0b}\u{5e8f}
        Keywords=nvidia;optimus;settings;switch;GPU;
        Keywords[ru]=nvidia;optimus;settings;switch;GPU;\u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438};\u{432}\u{438}\u{434}\u{435}\u{43e}\u{43a}\u{430}\u{440}\u{442}\u{430};
        Exec=optimus-manager-qt
        Icon=optimus-manager-qt
        Terminal=false
        StartupNotify=false
        Type=Application
        Categories=System;Settings;Qt;
        Actions=Gallery;Create;
        Hidden=true
        OnlyShowIn=XFCE;

        [Desktop Action Create]
        Exec=fooview --create-new
        Name=Create a new Foo!
        Icon=fooview-new
                "###;

        let entry = DesktopEntry::parse(content);

        assert_eq!(
            entry.exec,
            Some("optimus-manager-qt".to_string()),
            "exec failed"
        );
        assert!(entry.path.is_none(), "expect path none");
        assert!(entry.hidden, "expect hidden true");
        assert!(entry.only_show_in.is_some(), "expect only_show_in defined");

        assert!(
            entry.only_show_in.clone().unwrap().contains("XFCE"),
            "expect only_show_in contains XFCE"
        );
        assert!(
            !entry.only_show_in.clone().unwrap().contains(""),
            "expect only show in not contains empty-str"
        );
        assert!(entry.not_show_in.is_none(), "expect not_show_in none");
    }
}
