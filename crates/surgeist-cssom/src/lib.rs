//! Mutable authored CSS ownership and immutable, revisioned captures.
//!
//! CSS owns checked syntax, grammar and formatting. This crate owns live identity,
//! ancestry and atomic publication. It performs no cascade, evaluation or loading.
#![forbid(unsafe_code)]

mod css_adapter;
mod identity;
mod media_operations;
mod model;
mod publication;
mod store;

pub use identity::*;
pub use media_operations::CssomMediaEdit;
pub use model::*;
pub use publication::*;
pub use store::*;
