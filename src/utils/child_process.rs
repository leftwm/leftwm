use dirs;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use xdg::BaseDirectories;

pub struct Nanny {}

impl Default for Nanny {
    fn default() -> Self {
        Self::new()
    }
}
impl Nanny {
    pub fn new() -> Nanny {
        Nanny {}
    }

    pub fn autostart(&self) {
        dirs::home_dir()
            .and_then(|mut path| {
                path.push(".config");
                path.push("autostart");
                Some(path)
            })
            .and_then(|path| list_desktop_files(&path).ok())
            .and_then(|files| {
                files.iter().for_each(|file| {
                    let _ = boot_desktop_file(&file);
                });
                Some(files)
            });
    }

    pub fn boot_current_theme(&self) -> Result<(), Box<std::error::Error>> {
        let mut path = BaseDirectories::with_prefix("leftwm")?.create_config_directory("")?;
        path.push("themes");
        path.push("current");
        path.push("up");
        if path.is_file() {
            Command::new(&path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .spawn()?;
        }
        Ok(())
    }
}

fn boot_desktop_file(path: &PathBuf) -> std::result::Result<std::process::Child, std::io::Error> {
    let args = format!( "`grep '^Exec' {:?} | tail -1 | sed 's/^Exec=//' | sed 's/%.//' | sed 's/^\"//g' | sed 's/\" *$//g'`", path );
    Command::new("sh").arg("-c").arg(args).spawn()
}

// get all the .desktop files in a folder
fn list_desktop_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut list = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "desktop" {
                        list.push(path);
                    }
                }
            }
        }
    }
    Ok(list)
}
