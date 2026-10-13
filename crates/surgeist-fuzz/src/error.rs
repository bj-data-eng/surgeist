#![forbid(unsafe_code)]

use std::{error::Error, fmt};

/// Result returned by the fuzz contracts and their owner adapters.
pub type Result<T> = std::result::Result<T, FuzzError>;

/// Stable semantic category; diagnostics are not failure identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FuzzErrorKind {
    InvalidIdentity,
    InvalidDescriptor,
    InvalidLimits,
    InvalidObservation,
    InvalidCase,
    UnsupportedFormat,
    MissingCapability,
    InsufficientCoverage,
    Registry,
    Codec,
    Harness,
    Cli,
    Io,
    Artifact,
    Process,
    Timeout,
    Cancelled,
    Resource,
}

/// Operation diagnostic with an optional safe dependency source.
#[derive(Debug)]
pub struct FuzzError {
    kind: FuzzErrorKind,
    operation: String,
    detail: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl FuzzError {
    pub fn new(
        kind: FuzzErrorKind,
        operation: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            operation: operation.into(),
            detail: detail.into(),
            source: None,
        }
    }

    pub fn with_source<E>(
        kind: FuzzErrorKind,
        operation: impl Into<String>,
        detail: impl Into<String>,
        source: E,
    ) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            kind,
            operation: operation.into(),
            detail: detail.into(),
            source: Some(Box::new(source)),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> FuzzErrorKind {
        self.kind
    }
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for FuzzError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.detail)
    }
}

impl Error for FuzzError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

pub(crate) fn invalid(kind: FuzzErrorKind, detail: impl Into<String>) -> FuzzError {
    FuzzError::new(kind, "validate fuzz contract", detail)
}
