//! Section 3.2, "The input byte stream": decoding a style sheet's bytes.

use std::fmt;

use encoding_rs::{CoderResult, UTF_8, UTF_16BE, UTF_16LE};

/// An encoding of the Encoding Standard, found by one of its labels.
///
/// Every encoding of the standard's registry is valid, including the
/// `replacement` encoding that the labels of a few legacy encodings select
/// (such as `iso-2022-kr`). Labels are matched ASCII case-insensitively after
/// leading and trailing ASCII whitespace is removed, as the standard's "get an
/// encoding" algorithm says.
///
/// ```
/// use surgeist_css_parser::input::Encoding;
///
/// assert_eq!(Encoding::for_label(" LATIN1\n").unwrap().name(), "windows-1252");
/// assert_eq!(Encoding::for_label("utf8").unwrap().name(), "UTF-8");
/// assert!(Encoding::for_label("no such encoding").is_none());
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Encoding(&'static encoding_rs::Encoding);

impl Encoding {
    /// Looks up an encoding by label, returning `None` when the label names no
    /// encoding of the standard ("get an encoding" returns failure).
    #[must_use]
    pub fn for_label(label: &str) -> Option<Self> {
        encoding_rs::Encoding::for_label(label.as_bytes()).map(Self)
    }

    /// Returns the encoding's canonical name, as the standard spells it.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.0.name()
    }
}

/// What the host knows about a style sheet's encoding besides its bytes.
///
/// These are the inputs of "determine the fallback encoding" that a text-only
/// library cannot find for itself. The host extracts them: the crate does not
/// parse HTTP headers or `<meta>` elements.
///
/// ```
/// use surgeist_css_parser::input::{DecodeHints, Encoding};
///
/// let hints = DecodeHints {
///     protocol_label: Some("windows-1251"),
///     environment_encoding: Encoding::for_label("utf-8"),
/// };
/// assert_eq!(DecodeHints::default().protocol_label, None);
/// # let _ = hints;
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DecodeHints<'a> {
    /// The encoding label that HTTP or an equivalent protocol supplied (the
    /// `charset` parameter of a `Content-Type` header, for example), without
    /// any header parsing. A label that names no encoding is as good as none.
    pub protocol_label: Option<&'a str>,
    /// The environment encoding that the referring document provides, which
    /// has already crossed the label boundary.
    pub environment_encoding: Option<Encoding>,
}

/// Which of the sources of section 3.2 selected the encoding that decoded the
/// bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EncodingSelection {
    /// A UTF-8 or UTF-16 byte order mark, which the Encoding Standard's
    /// "decode" gives precedence over the fallback encoding.
    ByteOrderMark,
    /// A valid label that the protocol supplied.
    Protocol,
    /// An encoding declaration, `@charset "label";`, at the very start of the
    /// first 1024 bytes.
    CharsetPrefix,
    /// The environment encoding of the referring document.
    Environment,
    /// None of the above applied, so the fallback is UTF-8.
    DefaultUtf8,
}

/// The size limits of a decode.
///
/// A decode that would exceed either limit fails as a whole and returns no
/// text. The defaults impose no limit.
///
/// ```
/// use surgeist_css_parser::input::DecodeLimits;
///
/// let limits = DecodeLimits::new(1 << 20, 4 << 20);
/// assert_eq!(limits.max_original_bytes(), 1 << 20);
/// assert_eq!(DecodeLimits::default().max_decoded_utf8_bytes(), usize::MAX);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DecodeLimits {
    original_bytes: usize,
    decoded_utf8_bytes: usize,
}

impl DecodeLimits {
    /// Creates limits of at most `original_bytes` bytes of input and
    /// `decoded_utf8_bytes` bytes of decoded UTF-8 text.
    #[must_use]
    pub const fn new(original_bytes: usize, decoded_utf8_bytes: usize) -> Self {
        Self {
            original_bytes,
            decoded_utf8_bytes,
        }
    }

    /// Returns the greatest number of input bytes.
    #[must_use]
    pub const fn max_original_bytes(self) -> usize {
        self.original_bytes
    }

    /// Returns the greatest number of decoded UTF-8 bytes.
    #[must_use]
    pub const fn max_decoded_utf8_bytes(self) -> usize {
        self.decoded_utf8_bytes
    }
}

impl Default for DecodeLimits {
    /// No limit on either size.
    fn default() -> Self {
        Self::new(usize::MAX, usize::MAX)
    }
}

/// A decode failed as a whole, without returning any text.
///
/// Ask what happened with the `is_*` predicates; the cause is private so that
/// a new one is not a breaking change.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DecodeErrorKind {
    OriginalByteLimit,
    DecodedUtf8ByteLimit,
    CapacityOverflow,
    AllocationFailure,
}

impl DecodeError {
    const fn new(kind: DecodeErrorKind) -> Self {
        Self { kind }
    }

