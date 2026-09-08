use std::{error, fmt};

use crate::{NumericKind, negative_code};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
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
    NonFinite,
    NegativeSize,
    NegativeRadius,
    InvalidPath,
    InvalidStroke,
    InvalidDash,
    EmptyPath,
    UnsupportedStrokeBounds,
}
pub(crate) fn validate_finite(value: f64, name: &str) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorCode::NonFinite,
            format!("{name} must be finite"),
        ))
    }
}

pub(crate) fn validate_non_negative_kind(value: f64, name: &str, kind: NumericKind) -> Result<()> {
    validate_finite(value, name)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(Error::new(
            negative_code(kind),
            format!("{name} must be non-negative"),
        ))
    }
}
