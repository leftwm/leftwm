//! Error handling and reporting for this backend

use std::ffi::{FromVecWithNulError, IntoStringError, NulError};

use thiserror::Error;
use x11rb::rust_connection::{ConnectionError, ReplyError, ReplyOrIdError};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("")]
    NoSizingHints,
    #[error("Unable to find the root window.")]
    RootWindowNotFound,

    // String conversion
    #[error("Provided string contains null byte.")]
    FromStrWithNull(#[from] NulError),
    #[error("Provided bytes contains null byte.")]
    FromVecWithNull(#[from] FromVecWithNulError),
    #[error(transparent)]
    IntoString(#[from] IntoStringError),

    // Errors from x11rb
    #[error("Connection error occured: {0}")]
    ConnectionError(#[from] ConnectionError),

    #[error("Unable to parse reply: {0}")]
    ReplyError(#[from] ReplyError),

    #[error("Unable to parse reply: {0}")]
    ReplyOrIdError(#[from] ReplyOrIdError),
}
