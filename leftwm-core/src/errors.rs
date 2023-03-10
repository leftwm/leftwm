use thiserror::Error;

pub type Result<T> = std::result::Result<T, LeftError>;

#[derive(Debug, Error)]
pub enum LeftError {
    // TODO move StateSocket away from lib OR use Config::save_state?
    #[error("Parsing error: {0}")]
    SerdeParse(#[from] serde_json::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    // TODO move Nanny to bin
    #[error("XDG error: {0}")]
    XdgBaseDirError(#[from] xdg::BaseDirectoriesError),
    #[error("Stream error")]
    StreamError,
    #[error("Liquid parsing error")]
    LiquidParsingError,
}
