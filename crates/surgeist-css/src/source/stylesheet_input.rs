//! Stylesheet byte decoding and host-supplied symbolic locations.

use std::{fmt, sync::Arc};

use surgeist_css_parser::input::{
    DecodeError, DecodeHints, DecodeLimits, Encoding, EncodingSelection, decode,
};

use crate::source::CssSourceSnapshot;

/// A byte decode failed atomically, without returning partial text or input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssInputDecodeError {
    kind: InputDecodeKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputDecodeKind {
    /// The original byte length exceeds the supplied policy.
    OriginalByteLimit,
    /// The complete decoded UTF-8 length exceeds the supplied policy.
    DecodedUtf8ByteLimit,
    /// A length/capacity cannot be represented by Rust's allocation APIs.
    CapacityOverflow,
    /// A fallible byte, UTF-8 or source-checkpoint reservation failed.
    AllocationFailure,
}
impl From<InputDecodeKind> for CssInputDecodeError {
    fn from(kind: InputDecodeKind) -> Self {
        Self { kind }
    }
}
impl CssInputDecodeError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether the original byte length exceeds the supplied policy.
        is_original_byte_limit => InputDecodeKind::OriginalByteLimit,
        /// Returns whether the complete decoded UTF-8 length exceeds the supplied policy.
        is_decoded_utf8_byte_limit => InputDecodeKind::DecodedUtf8ByteLimit,
        /// Returns whether a length/capacity cannot be represented by Rust's allocation APIs.
        is_capacity_overflow => InputDecodeKind::CapacityOverflow,
        /// Returns whether a fallible byte, UTF-8 or source-checkpoint reservation failed.
        is_allocation_failure => InputDecodeKind::AllocationFailure,
    }
}
impl fmt::Display for CssInputDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match &self.kind {
            InputDecodeKind::OriginalByteLimit => "stylesheet original-byte limit exceeded",
            InputDecodeKind::DecodedUtf8ByteLimit => "stylesheet decoded UTF-8 byte limit exceeded",
            InputDecodeKind::CapacityOverflow => "stylesheet input capacity overflow",
            InputDecodeKind::AllocationFailure => "stylesheet input allocation failed",
        })
    }
}

impl std::error::Error for CssInputDecodeError {}

/// One immutable original byte input and its unfiltered decoded UTF-8 snapshot.
/// Equality compares contents and provenance; [`Self::same_input`] compares
/// invocation identity. Cloning preserves both input and source identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssDecodedStylesheetInput(Arc<DecodedStylesheetData>);

#[derive(Debug, Eq, PartialEq)]
struct DecodedStylesheetData {
    bytes: Vec<u8>,
    source: CssSourceSnapshot,
    encoding: Encoding,
    selection: EncodingSelection,
    had_replacements: bool,
}

impl CssDecodedStylesheetInput {
    /// Returns the unchanged original bytes, including any encoded BOM.
    #[must_use]
    pub fn original_bytes(&self) -> &[u8] {
        &self.0.bytes
    }

    /// Returns the one unfiltered UTF-8 source used by sheet parsing.
    /// Its UTF-8 offsets are not offsets into the original byte stream.
    #[must_use]
    pub fn source(&self) -> &CssSourceSnapshot {
        &self.0.source
    }

    /// Returns the actual encoding, including any BOM override.
    #[must_use]
    pub fn encoding(&self) -> Encoding {
        self.0.encoding
    }

    /// Returns the source of the actual encoding selection.
    #[must_use]
    pub fn selection(&self) -> EncodingSelection {
        self.0.selection
    }

    /// Reports whether malformed sequences required replacement. This is
    /// decoder provenance, not a CSS grammar diagnostic.
    #[must_use]
    pub fn had_replacements(&self) -> bool {
        self.0.had_replacements
    }

    /// Reports shared immutable decode-invocation identity.
    #[must_use]
    pub fn same_input(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// An opaque host-supplied stylesheet URL/location text. The CSS crate preserves
/// it verbatim without URL parsing, canonicalization, resolution or loading.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssStylesheetLocation(String);

impl CssStylesheetLocation {
    /// Retains the host's supplied text, including relative or empty locations.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// Returns the exact supplied text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CssDecodedStylesheetInput {
    /// Decodes a complete byte input according to CSS Syntax's fallback and BOM precedence.
    ///
    /// Decoding retains CR, FF, CRLF and NUL for the existing tokenizer to filter.
    /// An inferred ASCII charset prefix remains in the decoded text. The limits are
    /// atomic and independent: an input over either refuses the whole decode.
    ///
    /// # Errors
    ///
    /// Returns the exhausted byte limit, the failed allocation or the capacity overflow.
    pub fn decode(
        bytes: &[u8],
        hints: DecodeHints<'_>,
        limits: DecodeLimits,
    ) -> Result<Self, CssInputDecodeError> {
        let decoded = decode(bytes, hints, limits).map_err(decode_failure)?;
        let mut original = Vec::new();
        original
            .try_reserve_exact(bytes.len())
            .map_err(|_| CssInputDecodeError::from(InputDecodeKind::AllocationFailure))?;
        original.extend_from_slice(bytes);
        let (encoding, selection, had_replacements) = (
            decoded.encoding(),
            decoded.selection(),
            decoded.had_replacements(),
        );
        let source = CssSourceSnapshot::try_from_owned(decoded.into_text())
            .map_err(|_| CssInputDecodeError::from(InputDecodeKind::AllocationFailure))?;
        Ok(CssDecodedStylesheetInput(Arc::new(DecodedStylesheetData {
            bytes: original,
            source,
            encoding,
            selection,
            had_replacements,
        })))
    }
}

fn decode_failure(error: DecodeError) -> CssInputDecodeError {
    CssInputDecodeError::from(if error.is_original_byte_limit() {
        InputDecodeKind::OriginalByteLimit
    } else if error.is_decoded_utf8_byte_limit() {
        InputDecodeKind::DecodedUtf8ByteLimit
    } else if error.is_capacity_overflow() {
        InputDecodeKind::CapacityOverflow
    } else {
        debug_assert!(error.is_allocation_failure());
        InputDecodeKind::AllocationFailure
    })
}
