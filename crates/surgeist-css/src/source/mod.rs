//! Source snapshots, coordinates and origins, byte decoding, UTF-16 preparation and retention traversal.
//!
//! A parse input is one immutable [`CssSourceSnapshot`]. Positions and spans address it: byte
//! offsets index the original UTF-8 input, lines are zero-based and columns count UTF-16 code
//! units; UTF-16 ingress addresses the normalized UTF-8 buffer instead, with normalized
//! coordinates. Byte stylesheets are decoded by [`CssDecodedStylesheetInput::decode`] under the
//! Encoding Standard before parsing, and UTF-16 input is prepared under [`CssUtf16InputLimits`].
//! [`CssRetainedSources`] and [`CssSourceRetention`] let a consumer that stores CSS payloads
//! account the source backing they retain under finite limits.

pub(crate) mod position;
pub(crate) mod source_retention;
pub(crate) mod stylesheet_input;
pub(crate) mod utf16_declaration_block;
pub(crate) mod utf16_input;
pub(crate) mod utf16_value;

#[doc(inline)]
pub use position::{
    CssByteOffset, CssLineIndex, CssParsedOrigin, CssSourcePosition, CssSourceSnapshot,
    CssSourceSpan, CssSourceStorageUsage, CssUtf16ColumnIndex,
};
#[doc(inline)]
pub use source_retention::{
    CssRetainedPayloadError, CssRetainedSources, CssSourceRetention, CssSourceRetentionAction,
    CssSourceRetentionError, CssSourceRetentionResource,
};
#[doc(inline)]
pub use stylesheet_input::{
    CssDecodedStylesheetInput, CssEncoding, CssEncodingSelection, CssInputDecodeError,
    CssStylesheetDecodeHints, CssStylesheetDecodeLimits, CssStylesheetLocation,
};
#[doc(inline)]
pub use utf16_declaration_block::CssDeclarationBlockParseReport;
#[doc(inline)]
pub use utf16_input::{CssParseSource, CssUtf16InputError, CssUtf16InputLimits, CssUtf16Resource};
#[doc(inline)]
pub use utf16_value::{CssUtf16PropertyValueLimits, CssUtf16ValueAdmission, CssUtf16ValueStorage};
