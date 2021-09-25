use thiserror::Error;

pub type Result<T> = std::result::Result<T, LeftError>;

#[derive(Debug, Error)]
pub enum LeftError {
    #[error("Parsing error: {0}")]
    SerdeParse(#[from] serde_json::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("XDG error: {0}")]
    XdgBaseDirError(#[from] xdg::BaseDirectoriesError),
    #[error("Stream error")]
    StreamError,
}
