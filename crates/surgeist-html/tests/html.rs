//! The crate's single integration-test binary.
//!
//! Tests use only the public API. Modules group them by the language domain
//! they exercise, following the crate's current source modules (attributes,
//! bounded operations, construction, values, temporal values, color, focus,
//! dialog, document, forms, input, navigation, hyperlinks, scripting and
//! element rules); the public API itself is still a flat re-export apart
//! from `bounded`.

#[path = "html/mod.rs"]
mod html;
