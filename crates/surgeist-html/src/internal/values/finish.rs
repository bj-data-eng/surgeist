//! The refusal of a consuming finish that hands its owner back.

use std::{error::Error, fmt};

/// A finish its owner refused.
///
/// A finish consumes the owner it completes, so an owner that is not ready cannot stay in a
/// caller's slot. The refusal returns it unchanged, together with the cause, so the caller can
/// drive the missing step, recover its operands or drop it deliberately. The owner is never
/// dropped on this path.
#[derive(Debug)]
pub struct HtmlFinishFailure<T, E> {
    owner: T,
    cause: E,
}

impl<T, E> HtmlFinishFailure<T, E> {
    pub(crate) fn new(owner: T, cause: impl Into<E>) -> Self {
        Self {
            owner,
            cause: cause.into(),
        }
    }
    /// The cause the finish was refused for.
    pub fn cause(&self) -> &E {
        &self.cause
    }
    /// The owner, unchanged.
    pub fn owner(&self) -> &T {
        &self.owner
    }
    /// Returns the unchanged owner and the cause. Nothing the caller supplied is lost.
    pub fn into_parts(self) -> (T, E) {
        (self.owner, self.cause)
    }
}

impl<T, E: fmt::Display> fmt::Display for HtmlFinishFailure<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "finish refused: {}", self.cause)
    }
}

impl<T: fmt::Debug, E: Error + 'static> Error for HtmlFinishFailure<T, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
