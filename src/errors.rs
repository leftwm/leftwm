use std::fmt;
pub type Result<T> = std::result::Result<T, LeftError>;

#[derive(Debug)]
pub struct LeftError {
    inner: LeftErrorKind,
}

#[derive(Debug)]
pub enum LeftErrorKind {
    SerdeParse(serde_json::error::Error),
    IoError(std::io::Error),
    XdgBaseDirError(xdg::BaseDirectoriesError),
    TomlParse(toml::de::Error),
    StreamError(),
}

pub(crate) const fn stream_error() -> LeftError {
    LeftError {
        inner: LeftErrorKind::StreamError(),
    }
}

impl fmt::Display for LeftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Display for LeftErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeftErrorKind::SerdeParse(ref err) => write!(f, "{}", err),
            LeftErrorKind::IoError(ref err) => write!(f, "{}", err),
            LeftErrorKind::XdgBaseDirError(ref err) => write!(f, "{}", err),
            LeftErrorKind::TomlParse(ref err) => write!(f, "{}", err),
            LeftErrorKind::StreamError() => write!(f, "Stream Error"),
        }
    }
}

impl From<LeftErrorKind> for LeftError {
    fn from(inner: LeftErrorKind) -> Self {
        Self { inner }
    }
}

impl From<serde_json::error::Error> for LeftError {
    fn from(inner: serde_json::error::Error) -> Self {
        LeftErrorKind::SerdeParse(inner).into()
    }
}

impl From<std::io::Error> for LeftError {
    fn from(inner: std::io::Error) -> Self {
        LeftErrorKind::IoError(inner).into()
    }
}

impl From<xdg::BaseDirectoriesError> for LeftError {
    fn from(inner: xdg::BaseDirectoriesError) -> Self {
        LeftErrorKind::XdgBaseDirError(inner).into()
    }
}

impl From<toml::de::Error> for LeftError {
    fn from(inner: toml::de::Error) -> Self {
        LeftErrorKind::TomlParse(inner).into()
    }
}
