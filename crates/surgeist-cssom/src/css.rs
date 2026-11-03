//! Rust CSS namespace. Identifier escaping belongs to the CSS syntax owner;
//! this front preserves its cumulative resource limits and typed failures.

use surgeist_css::serialization::{CssSerializationError, CssSerializationLimits};

/// Escapes `value` as a CSS identifier, as CSSOM's `CSS.escape()` does.
///
/// Every code point that is not an ident code point of CSS Syntax 3 §4.2 is
/// escaped, so for non-ASCII code points this is stricter than CSSOM §2.1,
/// which writes every code point from U+0080 as itself; for example U+00A0
/// becomes `\a0 `. Empty text escapes to empty text and NUL becomes U+FFFD.
/// The identifier grammar of a property is not checked. `limits` bounds the
/// work and the UTF-8 output length.
///
/// # Errors
///
/// Returns [`CssSerializationError`] when a limit is exhausted; no partial
/// text is returned.
pub fn escape(
    value: &str,
    limits: CssSerializationLimits,
) -> Result<String, CssSerializationError> {
    surgeist_css::serialization::serialize_css_identifier(value, limits)
}
