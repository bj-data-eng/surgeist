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
/// use surgeist_css::source::{CssByteOffset, CssLineIndex, CssSourcePosition, CssUtf16ColumnIndex};
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
/// use surgeist_css::source::{CssSourcePosition, CssSourceSpan};
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
pub struct CssSourceSnapshot(pub(crate) Arc<SourceSnapshotData>);

pub(crate) struct SourceSnapshotData {
    pub(crate) text: String,
    pub(crate) checkpoints: Vec<CssSourcePosition>,
    pub(crate) utf16: Option<crate::source::utf16_input::Utf16Source>,
    storage: CssSourceStorageUsage,
}

struct SourceCheckpoints {
    values: Vec<CssSourcePosition>,
    position: CssSourcePosition,
    start: usize,
}
impl SourceCheckpoints {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            position: CssSourcePosition::from_byte_offset_in("", 0),
            start: 0,
        }
    }
    fn fill<E>(
        &mut self,
        source: &str,
        mut step: impl FnMut() -> Result<(), E>,
        mut reserve: impl FnMut(&mut Vec<CssSourcePosition>) -> Result<(), E>,
    ) -> Result<(), E> {
        reserve(&mut self.values)?;
        self.values.push(self.position);
        let mut current = self.position;
        let mut characters = source.char_indices().peekable();
        while let Some((offset, character)) = characters.next() {
            step()?;
            let mut end = offset + character.len_utf8();
            if character == '\r' && characters.peek().is_some_and(|(_, next)| *next == '\n') {
                step()?;
                let (offset, character) = characters.next().expect("peeked LF");
                end = offset + character.len_utf8();
            }
            // Advance over this one scalar (or indivisible CRLF), rather than
            // rescanning a whole checkpoint suffix after its metered walk.
            current = current.advanced_by(&source[offset..end]);
            if end - self.start >= 64 {
                self.position = current;
                reserve(&mut self.values)?;
                self.values.push(self.position);
                self.start = end;
            }
        }
        Ok(())
    }
}

struct SecondarySourceFormation {
    text: String,
    checkpoints: SourceCheckpoints,
    input: surgeist_css_parser::input::InputCursor,
    scalars: Vec<char>,
}

/// Checked retained backing of a source, excluding fixed Arc headers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssSourceStorageUsage(pub(crate) usize);

impl CssSourceStorageUsage {
    /// Returns the retained backing bytes.
    #[must_use]
    pub const fn retained_bytes(self) -> usize {
        self.0
    }

    /// Adds two usages, or returns `None` when the sum overflows `usize`.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Returns whether this usage is within `maximum` bytes.
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
        let mut checkpoints = SourceCheckpoints::new();
        checkpoints.fill(&source, || Ok(()), |values| values.try_reserve(1))?;
        let checkpoints = checkpoints.values;
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