    kind_predicates! { kind;
        /// Whether the input is longer than [`DecodeLimits::max_original_bytes`].
        is_original_byte_limit => DecodeErrorKind::OriginalByteLimit,
        /// Whether the decoded text would be longer than
        /// [`DecodeLimits::max_decoded_utf8_bytes`].
        is_decoded_utf8_byte_limit => DecodeErrorKind::DecodedUtf8ByteLimit,
        /// Whether a length or capacity cannot be represented by Rust's
        /// allocation interface.
        is_capacity_overflow => DecodeErrorKind::CapacityOverflow,
        /// Whether the allocator could not provide the decoded text's storage.
        is_allocation_failure => DecodeErrorKind::AllocationFailure,
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            DecodeErrorKind::OriginalByteLimit => "style sheet original-byte limit exceeded",
            DecodeErrorKind::DecodedUtf8ByteLimit => {
                "style sheet decoded UTF-8 byte limit exceeded"
            }
            DecodeErrorKind::CapacityOverflow => "style sheet input capacity overflow",
            DecodeErrorKind::AllocationFailure => "style sheet input allocation failed",
        })
    }
}

impl std::error::Error for DecodeError {}

/// The result of decoding a style sheet's bytes: the code points of the input
/// before section 3.3 filters them, and how they were found.
///
/// The text is *unfiltered*: a U+000D, a U+000C and a U+0000 in the bytes are
/// still in it, because section 3.3 is a separate step that the lexer applies
/// as it reads and [`filter_code_points`](super::filter_code_points) applies to
/// a whole text. A byte order mark is not in the text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decoded {
    text: String,
    encoding: Encoding,
    selection: EncodingSelection,
    had_replacements: bool,
}

impl Decoded {
    /// Returns the decoded text, not yet filtered by section 3.3.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the decoded text, not yet filtered by section 3.3.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }

    /// Returns the encoding that decoded the bytes, which is the one a byte
    /// order mark selected when there was one.
    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Returns which source selected [`encoding`](Self::encoding).
    #[must_use]
    pub const fn selection(&self) -> EncodingSelection {
        self.selection
    }

    /// Returns whether a malformed byte sequence was replaced by U+FFFD, which
    /// the Encoding Standard's decoders do instead of failing. This is a fact
    /// about the decoder, not a CSS parse error.
    #[must_use]
    pub const fn had_replacements(&self) -> bool {
        self.had_replacements
    }
}

/// Decodes a style sheet's bytes into code points (section 3.2).
///
/// "Determine the fallback encoding" runs first: a valid protocol label, then
/// an `@charset "label";` declaration at the start of the first 1024 bytes
/// (where a `utf-16be` or `utf-16le` label means UTF-8, because the
/// declaration itself is ASCII), then the environment encoding, then UTF-8.
/// The Encoding Standard's "decode" then runs with that fallback: a UTF-8 or
/// UTF-16 byte order mark overrides it and is not part of the text.
///
/// The text is not filtered by section 3.3 (see [`Decoded`]), and an
/// `@charset` declaration that selected the encoding stays in the text, as
/// the specification says no `@charset` rule exists.
///
/// ```
/// use surgeist_css_parser::input::{decode, DecodeHints, DecodeLimits, EncodingSelection};
///
/// let decoded = decode(
///     b"@charset \"windows-1251\"; a { content: \"\xcf\" }",
///     DecodeHints::default(),
///     DecodeLimits::default(),
/// )?;
/// assert_eq!(decoded.encoding().name(), "windows-1251");
/// assert_eq!(decoded.selection(), EncodingSelection::CharsetPrefix);
/// assert!(decoded.text().ends_with("content: \"\u{41f}\" }"));
/// # Ok::<(), surgeist_css_parser::input::DecodeError>(())
/// ```
///
/// # Errors
///
/// Returns [`DecodeError`] when the input or the decoded text exceeds its
/// limit, a size cannot be represented or storage cannot be allocated.
pub fn decode(
    bytes: &[u8],
    hints: DecodeHints<'_>,
    limits: DecodeLimits,
) -> Result<Decoded, DecodeError> {
    if bytes.len() > limits.original_bytes {
        return Err(DecodeError::new(DecodeErrorKind::OriginalByteLimit));
    }
    let (fallback, fallback_selection) = fallback_encoding(bytes, hints);
    let (text, encoding, had_replacements) =
        decode_bounded(bytes, fallback, limits.decoded_utf8_bytes, CHUNK_BYTES)?;
    let selection = if starts_with_byte_order_mark(bytes) {
        EncodingSelection::ByteOrderMark
    } else {
        fallback_selection
    };
    Ok(Decoded {
        text,
        encoding,
        selection,
        had_replacements,
    })
}

/// The size of the buffer a decode fills at a time.
const CHUNK_BYTES: usize = 4096;

/// The three byte order marks that the Encoding Standard's "decode" recognizes:
/// UTF-8, UTF-16 big-endian and UTF-16 little-endian.
fn starts_with_byte_order_mark(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xEF, 0xBB, 0xBF])
        || bytes.starts_with(&[0xFF, 0xFE])
        || bytes.starts_with(&[0xFE, 0xFF])
}

