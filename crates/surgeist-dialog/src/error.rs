use std::{error, fmt};

/// Dialog module result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable dialog diagnostic.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl error::Error for Error {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    BackendUnavailable,
    InvalidOptions,
}
