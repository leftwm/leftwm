use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
};

pub struct ReturnPipe {
    pipe_file: PathBuf,
    rx: mpsc::UnboundedReceiver<String>,
}

impl Drop for ReturnPipe {
    fn drop(&mut self) {
        use std::os::unix::fs::OpenOptionsExt;
        self.rx.close();

        // Open fifo for write to unblock pending open for read operation that prevents tokio runtime
        // from shutting down.
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
            .open(&self.pipe_file)
        {
            Err(err) => tracing::error!(
                "Failed to open {} when dropping ReturnPipe: {err}",
                self.pipe_file.display()
            ),
            Ok(f) => drop(f),
        };
        match std::fs::remove_file(&self.pipe_file) {
            Ok(_) => {}
            Err(e) => tracing::error!("Failed to delete pipe file: {e}"),
        }
    }
}

impl ReturnPipe {
    /// Create and listen to the named pipe.
    /// # Errors
    ///
    /// Will error if unable to `mkfifo`, likely a filesystem issue
    /// such as inadequate permissions.
    pub async fn new(pipe_file: PathBuf) -> Result<Self, std::io::Error> {
        fs::remove_file(pipe_file.as_path()).await.ok();
        if let Err(e) = nix::unistd::mkfifo(&pipe_file, nix::sys::stat::Mode::S_IRWXU) {
            tracing::error!("Failed to create new fifo {:?}", e);
        }

        let path = pipe_file.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while !tx.is_closed() {
                read_from_pipe(&path, &tx).await;
            }
            fs::remove_file(path).await.ok();
        });

        Ok(Self { pipe_file, rx })
    }

    pub fn pipe_name() -> PathBuf {
        let display = env::var("DISPLAY")
            .ok()
            .and_then(|d| d.rsplit_once(':').map(|(_, r)| r.to_owned()))
            .unwrap_or_else(|| String::from("0"));

        PathBuf::from(format!("return-{display}.pipe"))
    }

    pub async fn read_return(&mut self) -> Option<String> {
        self.rx.recv().await
    }
}

async fn read_from_pipe(pipe_file: &Path, tx: &mpsc::UnboundedSender<String>) -> Option<()> {
    let file = fs::File::open(pipe_file).await.ok()?;
    let mut lines = BufReader::new(file).lines();

    while let Some(line) = lines.next_line().await.ok()? {
        tx.send(line).ok()?;
    }

    Some(())
}
