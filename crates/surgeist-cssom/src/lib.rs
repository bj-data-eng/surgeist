//! Mutable authored CSS ownership and immutable, revisioned captures.
//!
//! CSS owns checked syntax, grammar and formatting. This crate owns live identity,
//! ancestry and atomic publication. It performs no cascade, evaluation or loading.
#![forbid(unsafe_code)]

mod css_adapter;
mod declaration;
mod declaration_support;
mod identity;
mod media_operations;
mod model;
mod owner_effect;
mod publication;
mod readonly_extensions;
mod rule_operations;
mod sheet_operations;
mod store;

pub use declaration::*;
pub use declaration_support::*;
pub use identity::*;
pub use media_operations::CssomMediaEdit;
pub use model::*;
pub use owner_effect::*;
pub use publication::*;
pub use readonly_extensions::*;
pub use sheet_operations::*;
pub use store::*;
