//! Cross-crate Surgeist integration adapters.
//!
//! Backend-local adapters remain in their owning crates. This module only
//! composes public Surgeist crate contracts.

mod error;

#[cfg(test)]
mod tests;

pub use error::{AdapterBoundary, AdapterError, AdapterErrorKind, AdapterResult};
