//! Stylesheet byte decoding and host-supplied symbolic locations.

use std::{fmt, sync::Arc};

use encoding_rs::{CoderResult, Encoding, UTF_8, UTF_16BE, UTF_16LE};

use crate::CssSourceSnapshot;

/// A validated canonical encoding from the complete Encoding Standard registry.
/// Labels are ASCII case-insensitive and trim ASCII whitespace. Recognized
/// replacement-encoding labels are valid and never trigger fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssEncoding(&'static Encoding);

impl CssEncoding {
    /// Looks up a standard label, returning null only for an invalid label.
    #[must_use]
    pub fn for_label(label: &str) -> Option<Self> {
        Encoding::for_label(label.as_bytes()).map(Self)
    }

    /// Returns the canonical Encoding Standard name.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.0.name()
    }
}

/// Host-extracted decoding hints. The protocol label may be invalid; an
/// environment encoding has already crossed the standard label boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssStylesheetDecodeHints<'a> {
    /// Label supplied by HTTP or an equivalent protocol, without header parsing.
    pub protocol_label: Option<&'a str>,
    /// Encoding supplied by the referring document.
    pub environment_encoding: Option<CssEncoding>,
}

/// The source that selected the actual decoder encoding.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssEncodingSelection {
    /// A UTF-8 or UTF-16 byte order mark overrode the fallback.
    ByteOrderMark,
    /// A valid protocol label selected the fallback.
    Protocol,
    /// An exact leading ASCII charset declaration within the first 1024 bytes.
    CharsetPrefix,
    /// The referring document's validated encoding.
    Environment,
    /// No earlier source supplied a valid encoding; UTF-8 was used.
    DefaultUtf8,
}

/// Atomic byte-input resource policy. Defaults impose no extra finite cap,
/// following the existing component-input length policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssStylesheetDecodeLimits {
    original_bytes: usize,
    decoded_utf8_bytes: usize,
}

impl CssStylesheetDecodeLimits {
    /// Sets independent original-byte and decoded-UTF-8-byte limits.
    #[must_use]
    pub const fn new(original_bytes: usize, decoded_utf8_bytes: usize) -> Self {
        Self {
            original_bytes,
            decoded_utf8_bytes,
        }
    }

    /// Returns the original-byte limit.
    #[must_use]
    pub const fn max_original_bytes(self) -> usize {
        self.original_bytes
    }

    /// Returns the decoded-UTF-8-byte limit.
    #[must_use]
    pub const fn max_decoded_utf8_bytes(self) -> usize {
        self.decoded_utf8_bytes
    }
}

