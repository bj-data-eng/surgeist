//! The crate's single integration-test binary.
//!
//! Tests use only the public API. Modules group them by the CSS domain
//! they exercise, following the crate's current source modules; the
//! public API itself is still a flat re-export.

#![forbid(unsafe_code)]

#[path = "css/mod.rs"]
mod css;
