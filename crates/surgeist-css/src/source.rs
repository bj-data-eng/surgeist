use std::fmt;
use std::ops::Range;
use std::sync::Arc;

/// A UTF8 byte offset into the parser input. Ordinary UTF8 ingress preserves the
/// original bytes; UTF16 ingress addresses the normalized UTF8 parser buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssByteOffset(usize);

impl CssByteOffset {
    const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based byte offset.
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}

/// A zero-based line index in the CSS parser input.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssLineIndex(u32);

impl CssLineIndex {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the zero-based line index.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A zero-based parser-input column measured in UTF16 code units.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssUtf16ColumnIndex(u32);

impl CssUtf16ColumnIndex {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the zero-based UTF-16 column index.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A position in the CSS parser input.
///
/// Ordinary UTF8 ingress addresses original input. UTF16 ingress addresses the
/// normalized UTF8 buffer, with normalized line/UTF16-column coordinates.
/// Original UTF16 unit intervals are available separately through
/// [`CssSourceSnapshot::original_utf16_range`]. Lines and columns are zero based.
/// Positions are parser-produced metadata and cannot be publicly forged.
///
/// ```compile_fail
/// use surgeist_css::{CssByteOffset, CssLineIndex, CssSourcePosition, CssUtf16ColumnIndex};
///
/// let _ = CssSourcePosition {
///     byte_offset: CssByteOffset(0),
///     line: CssLineIndex(0),
///     column: CssUtf16ColumnIndex(0),
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssSourcePosition {
    byte_offset: CssByteOffset,
    line: CssLineIndex,
    column: CssUtf16ColumnIndex,
}

impl CssSourcePosition {
    /// Advances a parser-owned checkpoint over original source text. Checkpoints
    /// never split CRLF; the final requested position may stop between its bytes.
    pub(crate) fn advanced_by(self, source: &str) -> Self {
        let mut line = self.line.value();
        let mut column = self.column.value();
        let mut characters = source.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek() == Some(&'\n') {
                        characters.next();
                    }
                    line = line.saturating_add(1);
                    column = 0;
                }
                '\n' | '\u{000c}' => {
                    line = line.saturating_add(1);
                    column = 0;
                }
                _ => column = column.saturating_add(character.len_utf16() as u32),
            }
        }
        Self {
            byte_offset: CssByteOffset::new(
                self.byte_offset
                    .value()
                    .checked_add(source.len())
                    .expect("source checkpoint plus its suffix fits the original input"),
            ),
            line: CssLineIndex::new(line),
            column: CssUtf16ColumnIndex::new(column),
        }
    }

    pub(crate) fn from_cssparser(
        position: cssparser::SourcePosition,
        location: cssparser::SourceLocation,
    ) -> Self {
        Self {
            byte_offset: CssByteOffset::new(position.byte_index()),
            line: CssLineIndex::new(location.line),
            column: CssUtf16ColumnIndex::new(location.column.saturating_sub(1)),
        }
    }

    pub(crate) const fn previous_ascii_byte(self) -> Self {
        Self {
            byte_offset: CssByteOffset::new(self.byte_offset.value().saturating_sub(1)),
            line: self.line,
            column: CssUtf16ColumnIndex::new(self.column.value().saturating_sub(1)),
        }
    }

    pub(crate) const fn from_source_location(location: cssparser::SourceLocation) -> Self {
        Self {
            byte_offset: CssByteOffset::new(0),
            line: CssLineIndex::new(location.line),
            column: CssUtf16ColumnIndex::new(location.column.saturating_sub(1)),
        }
    }

    pub(crate) fn from_source_location_in(
        source: &str,
        location: cssparser::SourceLocation,
    ) -> Self {
        let target_line = location.line;
        let target_column = location.column.saturating_sub(1);
        let mut line = 0_u32;
        let mut column = 0_u32;
        let mut byte_offset = 0_usize;
        let mut characters = source.char_indices().peekable();

        while let Some((index, character)) = characters.next() {
            if line == target_line && column >= target_column {
                byte_offset = index;
                break;
            }

            byte_offset = index + character.len_utf8();
            match character {
                '\r' => {
                    if characters
                        .peek()
                        .is_some_and(|(_, next_character)| *next_character == '\n')
                        && let Some((next_index, next_character)) = characters.next()
                    {
                        byte_offset = next_index + next_character.len_utf8();
                    }
                    line = line.saturating_add(1);
                    column = 0;
                }
                '\n' | '\u{000c}' => {
                    line = line.saturating_add(1);
                    column = 0;
                }
                _ => column = column.saturating_add(character.len_utf16() as u32),
            }
        }

        Self {
            byte_offset: CssByteOffset::new(byte_offset),
            line: CssLineIndex::new(location.line),
            column: CssUtf16ColumnIndex::new(target_column),
        }
    }

    pub(crate) fn from_byte_offset_in(source: &str, target: usize) -> Self {
        let target = target.min(source.len());
        let mut line = 0_u32;
        let mut column = 0_u32;
        let mut characters = source[..target].chars().peekable();

        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek().is_some_and(|next| *next == '\n') {
                        characters.next();
                    }
                    line = line.saturating_add(1);
                    column = 0;
                }
                '\n' | '\u{000c}' => {
                    line = line.saturating_add(1);
                    column = 0;
                }
                _ => column = column.saturating_add(character.len_utf16() as u32),
            }
        }

        Self {
            byte_offset: CssByteOffset::new(target),
            line: CssLineIndex::new(line),
            column: CssUtf16ColumnIndex::new(column),
        }
    }

    /// Returns the zero-based UTF-8 byte offset.
    #[must_use]
    pub const fn byte_offset(self) -> CssByteOffset {
        self.byte_offset
    }

    /// Returns the zero-based line index.
    #[must_use]
    pub const fn line(self) -> CssLineIndex {
        self.line
    }

    /// Returns the zero-based UTF-16 column index.
    #[must_use]
    pub const fn column(self) -> CssUtf16ColumnIndex {
        self.column
    }
}

