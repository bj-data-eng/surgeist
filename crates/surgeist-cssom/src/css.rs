//! Rust CSS namespace over the bounded CSSOM serialization operations.

use surgeist_css::serialization::{CssSerializationError, CssomIdentifierLimits};

/// Serializes a DOMString as an identifier for CSSOM's `CSS.escape()`.
///
/// The input and output are exact UTF-16 code units. NUL becomes U+FFFD.
/// Non-ASCII code units, including isolated surrogates, pass through unchanged.
/// `limits` bounds the input extent and encoded output bytes.
///
/// # Errors
///
/// Returns [`CssSerializationError`] when a limit or allocation refuses the
/// operation. A failure returns no text.
pub fn escape(
    value: &[u16],
    limits: CssomIdentifierLimits,
) -> Result<Vec<u16>, CssSerializationError> {
    surgeist_css::serialization::serialize_cssom_identifier(value, limits)
}
