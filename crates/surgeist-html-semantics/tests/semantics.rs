//! The crate's single integration-test binary.
//!
//! Tests use only the public API. Modules group them by the semantic domain
//! they exercise, following the crate's current source modules (base facts,
//! capture, controlled option values, hyperlinks and the semantic query
//! domains); the public API itself is still a flat re-export.

#[path = "semantics/mod.rs"]
mod semantics;