impl fmt::Display for CssSourcePosition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}",
            self.line.value().saturating_add(1),
            self.column.value().saturating_add(1)
        )
    }
}

/// An inclusive-start, exclusive-end span in the CSS parser input.
///
/// Spans are parser-produced metadata. Crate-owned construction rejects an end
/// that precedes the start byte offset; zero-width spans remain available for
/// missing-token and implicit-EOF diagnostics.
///
/// ```compile_fail
/// use surgeist_css::{CssSourcePosition, CssSourceSpan};
///
/// fn forge(start: CssSourcePosition, end: CssSourcePosition) -> CssSourceSpan {
///     CssSourceSpan { start, end }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssSourceSpan {
    start: CssSourcePosition,
    end: CssSourcePosition,
}

impl CssSourceSpan {
    #[must_use]
    pub(crate) const fn new(start: CssSourcePosition, end: CssSourcePosition) -> Option<Self> {
        if start.byte_offset.value() <= end.byte_offset.value() {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Returns the inclusive start position.
    #[must_use]
    pub const fn start(self) -> CssSourcePosition {
        self.start
    }

    /// Returns the exclusive end position.
    #[must_use]
    pub const fn end(self) -> CssSourcePosition {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use cssparser::{Parser, ParserInput};

    use super::*;

    fn position_after_tokenizing(source: &str) -> CssSourcePosition {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        while parser.next().is_ok() {}
        CssSourcePosition::from_cssparser(parser.position(), parser.current_source_location())
    }

    fn assert_position(position: CssSourcePosition, byte_offset: usize, line: u32, column: u32) {
        assert_eq!(position.byte_offset().value(), byte_offset);
        assert_eq!(position.line().value(), line);
        assert_eq!(position.column().value(), column);
    }

    #[test]
    fn source_empty_input_starts_at_zero() {
        assert_position(position_after_tokenizing(""), 0, 0, 0);
    }

    #[test]
    fn source_first_and_later_columns_are_zero_based() {
        assert_position(position_after_tokenizing("a"), 1, 0, 1);
        assert_position(position_after_tokenizing("abc"), 3, 0, 3);
    }

    #[test]
    fn source_lf_and_crlf_advance_one_zero_based_line() {
        assert_position(position_after_tokenizing("a\nb"), 3, 1, 1);
        assert_position(position_after_tokenizing("a\r\nb"), 4, 1, 1);
    }

    #[test]
    fn source_multiline_comment_preserves_authored_coordinates() {
        assert_position(position_after_tokenizing("/*a\nbc*/"), 8, 1, 4);
    }

    #[test]
    fn source_escape_counts_authored_bytes_instead_of_decoded_spelling() {
        assert_position(position_after_tokenizing("\\61 bc"), 6, 0, 6);
    }

    #[test]
    fn source_supplementary_scalar_has_distinct_utf8_and_utf16_widths() {
        assert_position(position_after_tokenizing("😀2x"), 6, 0, 4);
    }

    #[test]
    fn source_span_rejects_reverse_order_and_allows_zero_width() {
        let start = position_after_tokenizing("a");
        let end = position_after_tokenizing("ab");

        let span = CssSourceSpan::new(start, end).expect("ordered span");
        assert_eq!(span.start(), start);
        assert_eq!(span.end(), end);
        let empty = CssSourceSpan::new(start, start).expect("zero-width span");
        assert_eq!(empty.start(), start);
        assert_eq!(empty.end(), start);
        assert_eq!(CssSourceSpan::new(end, start), None);
    }

    #[test]
    fn source_dependency_zero_column_saturates_to_zero() {
        let mut input = ParserInput::new("");
        let parser = Parser::new(&mut input);
        let position = CssSourcePosition::from_cssparser(
            parser.position(),
            cssparser::SourceLocation { line: 0, column: 0 },
        );

        assert_position(position, 0, 0, 0);
    }
}

/// One immutable parse input. Equality compares source text.
///
/// [`Self::same_snapshot`] separately compares parse-input identity. Neither
/// operation represents a mutable stylesheet revision or CSSOM object handle.
#[derive(Clone)]
pub struct CssSourceSnapshot(Arc<SourceSnapshotData>);

struct SourceSnapshotData {
    text: String,
    checkpoints: Vec<CssSourcePosition>,
    utf16: Option<crate::utf16_input::Utf16Source>,
    storage: CssSourceStorageUsage,
}

/// Checked retained backing of a source, excluding fixed Arc headers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssSourceStorageUsage(pub(crate) usize);

impl CssSourceStorageUsage {
    #[must_use]
    pub const fn retained_bytes(self) -> usize {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn validate(self, maximum: usize) -> bool {
        self.0 <= maximum
    }
}

impl CssSourceSnapshot {
    pub(crate) fn new(source: &str) -> Self {
        Self::try_from_owned(source.to_owned()).expect("source snapshot allocation")
    }

    pub(crate) fn try_from_owned(
        source: String,
    ) -> Result<Self, std::collections::TryReserveError> {
        let mut position = CssSourcePosition::from_byte_offset_in("", 0);
        let mut checkpoints = Vec::new();
        checkpoints.try_reserve(1)?;
        checkpoints.push(position);
        let mut start = 0;
        let mut characters = source.char_indices().peekable();
        while let Some((offset, character)) = characters.next() {
            let mut end = offset + character.len_utf8();
            if character == '\r' && characters.peek().is_some_and(|(_, next)| *next == '\n') {
                let (offset, character) = characters.next().expect("peeked LF");
                end = offset + character.len_utf8();
            }
            if end - start >= 64 {
                position = position.advanced_by(&source[start..end]);
                checkpoints.try_reserve(1)?;
                checkpoints.push(position);
                start = end;
            }
        }
        let storage = CssSourceStorageUsage(
            source
                .capacity()
                .checked_add(
                    checkpoints
                        .capacity()
                        .checked_mul(std::mem::size_of::<CssSourcePosition>())
                        .expect("allocated checkpoint backing fits usize"),
                )
                .expect("allocated source backing fits usize"),
        );
        Ok(Self(Arc::new(SourceSnapshotData {
            text: source,
            checkpoints,
            utf16: None,
            storage,
        })))
    }

    pub(crate) fn position_at(&self, byte_offset: usize) -> Option<CssSourcePosition> {
        if !self.as_str().is_char_boundary(byte_offset) {
            return None;
        }
        let index = self
            .0
            .checkpoints
            .partition_point(|position| position.byte_offset().value() <= byte_offset);
        let checkpoint = self.0.checkpoints[index - 1];
        self.as_str()
            .get(checkpoint.byte_offset().value()..byte_offset)
            .map(|suffix| checkpoint.advanced_by(suffix))
    }

    /// Returns the complete UTF8 parser input: original for UTF8 ingress,
    /// CSS-preprocessed normalized UTF8 for UTF16 ingress.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0.text
    }

    /// Reports whether two origins refer to the same immutable parse input.
    #[must_use]
    pub fn same_snapshot(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    #[must_use]
    pub fn storage_usage(&self) -> CssSourceStorageUsage {
        self.0.storage
    }

    /// Original shared UTF16 input, when this source used UTF16 ingress.
    #[must_use]
    pub fn original_utf16(&self) -> Option<&Arc<Vec<u16>>> {
        self.0.utf16.as_ref().map(|value| &value.original)
    }

    /// Maps complete normalized scalar boundaries to original half-open units.
    /// Interior UTF8 byte offsets and out-of-range intervals are rejected.
    #[must_use]
    pub fn original_utf16_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        let input = self.0.utf16.as_ref()?;
        if range.start > range.end {
            return None;
        }
        let boundary = |byte| {
            if byte == self.as_str().len() {
                return Some(input.original.len());
            }
            input
                .map
                .binary_search_by_key(&byte, |entry| entry.0)
                .ok()
                .map(|index| input.map[index].1)
        };
        Some(boundary(range.start)?..boundary(range.end)?)
    }

    pub(crate) fn attach_utf16(mut self, input: crate::utf16_input::Utf16Source) -> Option<Self> {
        let extra = input.original.capacity().checked_mul(2)?.checked_add(
            input
                .map
                .capacity()
                .checked_mul(std::mem::size_of::<(usize, usize)>())?,
        )?;
        let data = Arc::get_mut(&mut self.0)?;
        data.storage = data.storage.checked_add(CssSourceStorageUsage(extra))?;
        data.utf16 = Some(input);
        Some(self)
    }

    pub(crate) fn try_from_preprocessed<E>(
        source: String,
        limits: crate::CssUtf16InputLimits,
        other_bytes: usize,
        step: &mut impl FnMut() -> Result<(), crate::CssUtf16InputError<E>>,
    ) -> Result<Self, crate::CssUtf16InputError<E>> {
        use crate::CssUtf16InputError::{Allocation, CapacityOverflow};
        let backing = |capacity: usize| {
            other_bytes
                .checked_add(source.capacity())?
                .checked_add(capacity.checked_mul(std::mem::size_of::<CssSourcePosition>())?)
        };
        let mut position = CssSourcePosition::from_byte_offset_in("", 0);
        let mut checkpoints = Vec::new();
        crate::utf16_input::check(1, limits.max_checkpoints, "source checkpoints")?;
        crate::utf16_input::check(
            backing(1).ok_or(CapacityOverflow)?,
            limits.max_retained_bytes,
            "retained source bytes",
        )?;
        checkpoints.try_reserve_exact(1).map_err(|_| Allocation)?;
        crate::utf16_input::check(
            checkpoints.capacity(),
            limits.max_checkpoints,
            "source checkpoints",
        )?;
        crate::utf16_input::check(
            backing(checkpoints.capacity()).ok_or(CapacityOverflow)?,
            limits.max_retained_bytes,
            "retained source bytes",
        )?;
        checkpoints.push(position);
        let mut start = 0;
        for (offset, character) in source.char_indices() {
            step()?;
            let end = offset + character.len_utf8();
            if end - start >= 64 {
                position = position.advanced_by(&source[start..end]);
                crate::utf16_input::check(
                    checkpoints.len().checked_add(1).ok_or(CapacityOverflow)?,
                    limits.max_checkpoints,
                    "source checkpoints",
                )?;
                if checkpoints.len() == checkpoints.capacity() {
                    let capacity = checkpoints
                        .capacity()
                        .saturating_mul(2)
                        .max(1)
                        .min(limits.max_checkpoints);
                    crate::utf16_input::check(
                        backing(capacity).ok_or(CapacityOverflow)?,
                        limits.max_retained_bytes,
                        "retained source bytes",
                    )?;
                    checkpoints
                        .try_reserve_exact(capacity - checkpoints.len())
                        .map_err(|_| Allocation)?;
                    crate::utf16_input::check(
                        checkpoints.capacity(),
                        limits.max_checkpoints,
                        "source checkpoints",
                    )?;
                    crate::utf16_input::check(
                        backing(checkpoints.capacity()).ok_or(CapacityOverflow)?,
                        limits.max_retained_bytes,
                        "retained source bytes",
                    )?;
                }
                checkpoints.push(position);
                start = end;
            }
        }
        let bytes = source
            .capacity()
            .checked_add(
                checkpoints
                    .capacity()
                    .checked_mul(std::mem::size_of::<CssSourcePosition>())
                    .ok_or(CapacityOverflow)?,
            )
            .ok_or(CapacityOverflow)?;
        crate::utf16_input::check(bytes, limits.max_retained_bytes, "retained source bytes")?;
        Ok(Self(Arc::new(SourceSnapshotData {
            text: source,
            checkpoints,
            utf16: None,
            storage: CssSourceStorageUsage(bytes),
        })))
    }
}

impl fmt::Debug for CssSourceSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CssSourceSnapshot")
            .field(&self.as_str())
            .finish()
    }
}

impl PartialEq for CssSourceSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CssSourceSnapshot {}

/// A parser-produced span bound to the immutable source that it addresses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssParsedOrigin {
    source: CssSourceSnapshot,
    span: CssSourceSpan,
}

