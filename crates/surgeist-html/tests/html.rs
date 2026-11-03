//! The crate's single integration-test binary.
//!
//! Tests use only the public API. Modules group them by the language domain
//! they exercise, following the crate's public modules (attributes,
//! bounded operations, construction, values, temporal values, color, focus,
//! dialog, document, forms, input, navigation, hyperlinks, scripting and
//! element rules).

#[path = "html/mod.rs"]
mod html;
