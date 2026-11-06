//! Work admitted before frontend operations.

use std::fmt;

use crate::source::ReadUnit;

/// Work about to be performed by the frontend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendWork {
    /// One decoder input attempt.
    Read(ReadUnit),
    /// Consume one decoded point and retain its UTF-16 representation together.
    Consume {
        /// Width of the decoded point's UTF-16 representation.
        retained_units: u8,
    },
}

/// The frontend budget dimension that refused work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendLimit {
    /// Maximum admitted decoder input attempts.
    Read(u64),
    /// Maximum admitted decoded code points.
    Consumption(u64),
    /// Maximum admitted retained UTF-16 units.
    Retention(u64),
}

impl fmt::Display for FrontendLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (limit, units) = match self {
            Self::Read(limit) => (limit, "source reads"),
            Self::Consumption(limit) => (limit, "consumed source code points"),
            Self::Retention(limit) => (limit, "retained UTF-16 source units"),
        };
        write!(
            formatter,
            "reached the operation's limit of {limit} {units}"
        )
    }
}

impl std::error::Error for FrontendLimit {}

/// The caller's admission policy for frontend work.
pub type FrontendControl<'control> =
    dyn FnMut(FrontendWork) -> Result<(), FrontendLimit> + Send + Sync + 'control;

pub(crate) struct Control<'control>(pub(crate) Option<&'control mut FrontendControl<'control>>);

impl Control<'_> {
    #[inline]
    pub(crate) fn admit(&mut self, work: FrontendWork) -> Result<(), FrontendLimit> {
        self.0.as_deref_mut().map_or(Ok(()), |admit| admit(work))
    }
}

impl fmt::Debug for Control<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Control")
            .field(&self.0.is_some())
            .finish()
    }
}