impl CssParsedOrigin {
    pub(crate) fn from_range(source: &CssSourceSnapshot, range: Range<usize>) -> Option<Self> {
        Some(Self {
            source: source.clone(),
            span: CssSourceSpan::new(
                source.position_at(range.start)?,
                source.position_at(range.end)?,
            )?,
        })
    }

    /// Returns the source snapshot that owns this span.
    #[must_use]
    pub const fn source(&self) -> &CssSourceSnapshot {
        &self.source
    }

    /// Returns parser-input coordinates. UTF16 ingress uses normalized UTF8;
    /// query the source's original-unit mapping separately.
    #[must_use]
    pub const fn span(&self) -> CssSourceSpan {
        self.span
    }
}

#[cfg(test)]
mod utf16_admission_tests {
    use super::*;
    use crate::{CssParserContext, CssUtf16InputError, CssUtf16InputLimits};

    #[test]
    fn actual_preparation_backing_and_work_accept_exact_and_refuse_one_short() {
        let original = Arc::new(vec![u16::from(b'a'); 64]);
        let mut calls = 0;
        let report = CssParserContext::default()
            .parse_sheet_utf16_with_limits(original.clone(), CssUtf16InputLimits::default(), || {
                calls += 1;
                Ok::<_, ()>(())
            })
            .unwrap();
        let source = report.syntax().parse_source().unwrap().source();
        let raw = source.0.utf16.as_ref().unwrap();
        let scratch = std::mem::size_of::<crate::tokenization::bounded::InputCursor>()
            + std::mem::size_of::<CssSourcePosition>();
        let exact = CssUtf16InputLimits {
            max_input_units: original.len(),
            max_output_bytes: source.0.text.capacity(),
            max_map_entries: raw.map.capacity(),
            max_checkpoints: source.0.checkpoints.capacity(),
            max_scratch_bytes: scratch,
            max_retained_bytes: source.storage_usage().retained_bytes(),
            // The initial interruption observation is not a preparation action.
            max_work: calls - 1,
        };
        assert_eq!(source.as_str().len(), exact.max_output_bytes);
        assert_eq!(raw.map.len(), exact.max_map_entries);
        assert_eq!(source.0.checkpoints.len(), exact.max_checkpoints);
        CssParserContext::default()
            .parse_sheet_utf16_with_limits(original.clone(), exact, || Ok::<_, ()>(()))
            .unwrap();
        for (limits, cause) in [
            (
                CssUtf16InputLimits {
                    max_input_units: exact.max_input_units - 1,
                    ..exact
                },
                "original UTF16 units",
            ),
            (
                CssUtf16InputLimits {
                    max_output_bytes: exact.max_output_bytes - 1,
                    ..exact
                },
                "normalized UTF8 bytes",
            ),
            (
                CssUtf16InputLimits {
                    max_map_entries: exact.max_map_entries - 1,
                    ..exact
                },
                "source map entries",
            ),
            (
                CssUtf16InputLimits {
                    max_checkpoints: exact.max_checkpoints - 1,
                    ..exact
                },
                "source checkpoints",
            ),
            (
                CssUtf16InputLimits {
                    max_scratch_bytes: exact.max_scratch_bytes - 1,
                    ..exact
                },
                "preparation scratch bytes",
            ),
            (
                CssUtf16InputLimits {
                    max_retained_bytes: exact.max_retained_bytes - 1,
                    ..exact
                },
                "retained source bytes",
            ),
            (
                CssUtf16InputLimits {
                    max_work: exact.max_work - 1,
                    ..exact
                },
                "preparation work",
            ),
        ] {
            assert!(
                matches!(CssParserContext::default().parse_sheet_utf16_with_limits(original.clone(), limits, || Ok::<_, ()>(())), Err(CssUtf16InputError::Limit(actual)) if actual == cause),
                "{cause}"
            );
        }
    }

    #[test]
    fn post_grammar_interrupt_returns_no_report_and_releases_temporary_source() {
        let original = Arc::new(".a{color:red}".encode_utf16().collect::<Vec<_>>());
        let mut total = 0;
        drop(
            CssParserContext::default()
                .parse_sheet_utf16_with_limits(
                    original.clone(),
                    CssUtf16InputLimits::default(),
                    || {
                        total += 1;
                        Ok::<_, usize>(())
                    },
                )
                .unwrap(),
        );
        let mut calls = 0;
        let result = CssParserContext::default().parse_sheet_utf16_with_limits(
            original.clone(),
            CssUtf16InputLimits::default(),
            || {
                calls += 1;
                if calls == total { Err(calls) } else { Ok(()) }
            },
        );
        assert!(matches!(result, Err(CssUtf16InputError::Interrupted(actual)) if actual == total));
        assert_eq!(calls, total);
        assert_eq!(Arc::strong_count(&original), 1);
    }
}