/// "Determine the fallback encoding" of section 3.2.
fn fallback_encoding(bytes: &[u8], hints: DecodeHints<'_>) -> (Encoding, EncodingSelection) {
    if let Some(encoding) = hints.protocol_label.and_then(Encoding::for_label) {
        return (encoding, EncodingSelection::Protocol);
    }
    if let Some(encoding) = declared_encoding(bytes) {
        return (encoding, EncodingSelection::CharsetPrefix);
    }
    if let Some(encoding) = hints.environment_encoding {
        return (encoding, EncodingSelection::Environment);
    }
    (Encoding(UTF_8), EncodingSelection::DefaultUtf8)
}

/// Step 2 of "determine the fallback encoding": the encoding declaration
/// `@charset "` `XX*` `";` at the start of the first 1024 bytes, where each
/// `XX` is a byte from 0x00 to 0x21 or from 0x23 to 0x7F (any ASCII byte but
/// the quotation mark).
fn declared_encoding(bytes: &[u8]) -> Option<Encoding> {
    let prefix = bytes
        .get(..bytes.len().min(1024))?
        .strip_prefix(b"@charset \"")?;
    let quote = prefix.iter().position(|byte| *byte == b'"')?;
    if prefix.get(quote + 1) != Some(&b';') || !prefix[..quote].is_ascii() {
        return None;
    }
    let encoding = encoding_rs::Encoding::for_label(&prefix[..quote])?;
    Some(Encoding(if encoding == UTF_16LE || encoding == UTF_16BE {
        UTF_8
    } else {
        encoding
    }))
}

fn check_capacity(length: usize) -> Result<(), DecodeError> {
    if length > isize::MAX as usize {
        Err(DecodeError::new(DecodeErrorKind::CapacityOverflow))
    } else {
        Ok(())
    }
}

/// Runs the decoder over `bytes` in fresh buffers of `chunk_size` bytes (at
/// most [`CHUNK_BYTES`] and at least four), admitting the complete output of
/// each buffer against `max_bytes` before storing it. A fixed fresh buffer is
/// independent of the remaining output budget, so a small budget cannot stall
/// the decoder. The chunk size is a parameter so that tests can exercise a
/// four-byte sequence continued across buffers.
fn decode_bounded(
    bytes: &[u8],
    encoding: Encoding,
    max_bytes: usize,
    chunk_size: usize,
) -> Result<(String, Encoding, bool), DecodeError> {
    let mut decoder = encoding.0.new_decoder();
    let mut buffer = [0_u8; CHUNK_BYTES];
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
            .ok_or(DecodeError::new(DecodeErrorKind::CapacityOverflow))?;
        if length > max_bytes {
            return Err(DecodeError::new(DecodeErrorKind::DecodedUtf8ByteLimit));
        }
        check_capacity(length)?;
        output
            .try_reserve_exact(written)
            .map_err(|_| DecodeError::new(DecodeErrorKind::AllocationFailure))?;
        // The codec guarantees complete, valid UTF-8 in every written slice.
        output.push_str(
            std::str::from_utf8(&buffer[..written]).expect("encoding_rs guarantees UTF-8"),
        );
        if result == CoderResult::InputEmpty {
            return Ok((output, Encoding(decoder.encoding()), had_replacements));
        }
        if read == 0 && written == 0 {
            // A few legacy mappings produce several scalars atomically: grow
            // the buffer, keeping the same decoder.
            chunk_size = chunk_size
                .checked_mul(2)
                .filter(|size| *size <= buffer.len())
                .ok_or(DecodeError::new(DecodeErrorKind::CapacityOverflow))?;
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
            decode_bounded(bytes, Encoding(UTF_8), 10, 4),
            Ok(("AAA€Z�".to_owned(), Encoding(UTF_8), true))
        );
        assert_eq!(
            decode_bounded(bytes, Encoding(UTF_8), 9, 4),
            Err(DecodeError::new(DecodeErrorKind::DecodedUtf8ByteLimit))
        );
        assert_eq!(
            decode_bounded(b"AAA\xE2\x82\xACZ", Encoding(UTF_8), 7, 4),
            Ok(("AAA€Z".to_owned(), Encoding(UTF_8), false))
        );
    }

    #[test]
    fn four_byte_chunks_preserve_roman_state_and_final_escape_replacement() {
        let encoding = Encoding::for_label("iso-2022-jp").unwrap();
        let bytes = b"AAA\x1B(J\\\x1B(BZ\x1B";
        assert_eq!(
            decode_bounded(bytes, encoding, 9, 4),
            Ok(("AAA¥Z�".to_owned(), encoding, true))
        );
        assert_eq!(
            decode_bounded(bytes, encoding, 8, 4),
            Err(DecodeError::new(DecodeErrorKind::DecodedUtf8ByteLimit))
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
                Encoding::for_label("windows-1252").unwrap(),
                9,
                4
            ),
            Ok(("�AAA�".to_owned(), Encoding(UTF_8), true))
        );
    }

    #[test]
    fn capacity_overflow_is_typed_without_allocating() {
        assert_eq!(
            check_capacity(usize::MAX),
            Err(DecodeError::new(DecodeErrorKind::CapacityOverflow))
        );
    }
}