impl Default for CssStylesheetDecodeLimits {
    fn default() -> Self {
        Self::new(usize::MAX, usize::MAX)
    }
}

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
    crate::error::kind_predicates! { kind;
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
    encoding: CssEncoding,
    selection: CssEncodingSelection,
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
    pub fn encoding(&self) -> CssEncoding {
        self.0.encoding
    }

    /// Returns the source of the actual encoding selection.
    #[must_use]
    pub fn selection(&self) -> CssEncodingSelection {
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
        hints: CssStylesheetDecodeHints<'_>,
        limits: CssStylesheetDecodeLimits,
    ) -> Result<Self, CssInputDecodeError> {
        if bytes.len() > limits.original_bytes {
            return Err(CssInputDecodeError::from(
                InputDecodeKind::OriginalByteLimit,
            ));
        }
        check_capacity(bytes.len())?;
        let mut original = Vec::new();
        original
            .try_reserve_exact(bytes.len())
            .map_err(|_| CssInputDecodeError::from(InputDecodeKind::AllocationFailure))?;
        original.extend_from_slice(bytes);
        let (fallback, fallback_selection) = fallback_encoding(bytes, hints);
        let (text, encoding, had_replacements) =
            decode_bounded(bytes, fallback, limits.decoded_utf8_bytes, 4096)?;
        let selection = if bytes.starts_with(&[0xEF, 0xBB, 0xBF])
            || bytes.starts_with(&[0xFF, 0xFE])
            || bytes.starts_with(&[0xFE, 0xFF])
        {
            CssEncodingSelection::ByteOrderMark
        } else {
            fallback_selection
        };
        let source = CssSourceSnapshot::try_from_owned(text)
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

fn fallback_encoding(
    bytes: &[u8],
    hints: CssStylesheetDecodeHints<'_>,
) -> (CssEncoding, CssEncodingSelection) {
    if let Some(encoding) = hints.protocol_label.and_then(CssEncoding::for_label) {
        return (encoding, CssEncodingSelection::Protocol);
    }
    if let Some(encoding) = prefix_encoding(bytes) {
        return (encoding, CssEncodingSelection::CharsetPrefix);
    }
    if let Some(encoding) = hints.environment_encoding {
        return (encoding, CssEncodingSelection::Environment);
    }
    (CssEncoding(UTF_8), CssEncodingSelection::DefaultUtf8)
}

fn prefix_encoding(bytes: &[u8]) -> Option<CssEncoding> {
    let prefix = bytes
        .get(..bytes.len().min(1024))?
        .strip_prefix(b"@charset \"")?;
    let quote = prefix.iter().position(|byte| *byte == b'"')?;
    if prefix.get(quote + 1) != Some(&b';') || !prefix[..quote].is_ascii() {
        return None;
    }
    let encoding = Encoding::for_label(&prefix[..quote])?;
    Some(CssEncoding(
        if encoding == UTF_16LE || encoding == UTF_16BE {
            UTF_8
        } else {
            encoding
        },
    ))
}

fn check_capacity(length: usize) -> Result<(), CssInputDecodeError> {
    if length > isize::MAX as usize {
        Err(CssInputDecodeError::from(InputDecodeKind::CapacityOverflow))
    } else {
        Ok(())
    }
}

// A fixed fresh buffer is independent of the remaining output budget. Admit
// complete returned output afterward, so small budgets cannot stall the codec.
// The size parameter is private; tests exercise stateful four-byte continuation.
fn decode_bounded(
    bytes: &[u8],
    encoding: CssEncoding,
    max_bytes: usize,
    chunk_size: usize,
) -> Result<(String, CssEncoding, bool), CssInputDecodeError> {
    let mut decoder = encoding.0.new_decoder();
    let mut buffer = [0_u8; 4096];
    let mut chunk_size = chunk_size.clamp(4, buffer.len());
    let mut remaining = bytes;
    let mut output = String::new();
    let mut had_replacements = false;
    loop {
        let (result, read, written, replaced) =
            decoder.decode_to_utf8(remaining, &mut buffer[..chunk_size], true);
        remaining = &remaining[read..];
        had_replacements |= replaced;
        let length = output
            .len()
            .checked_add(written)
            .ok_or(CssInputDecodeError::from(InputDecodeKind::CapacityOverflow))?;
        if length > max_bytes {
            return Err(CssInputDecodeError::from(
                InputDecodeKind::DecodedUtf8ByteLimit,
            ));
        }
        check_capacity(length)?;
        output
            .try_reserve_exact(written)
            .map_err(|_| CssInputDecodeError::from(InputDecodeKind::AllocationFailure))?;
        // The safe codec guarantees complete valid UTF-8 in every written slice.
        output.push_str(
            std::str::from_utf8(&buffer[..written]).expect("encoding_rs guarantees UTF-8"),
        );
        if result == CoderResult::InputEmpty {
            return Ok((output, CssEncoding(decoder.encoding()), had_replacements));
        }
        if read == 0 && written == 0 {
            // A few legacy mappings produce multiple Unicode scalars atomically.
            // Grow a tiny private test chunk while keeping the same decoder.
            chunk_size = chunk_size
                .checked_mul(2)
                .filter(|size| *size <= buffer.len())
                .ok_or(CssInputDecodeError::from(InputDecodeKind::CapacityOverflow))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_byte_chunks_preserve_utf8_suffix_and_final_replacement() {
        let bytes = b"AAA\xE2\x82\xACZ\xE2\x82";
        assert_eq!(
            decode_bounded(bytes, CssEncoding(UTF_8), 10, 4),
            Ok(("AAA€Z�".to_owned(), CssEncoding(UTF_8), true))
        );
        assert_eq!(
            decode_bounded(bytes, CssEncoding(UTF_8), 9, 4),
            Err(CssInputDecodeError::from(
                InputDecodeKind::DecodedUtf8ByteLimit
            ))
        );
        assert_eq!(
            decode_bounded(b"AAA\xE2\x82\xACZ", CssEncoding(UTF_8), 7, 4),
            Ok(("AAA€Z".to_owned(), CssEncoding(UTF_8), false))
        );
    }

    #[test]
    fn four_byte_chunks_preserve_roman_state_and_final_escape_replacement() {
        let encoding = CssEncoding::for_label("iso-2022-jp").unwrap();
        let bytes = b"AAA\x1B(J\\\x1B(BZ\x1B";
        assert_eq!(
            decode_bounded(bytes, encoding, 9, 4),
            Ok(("AAA¥Z�".to_owned(), encoding, true))
        );
        assert_eq!(
            decode_bounded(bytes, encoding, 8, 4),
            Err(CssInputDecodeError::from(
                InputDecodeKind::DecodedUtf8ByteLimit
            ))
        );
        assert_eq!(
            decode_bounded(bytes, encoding, 9, 4),
            Ok(("AAA¥Z�".to_owned(), encoding, true))
        );
    }

    #[test]
    fn four_byte_chunks_report_actual_bom_encoding_and_accumulate_errors() {
        assert_eq!(
            decode_bounded(
                b"\xEF\xBB\xBF\xFFAAA\xE2\x82",
                CssEncoding::for_label("windows-1252").unwrap(),
                9,
                4
            ),
            Ok(("�AAA�".to_owned(), CssEncoding(UTF_8), true))
        );
    }

    #[test]
    fn capacity_overflow_is_typed_without_allocating() {
        assert_eq!(
            check_capacity(usize::MAX),
            Err(CssInputDecodeError::from(InputDecodeKind::CapacityOverflow))
        );
    }
}