    /// A distinct decoded CSS-language input used by a called grammar. The
    /// original CSS input cursor preprocesses it before tokenization. Positions
    /// are local to that preprocessed input; it never acquires the main UTF16
    /// source's mapping or replaces that source's authored origins.
    pub(crate) fn try_from_decoded_utf8(
        source: &str,
        control: &crate::values::color_adjust::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        use crate::values::color_adjust::utf16_value::Failure;
        use surgeist_css_parser::input::{InputCursor, InputKind, InputRequest};
        let baseline = control.live().bytes();
        let result = (|| {
            control.step()?;
            control.add(std::mem::size_of::<SecondarySourceFormation>())?;
            let mut formation = SecondarySourceFormation {
                text: String::new(),
                checkpoints: SourceCheckpoints::new(),
                input: InputCursor::new(InputKind::Utf8, source.len()),
                scalars: Vec::new(),
            };
            let mut output_bytes = 0_usize;
            loop {
                control.step()?;
                match formation.input.request() {
                    InputRequest::Read { index } => formation
                        .input
                        .read(u16::from(source.as_bytes()[index]))
                        .expect("actual decoded UTF8 input"),
                    InputRequest::HeldUnit => {
                        formation.input.process_held().expect("actual held input")
                    }
                    InputRequest::Emit(scalar) => {
                        let Some(bytes) = output_bytes.checked_add(scalar.value.len_utf8()) else {
                            return control.fail(Failure::CapacityOverflow);
                        };
                        control.reserve(&mut formation.scalars, 1)?;
                        formation.scalars.push(scalar.value);
                        output_bytes = bytes;
                        formation
                            .input
                            .emitted()
                            .expect("actual admitted scalar storage");
                    }
                    InputRequest::End => {
                        if formation.input.end().expect("actual UTF8 EOF") {
                            break;
                        }
                    }
                }
            }
            // Reserve only the actual reached normalized output extent. The
            // scalar bank remains live during copying; it is neither a second
            // decoder nor an unaccounted worst-case text reservation.
            control.add(output_bytes)?;
            if formation.text.try_reserve_exact(output_bytes).is_err() {
                return control.fail(Failure::Allocation);
            }
            if formation.text.capacity() > output_bytes {
                control.add(formation.text.capacity() - output_bytes)?;
            }
            for scalar in &formation.scalars {
                control.step()?;
                formation.text.push(*scalar);
            }
            let Some(scalar_bytes) = formation
                .scalars
                .capacity()
                .checked_mul(std::mem::size_of::<char>())
            else {
                return control.fail(Failure::CapacityOverflow);
            };
            drop(std::mem::take(&mut formation.scalars));
            control.released(scalar_bytes);
            formation.checkpoints.fill(
                &formation.text,
                || control.step(),
                |values| control.reserve(values, 1),
            )?;
            let Some(checkpoint_bytes) = formation
                .checkpoints
                .values
                .capacity()
                .checked_mul(std::mem::size_of::<CssSourcePosition>())
            else {
                return control.fail(Failure::CapacityOverflow);
            };
            let Some(bytes) = formation.text.capacity().checked_add(checkpoint_bytes) else {
                return control.fail(Failure::CapacityOverflow);
            };
            control.add(std::mem::size_of::<SourceSnapshotData>())?;
            let SecondarySourceFormation {
                text,
                checkpoints,
                input: _,
                scalars,
            } = formation;
            drop(scalars);
            let result = Self(Arc::new(SourceSnapshotData {
                text,
                checkpoints: checkpoints.values,
                utf16: None,
                storage: CssSourceStorageUsage(bytes),
            }));
            // Formation ownership has moved into the immutable source owner.
            control.released(std::mem::size_of::<SecondarySourceFormation>());
            Ok(result)
        })();
        if result.is_err() {
            // Every failed formation payload has dropped before cleanup; the
            // caller's main source and unrelated banks retain their baseline.
            control.released(
                control
                    .live()
                    .bytes()
                    .checked_sub(baseline)
                    .expect("secondary source keeps caller baseline"),
            );
        }
        result
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

    /// Returns the checked retained backing of this snapshot.
    #[must_use]
    pub fn storage_usage(&self) -> CssSourceStorageUsage {
        self.0.storage
    }

    pub(crate) fn payload_bytes(&self, include_original: bool) -> Option<usize> {
        let mut bytes = std::mem::size_of::<SourceSnapshotData>()
            .checked_add(self.0.text.capacity())?
            .checked_add(
                self.0
                    .checkpoints
                    .capacity()
                    .checked_mul(std::mem::size_of::<CssSourcePosition>())?,
            )?;
        if let Some(input) = &self.0.utf16 {
            bytes = bytes.checked_add(input.map.capacity_bytes())?;
            if include_original {
                bytes = bytes
                    .checked_add(std::mem::size_of::<Vec<u16>>())?
                    .checked_add(
                        input
                            .original
                            .capacity()
                            .checked_mul(std::mem::size_of::<u16>())?,
                    )?;
            }
        }
        Some(bytes)
    }

    /// Original shared UTF16 input, when this source used UTF16 ingress.
    #[must_use]
    pub fn original_utf16(&self) -> Option<&Arc<Vec<u16>>> {
        self.0.utf16.as_ref().map(|value| &value.original)
    }

    pub(crate) fn preprocessed_scalar_count(&self) -> Option<usize> {
        self.0.utf16.as_ref().map(|value| value.map.scalar_count())
    }

    /// Maps complete normalized scalar boundaries to original half-open units.
    /// Interior UTF8 byte offsets and out-of-range intervals are rejected.
    #[must_use]
    pub fn original_utf16_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        self.0.utf16.as_ref()?.map.original_utf16(range)
    }

