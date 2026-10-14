//! Controlled HTML-owned operations, driven by a caller's admission policy.
//!
//! Actions describe their work before reading their operands. This module owns
//! value invariants; callers retain cancellation, work and allocation accounting.

pub use crate::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlAttributeValidationLayout, HtmlValidatedAttributes,
};
