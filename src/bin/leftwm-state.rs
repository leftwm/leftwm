extern crate leftwm;
use std::io::prelude::*;
use std::io::Result;
use std::os::unix::net::UnixStream;
use std::str;
use xdg::BaseDirectories;

fn main() -> Result<()> {
    let base = BaseDirectories::with_prefix("leftwm")?;
    let socket_file = base.place_runtime_file("current_state.sock")?;
    let mut stream = UnixStream::connect(socket_file)?;
    let mut buffer = [0; 4096];
    let mut running = true;

    while running {
        match stream.read(&mut buffer) {
            Ok(size) => {
                let raw = str::from_utf8(&buffer[0..size]).unwrap();
                if let Some(raw) = raw.lines().last() {
                    println!("{}", raw);
                }
            }
            Err(_) => running = false,
        }
    }
    Ok(())
}
