//! Component values, token views, serialized values and their limits and errors.
//!
//! A component value is a checked, immutable CSS Syntax token, function or simple block.
//! Every component carries a [`CssValueOrigin`], so provenance stays honest for parsed and
//! programmatic input alike. Parsing and construction obey [`CssComponentValueLimits`], and
//! `serialize_tokens` returns token-preserving text with a map back to the originating
//! syntax ([`CssSerializedValue`]). Typed values elsewhere in the crate are built from, and can be
//! turned back into, component values.

pub(crate) mod values;

#[doc(inline)]
pub use values::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValueRef, CssComponentValues, CssFunctionValue, CssHashFlag, CssNumericTokenKind,
    CssNumericTokenRef, CssSerializedOrigin, CssSerializedOriginSegment, CssSerializedValue,
    CssSimpleBlock, CssValueOrigin, CssValueTokenRef,
};
