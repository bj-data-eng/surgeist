use std::{error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl error::Error for Error {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    InvalidValue,
    InvalidProperty,
    InvalidSelector,
    InvalidString,
    MissingNode,
    Traversal,
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn validate_finite(value: f32, name: &str) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{name} must be finite"),
        ))
    }
}

pub(crate) fn validate_non_negative(value: f32, name: &str) -> Result<()> {
    validate_finite(value, name)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{name} must be non-negative"),
        ))
    }
}
