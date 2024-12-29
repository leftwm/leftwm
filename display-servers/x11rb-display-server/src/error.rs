//! Error handling and reporting for this backend

use std::{
    backtrace::Backtrace,
    ffi::{IntoStringError, NulError},
    fmt::Display,
    num::{ParseIntError, TryFromIntError},
    string::FromUtf8Error,
};

use x11rb::rust_connection::{ConnectionError, ReplyError, ReplyOrIdError};

pub(crate) type Result<T> = std::result::Result<T, BackendError>;

/// An error originating from this backend
///
/// # `thiserror`
///
/// It is not possible to use `thiserror` for helping here because it currently relies on a nightly
/// feature (`error_generic_member_access`) for supporting backtrace.
///
/// - `thiserror` PR: <https://github.com/dtolnay/thiserror/pull/246>
/// - features tracking issue: <https://github.com/rust-lang/rust/issues/99301>
#[derive(Debug)]
pub(crate) struct BackendError {
    pub src: Option<Box<dyn std::error::Error + 'static>>,
    pub msg: &'static str,
    pub backtrace: Backtrace,
    pub kind: ErrorKind,
}

impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.src {
            Some(src) => Some(src.as_ref()),
            None => None,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.kind.fmt(f);
        f.debug_list().entry(&kind).finish()?;
        f.write_str(" ")?;
        f.write_str(self.msg)?;
        if let Some(e) = &self.src {
            f.write_str(": ")?;
            e.fmt(f)?;
        };
        f.write_str("\nBacktrace:\n")?;
        self.backtrace.fmt(f)
    }
}

/// The possible errors
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    RootWindowNotFound,
    StringConversion,
    IntConversion,

    // Errors from x11rb
    XConnection,
    XReply,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ErrorKind::RootWindowNotFound => "RootWindowNotFound",
            ErrorKind::StringConversion => "StringConversion",
            ErrorKind::IntConversion => "IntConversion",
            ErrorKind::XConnection => "XConnection",
            ErrorKind::XReply => "XReply",
        };
        f.write_str(msg)?;
        Ok(())
    }
}

/// Implement From<T> for given error
macro_rules! from_err {
    ($e:ty, $kind:expr, $msg:literal) => {
        impl core::convert::From<$e> for BackendError {
            fn from(value: $e) -> Self {
                Self {
                    src: Some(Box::new(value)),
                    msg: $msg,
                    backtrace: Backtrace::capture(),
                    kind: $kind,
                }
            }
        }
    };
}

// Conversion
from_err!(
    NulError,
    ErrorKind::StringConversion,
    "Unable to parse nul terminated string"
);
from_err!(
    FromUtf8Error,
    ErrorKind::StringConversion,
    "Unable to parse utf-8 string"
);
from_err!(
    IntoStringError,
    ErrorKind::StringConversion,
    "Unable to convert value into String"
);
from_err!(
    ParseIntError,
    ErrorKind::StringConversion,
    "Unable to parse int with given base"
);
from_err!(
    TryFromIntError,
    ErrorKind::IntConversion,
    "Unable to parse int from a value"
);

// X11 Errors
from_err!(
    ConnectionError,
    ErrorKind::XConnection,
    "Error in connection to the X server"
);
from_err!(ReplyError, ErrorKind::XReply, "Error when parsing reply");
from_err!(
    ReplyOrIdError,
    ErrorKind::XReply,
    "Error when parsing reply"
);
