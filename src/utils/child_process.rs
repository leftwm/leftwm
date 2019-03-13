use crate::models::Manager;
use crate::utils::logging;
use std::collections::HashSet;
use std::io::prelude::*;
use std::process::{Command, Stdio};

//static CHILD_LIST: &'static [&'static str] =
//    &["compton", "lemonbar -B '#99ffffff' -F '#000000' -d"];
static CHILD_LIST: &'static [&'static str] = &["lemonbar -B #aaffffff -F #000000"];

pub struct Nanny {
    children: Vec<ChildProcess>,
}

impl Default for Nanny {
    fn default() -> Nanny {
        Nanny { children: vec![] }
    }
}

impl Drop for Nanny {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.process.kill();
        }
    }
}

impl Nanny {
    pub fn new() -> Nanny {
        Nanny::default()
    }

    pub fn update_children(&mut self, manager: &Manager) {
        self.boot_missing_children();

        let workspaces = manager.workspaces_display();
        let tags = manager.tags_display();
        let state = format!(" {}        {}\n", tags, workspaces);
        let _ = self.send_state_to_stdin(&state);

        //cleanup dead sockets
        self.children.retain(|s| s.is_alive());
    }

    fn boot_missing_children(&mut self) {
        let not_running = self.find_not_running();

        for cmd in not_running {
            if let Ok(child) = ChildProcess::new(cmd) {
                self.children.push(child);
            }
        }
    }

    fn find_not_running(&self) -> Vec<String> {
        let all: HashSet<String> = CHILD_LIST.iter().map(|s| s.to_string()).collect();
        let running: HashSet<String> = (&self.children).iter().map(|c| c.cmd.clone()).collect();
        all.difference(&running).map(|s| s.clone()).collect()
    }

    fn send_state_to_stdin(&mut self, info: &str) -> Result<(), Box<std::error::Error>> {
        for child in self.children.iter_mut() {
            let p = &mut child.process;
            let std_op = &mut p.stdin;
            if let Some(std) = std_op {
                let _ = std.write_all(info.as_bytes());
            }
        }
        Ok(())
    }
}

struct ChildProcess {
    cmd: String,
    process: std::process::Child,
}

impl ChildProcess {
    fn new(cmd: String) -> Result<ChildProcess, Box<std::error::Error>> {
        logging::log_info("CHILD_PROCESS:", &cmd);
        let mut parts = cmd.split(" ");
        let head = parts.next().unwrap();
        let arguments: Vec<&str> = parts.collect();
        let mut child = Command::new(head);
        for arg in &arguments {
            child.arg(arg);
        }
        let process = child.stdin(Stdio::piped()).spawn()?;
        Ok(ChildProcess { cmd, process })
    }

    fn is_alive(&self) -> bool {
        true
    }
}