    pub(crate) fn attach_utf16(
        mut self,
        input: crate::source::utf16_input::Utf16Source,
    ) -> Option<Self> {
        let extra = input
            .original
            .capacity()
            .checked_mul(2)?
            .checked_add(input.map.capacity_bytes())?;
        let data = Arc::get_mut(&mut self.0)?;
        data.storage = data.storage.checked_add(CssSourceStorageUsage(extra))?;
        data.utf16 = Some(input);
        Some(self)
    }

    pub(crate) fn try_from_preprocessed<E>(
        source: String,
        mut checkpoints: Vec<CssSourcePosition>,
        mut position: CssSourcePosition,
        limits: crate::source::CssUtf16InputLimits,
        other_bytes: usize,
        control: &mut crate::source::utf16_input::Preparation<'_, E>,
    ) -> Result<Self, crate::source::CssUtf16InputError<E>> {
        let backing = |capacity: usize| {
            other_bytes
                .checked_add(source.capacity())?
                .checked_add(capacity.checked_mul(std::mem::size_of::<CssSourcePosition>())?)
        };
        crate::source::utf16_input::check(
            1,
            limits.max_checkpoints,
            crate::source::CssUtf16Resource::Checkpoints,
        )?;
        crate::source::utf16_input::check(
            backing(1).ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
            limits.max_retained_bytes,
            crate::source::CssUtf16Resource::RetainedBytes,
        )?;
        control.grow_vec(&mut checkpoints, 1)?;
        crate::source::utf16_input::check(
            checkpoints.capacity(),
            limits.max_checkpoints,
            crate::source::CssUtf16Resource::Checkpoints,
        )?;
        crate::source::utf16_input::check(
            backing(checkpoints.capacity())
                .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
            limits.max_retained_bytes,
            crate::source::CssUtf16Resource::RetainedBytes,
        )?;
        checkpoints.push(position);
        let mut start = 0;
        for (offset, character) in source.char_indices() {
            control.step()?;
            let end = offset + character.len_utf8();
            if end - start >= 64 {
                position = position.advanced_by(&source[start..end]);
                crate::source::utf16_input::check(
                    checkpoints
                        .len()
                        .checked_add(1)
                        .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
                    limits.max_checkpoints,
                    crate::source::CssUtf16Resource::Checkpoints,
                )?;
                if checkpoints.len() == checkpoints.capacity() {
                    let capacity = checkpoints
                        .capacity()
                        .saturating_mul(2)
                        .max(1)
                        .min(limits.max_checkpoints);
                    crate::source::utf16_input::check(
                        backing(capacity)
                            .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
                        limits.max_retained_bytes,
                        crate::source::CssUtf16Resource::RetainedBytes,
                    )?;
                    control.grow_vec(&mut checkpoints, capacity)?;
                    crate::source::utf16_input::check(
                        checkpoints.capacity(),
                        limits.max_checkpoints,
                        crate::source::CssUtf16Resource::Checkpoints,
                    )?;
                    crate::source::utf16_input::check(
                        backing(checkpoints.capacity())
                            .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
                        limits.max_retained_bytes,
                        crate::source::CssUtf16Resource::RetainedBytes,
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
                    .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?,
            )
            .ok_or_else(crate::source::CssUtf16InputError::capacity_overflow)?;
        crate::source::utf16_input::check(
            bytes,
            limits.max_retained_bytes,
            crate::source::CssUtf16Resource::RetainedBytes,
        )?;
        // Admit the actual immutable source payload before forming its owner;
        // the old String/checkpoint headers remain owned until this move.
        control.add(std::mem::size_of::<SourceSnapshotData>())?;
        let result = Self(Arc::new(SourceSnapshotData {
            text: source,
            checkpoints,
            utf16: None,
            storage: CssSourceStorageUsage(bytes),
        }));
        control.released(
            std::mem::size_of::<String>() + std::mem::size_of::<Vec<CssSourcePosition>>(),
        );
        Ok(result)
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

// Test-only work counter: source bytes compared by snapshot equality.
#[cfg(test)]
thread_local! {
    pub(crate) static SNAPSHOT_TEXT_BYTES_COMPARED: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

// Snapshots are equal when their parser input text is. One shared immutable
// owner has one text, so identity answers without reading it.
impl PartialEq for CssSourceSnapshot {
    fn eq(&self, other: &Self) -> bool {
        if self.same_snapshot(other) {
            return true;
        }
        #[cfg(test)]
        SNAPSHOT_TEXT_BYTES_COMPARED.with(|compared| {
            compared.set(compared.get() + self.as_str().len().min(other.as_str().len()));
        });
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
    use crate::source::CssUtf16InputLimits;

    #[test]
    fn actual_preparation_backing_and_work_accept_exact_and_refuse_one_short() {
        let original = Arc::new(vec![u16::from(b'a'); 64]);
        let mut calls = 0;
        let report = crate::CssParser::default()
            .parse_sheet_utf16(original.clone(), CssUtf16InputLimits::default(), || {
                calls += 1;
                Ok::<_, ()>(())
            })
            .unwrap();
        let source = report.syntax().parse_source().unwrap().source();
        let raw = source.0.utf16.as_ref().unwrap();
        let scratch = std::mem::size_of::<surgeist_css_parser::input::InputCursor>()
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
        assert_eq!(raw.map.scalar_count(), exact.max_map_entries);
        assert_eq!(source.0.checkpoints.len(), exact.max_checkpoints);
        crate::CssParser::default()
            .parse_sheet_utf16(original.clone(), exact, || Ok::<_, ()>(()))
            .unwrap();
        for (limits, cause) in [
            (
                CssUtf16InputLimits {
                    max_input_units: exact.max_input_units - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::InputUnits,
            ),
            (
                CssUtf16InputLimits {
                    max_output_bytes: exact.max_output_bytes - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::OutputBytes,
            ),
            (
                CssUtf16InputLimits {
                    max_map_entries: exact.max_map_entries - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::MapEntries,
            ),
            (
                CssUtf16InputLimits {
                    max_checkpoints: exact.max_checkpoints - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::Checkpoints,
            ),
            (
                CssUtf16InputLimits {
                    max_scratch_bytes: exact.max_scratch_bytes - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::ScratchBytes,
            ),
            (
                CssUtf16InputLimits {
                    max_retained_bytes: exact.max_retained_bytes - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::RetainedBytes,
            ),
            (
                CssUtf16InputLimits {
                    max_work: exact.max_work - 1,
                    ..exact
                },
                crate::source::CssUtf16Resource::Work,
            ),
        ] {
            assert!(
                matches!(crate::CssParser::default().parse_sheet_utf16(original.clone(), limits, || Ok::<_, ()>(())), Err(error) if error.exhausted_limit() == Some(cause)),
                "{cause:?}"
            );
        }
    }

    #[test]
    fn post_grammar_interrupt_returns_no_report_and_releases_temporary_source() {
        let original = Arc::new(".a{color:red}".encode_utf16().collect::<Vec<_>>());
        let mut total = 0;
        drop(
            crate::CssParser::default()
                .parse_sheet_utf16(original.clone(), CssUtf16InputLimits::default(), || {
                    total += 1;
                    Ok::<_, usize>(())
                })
                .unwrap(),
        );
        let mut calls = 0;
        let result = crate::CssParser::default().parse_sheet_utf16(
            original.clone(),
            CssUtf16InputLimits::default(),
            || {
                calls += 1;
                if calls == total { Err(calls) } else { Ok(()) }
            },
        );
        assert!(matches!(result, Err(error) if error.interruption() == Some(&total)));
        assert_eq!(calls, total);
        assert_eq!(Arc::strong_count(&original), 1);
    }
}

#[cfg(test)]
mod secondary_admission_tests {
    use super::*;
    use crate::source::{CssUtf16PropertyValueLimits, CssUtf16ValueAdmission};
    use crate::values::color_adjust::utf16_value::{Control, Failure};
    use std::cell::{Cell, RefCell};
    use std::mem::size_of;

    fn limits() -> CssUtf16PropertyValueLimits {
        CssUtf16PropertyValueLimits {
            input: crate::source::CssUtf16InputLimits::default(),
            max_work: 1_000_000,
            max_live_bytes: 16_000_000,
            max_retained_bytes: 16_000_000,
        }
    }

    #[test]
    fn decoded_secondary_source_keeps_local_positions_and_original_utf16_origin_independent() {
        let raw = Arc::new(r#""\3c length\3e ""#.encode_utf16().collect::<Vec<_>>());
        let main = crate::source::utf16_input::prepare_utf16(
            raw.clone(),
            crate::source::CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
            |_, _| Ok(()),
            |_, _| {},
        )
        .unwrap();
        let original = CssParsedOrigin::from_range(&main, 0..main.as_str().len()).unwrap();
        let events = RefCell::new(Vec::new());
        let admit = |action| {
            events.borrow_mut().push(action);
            Ok(())
        };
        let release = |_| {};
        let control = Control::new(limits(), &admit, &release);
        let baseline = main.payload_bytes(true).unwrap();
        control.add(baseline).unwrap();
        let secondary = CssSourceSnapshot::try_from_decoded_utf8("<length>", &control).unwrap();
        assert!(!secondary.same_snapshot(&main));
        assert!(secondary.original_utf16().is_none());
        assert_eq!(secondary.original_utf16_range(0..8), None);
        assert_eq!(secondary.preprocessed_scalar_count(), None);
        assert_eq!(secondary.as_str(), "<length>");
        let local = CssParsedOrigin::from_range(&secondary, 0..8).unwrap();
        assert_eq!(local.span().start().byte_offset().value(), 0);
        assert_eq!(local.span().end().byte_offset().value(), 8);
        assert!(original.source().same_snapshot(&main));
        assert!(Arc::ptr_eq(
            original.source().original_utf16().unwrap(),
            &raw
        ));
        assert_eq!(original.span().end().byte_offset().value(), raw.len());
        assert_eq!(
            control.live().bytes(),
            baseline + secondary.payload_bytes(false).unwrap()
        );
        let payload = secondary.payload_bytes(false).unwrap();
        drop(local);
        drop(secondary);
        control.released(payload);
        assert_eq!(control.live().bytes(), baseline);
        drop(original);
        drop(main);
        control.released(baseline);
        assert_eq!(control.live().bytes(), 0);
        assert!(
            events
                .borrow()
                .iter()
                .any(|action| matches!(action, CssUtf16ValueAdmission::Work))
        );
    }

    #[test]
    fn secondary_checkpoints_address_preprocessed_crlf_and_unicode_positions() {
        let text = format!("{}\r\n{}", "a".repeat(63), "😀".repeat(18));
        let admit = |_| Ok(());
        let released = Cell::new(0);
        let release = |usage: crate::source::CssUtf16ValueStorage| released.set(usage.bytes());
        let control = Control::new(limits(), &admit, &release);
        let source = CssSourceSnapshot::try_from_decoded_utf8(&text, &control).unwrap();
        let positions: Vec<_> = source
            .0
            .checkpoints
            .iter()
            .map(|position| {
                (
                    position.byte_offset().value(),
                    position.line().value(),
                    position.column().value(),
                )
            })
            .collect();
        assert_eq!(positions, [(0, 0, 0), (64, 1, 0), (128, 1, 32)]);
        let end = source.position_at(136).unwrap();
        assert_eq!((end.line().value(), end.column().value()), (1, 36));
        let normalized = format!("{}\n{}", "a".repeat(63), "😀".repeat(18));
        assert_eq!(source.as_str(), normalized);
        assert_eq!(control.live().bytes(), source.payload_bytes(false).unwrap());
        let ordinary = CssSourceSnapshot::new(&normalized);
        assert_eq!(source.0.checkpoints, ordinary.0.checkpoints);
        let payload = source.payload_bytes(false).unwrap();
        drop(source);
        control.released(payload);
        assert_eq!(released.get(), 0);
    }

    #[test]
    fn secondary_scalar_growth_one_short_and_preprocessing_cancellation_release_only_their_payload()
    {
        let text = "x".repeat(130);
        // The first genuinely emitted scalar forms the original Control's
        // four-char bank while the new Vec/old IntoIter growth headers coexist.
        let first_scalar_peak = size_of::<SecondarySourceFormation>()
            + size_of::<Vec<char>>()
            + size_of::<std::vec::IntoIter<char>>()
            + 4 * size_of::<char>();
        let mut short = limits();
        short.max_live_bytes = first_scalar_peak - 1;
        let admit = |_| Ok(());
        let released = Cell::new(None);
        let release =
            |usage: crate::source::CssUtf16ValueStorage| released.set(Some(usage.bytes()));
        let control = Control::new(short, &admit, &release);
        assert!(CssSourceSnapshot::try_from_decoded_utf8(&text, &control).is_err());
        assert_eq!(
            control.failure(),
            Some(Failure::Limit(
                crate::source::CssUtf16Resource::PropertyLiveBytes
            ))
        );
        assert_eq!(control.live().bytes(), 0);
        assert_eq!(released.get(), Some(0));

        let main = CssSourceSnapshot::new("original authored input");
        let baseline = main.payload_bytes(false).unwrap();
        let work = Cell::new(0);
        let admit = |action| {
            if matches!(action, CssUtf16ValueAdmission::Work) {
                work.set(work.get() + 1);
                // Refuse an actual decoder/storage action while the original
                // caller's source remains independently owned.
                if work.get() == 190 {
                    return Err(());
                }
            }
            Ok(())
        };
        let released = Cell::new(None);
        let release =
            |usage: crate::source::CssUtf16ValueStorage| released.set(Some(usage.bytes()));
        let control = Control::new(limits(), &admit, &release);
        control.add(baseline).unwrap();
        assert!(CssSourceSnapshot::try_from_decoded_utf8(&text, &control).is_err());
        assert_eq!(control.failure(), Some(Failure::Admission));
        assert_eq!(work.get(), 190);
        assert_eq!(control.live().bytes(), baseline);
        assert_eq!(released.get(), Some(baseline));
        assert_eq!(main.as_str(), "original authored input");
        drop(main);
        control.released(baseline);
    }

    #[test]
    fn secondary_css_preprocessing_normalizes_cr_ff_crlf_and_nul_without_main_origin() {
        let admit = |_| Ok(());
        let release = |_| {};
        let control = Control::new(limits(), &admit, &release);
        let source =
            CssSourceSnapshot::try_from_decoded_utf8("\r\nA\rB\u{c}C\0😀\r", &control).unwrap();
        assert_eq!(source.as_str(), "\nA\nB\nC�😀\n");
        assert!(source.original_utf16().is_none());
        assert_eq!(source.original_utf16_range(0..14), None);
        let end = source.position_at(14).unwrap();
        assert_eq!(
            (
                end.byte_offset().value(),
                end.line().value(),
                end.column().value()
            ),
            (14, 4, 0)
        );
        let payload = source.payload_bytes(false).unwrap();
        assert_eq!(control.live().bytes(), payload);
        drop(source);
        control.released(payload);
        assert_eq!(control.live().bytes(), 0);
    }

    #[test]
    fn every_secondary_preprocessing_admission_refuses_without_retaining_partial_payload() {
        let input = "\r\n\0\u{c}😀\r";
        let actions = Cell::new(0);
        let admit = |_| {
            actions.set(actions.get() + 1);
            Ok(())
        };
        let release = |_| {};
        let control = Control::new(limits(), &admit, &release);
        let source = CssSourceSnapshot::try_from_decoded_utf8(input, &control).unwrap();
        assert_eq!(source.as_str(), "\n�\n😀\n");
        let payload = source.payload_bytes(false).unwrap();
        drop(source);
        control.released(payload);

        // Cuts enumerate actual reached source-owner admissions; expected
        // normalization and unchanged main ownership are independent oracles.
        for cut in 1..=actions.get() {
            let main = CssSourceSnapshot::new("original authored source");
            let baseline = main.payload_bytes(false).unwrap();
            let seen = Cell::new(0);
            let enabled = Cell::new(false);
            let admit = |_| {
                if enabled.get() {
                    seen.set(seen.get() + 1);
                    if seen.get() == cut {
                        return Err(());
                    }
                }
                Ok(())
            };
            let release = |_| {};
            let control = Control::new(limits(), &admit, &release);
            control.add(baseline).unwrap();
            enabled.set(true);
            assert!(
                CssSourceSnapshot::try_from_decoded_utf8(input, &control).is_err(),
                "cut {cut}"
            );
            assert_eq!(control.failure(), Some(Failure::Admission), "cut {cut}");
            assert_eq!(control.live().bytes(), baseline, "cut {cut}");
            assert_eq!(main.as_str(), "original authored source");
            drop(main);
            control.released(baseline);
        }
    }
}

#[cfg(test)]
pub(crate) mod payload_tests;

#[cfg(test)]
pub(crate) mod equality_tests;
