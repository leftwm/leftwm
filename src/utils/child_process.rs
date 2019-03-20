use std::env;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Nanny {}

impl Nanny {
    pub fn new() -> Nanny {
        Nanny {}
    }

    pub fn autostart(&self) {
        if let Some(mut path) = env::home_dir() {
            path.push(".config");
            path.push("autostart");
            if let Ok(files) = list_desktop_files(&path) {
                for file in files {
                    let _ = boot_desktop_file(&file);
                    println!("PATH: {:?}", file);
                }
            }
        }
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
