//! Immutable Unicode sources and validated coordinates, independent of URL context.

use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};
use surgeist_html::values::DocumentString;

/// Opaque identity of one supplied source, independent of equal text or URLs.
#[derive(Clone, Debug)]
pub struct HtmlSourceId(Arc<()>);

impl PartialEq for HtmlSourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlSourceId {}
impl Hash for HtmlSourceId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

/// Coordinates in the caller's already-decoded input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlSourceCoordinates {
    /// Byte offsets in supplied valid UTF-8, at scalar boundaries.
    Utf8Bytes,
    /// Offsets between exact supplied UTF-16 code units.
    Utf16CodeUnits,
}

#[derive(Clone, Debug)]
enum SourceText {
    Utf8(Arc<str>),
    Utf16(DocumentString),
}

/// An immutable, independently identified already-decoded Unicode source.
///
/// A literal U+FEFF remains supplied content. This is not a byte decoder, encoding
/// sniffer, document URL, base URL, security origin or permission policy.
#[derive(Clone, Debug)]
pub struct HtmlSource {
    identity: HtmlSourceId,
    text: SourceText,
    lines: Arc<LineIndex>,
}

/// The offsets at which the lines of a source start, built when a position is
/// first read and shared by the clones of the source.
#[derive(Default)]
struct LineIndex(OnceLock<Vec<usize>>);

impl fmt::Debug for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LineIndex")
    }
}

/// A line and column of a source, both counted from one.
///
/// The column counts UTF-16 code units of the decoded source from the start of
/// the line, whichever coordinates the source uses. A line ends at LF, CRLF or a
/// lone CR, as the input stream preprocessing treats them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HtmlSourcePosition {
    line: usize,
    column: usize,
}

impl HtmlSourcePosition {
    /// The line, counted from one.
    pub fn line(&self) -> usize {
        self.line
    }
    /// The column in UTF-16 code units, counted from one.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl HtmlSource {
    /// Supplies valid UTF-8 with UTF-8 byte coordinates and a fresh identity.
    pub fn utf8(text: impl Into<Arc<str>>) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf8(text.into()),
            lines: Arc::default(),
        }
    }

    /// Supplies exact UTF-16 units with unit coordinates and a fresh identity.
    pub fn utf16(text: DocumentString) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf16(text),
            lines: Arc::default(),
        }
    }

    /// Returns the source identity; cloning the source retains this identity.
    pub fn identity(&self) -> &HtmlSourceId {
        &self.identity
    }

    /// Returns the supplied coordinate space.
    pub fn coordinates(&self) -> HtmlSourceCoordinates {
        match self.text {
            SourceText::Utf8(_) => HtmlSourceCoordinates::Utf8Bytes,
            SourceText::Utf16(_) => HtmlSourceCoordinates::Utf16CodeUnits,
        }
    }

    /// Returns the supplied length in the explicit coordinate space.
    pub fn len(&self) -> usize {
        match &self.text {
            SourceText::Utf8(text) => text.len(),
            SourceText::Utf16(text) => text.len(),
        }
    }

    /// Reports whether the supplied source contains no units.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Borrows the supplied UTF-8 when that is the source's input representation.
    pub fn as_utf8(&self) -> Option<&str> {
        match &self.text {
            SourceText::Utf8(text) => Some(text),
            SourceText::Utf16(_) => None,
        }
    }

    /// Borrows the exact UTF-16 when that is the source's input representation.
    pub fn as_utf16(&self) -> Option<&DocumentString> {
        match &self.text {
            SourceText::Utf8(_) => None,
            SourceText::Utf16(text) => Some(text),
        }
    }

    /// Validates a half-open source range in the supplied coordinate space.
    ///
    /// UTF-16 ranges may separate the units of a surrogate pair: they describe
    /// supplied units, not an assertion that the selected units are scalars.
    pub fn range(&self, start: usize, end: usize) -> Result<HtmlSourceRange, HtmlSourceRangeError> {
        if start > end {
            return Err(HtmlSourceRangeError::Reversed { start, end });
        }
        if end > self.len() {
            return Err(HtmlSourceRangeError::OutOfBounds {
                end,
                length: self.len(),
            });
        }
        if let SourceText::Utf8(text) = &self.text {
            for offset in [start, end] {
                if !text.is_char_boundary(offset) {
                    return Err(HtmlSourceRangeError::Utf8Boundary { offset });
                }
            }
        }
        Ok(HtmlSourceRange {
            source: self.identity.clone(),
            coordinates: self.coordinates(),
            start,
            end,
        })
    }

    /// The line and column of `offset`, in this source's coordinates.
    ///
    /// The line index is built when the first position is read, and the parse
    /// itself counts no line. The index holds 8 bytes for each line of the source
    /// and is not charged to any parse limit. An offset that is past the end or splits a UTF-8
    /// scalar has no position.
    pub fn position(&self, offset: usize) -> Result<HtmlSourcePosition, HtmlSourceRangeError> {
        // The same validation as an empty range at `offset`.
        self.range(offset, offset)?;
        let starts = self.lines.0.get_or_init(|| self.line_starts());
        let line = starts.partition_point(|start| *start <= offset);
        // The first start is zero, so at least one start is at or before it.
        let start = line.checked_sub(1).and_then(|index| starts.get(index));
        let (Some(line_start), true) = (start, line != 0) else {
            return Err(HtmlSourceRangeError::OutOfBounds {
                end: offset,
                length: self.len(),
            });
        };
        let column = match &self.text {
            SourceText::Utf8(text) => text
                .get(*line_start..offset)
                .map(|line| line.chars().map(char::len_utf16).sum::<usize>()),
            SourceText::Utf16(_) => offset.checked_sub(*line_start),
        };
        let Some(column) = column else {
            return Err(HtmlSourceRangeError::Utf8Boundary { offset });
        };
        Ok(HtmlSourcePosition {
            line,
            column: column.saturating_add(1),
        })
    }

    /// The offsets at which each line starts: zero, and the offset after each
    /// LF, CRLF or lone CR.
    fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0];
        match &self.text {
            SourceText::Utf8(text) => {
                let mut previous = None;
                for (index, byte) in text.bytes().enumerate() {
                    match (byte, previous) {
                        (0x0A, Some(0x0D)) => {
                            // CRLF: the CR already started the next line at the
                            // LF's offset, so move it past the LF.
                            if let Some(last) = starts.last_mut() {
                                *last = index.saturating_add(1);
                            }
                        }
                        (0x0A | 0x0D, _) => starts.push(index.saturating_add(1)),
                        _ => {}
                    }
                    previous = Some(byte);
                }
            }
            SourceText::Utf16(text) => {
                let mut previous = None;
                for (index, &unit) in text.code_units().iter().enumerate() {
                    match (unit, previous) {
                        (0x0A, Some(0x0D)) => {
                            if let Some(last) = starts.last_mut() {
                                *last = index.saturating_add(1);
                            }
                        }
                        (0x0A | 0x0D, _) => starts.push(index.saturating_add(1)),
                        _ => {}
                    }
                    previous = Some(unit);
                }
            }
        }
        starts
    }

    /// Obtains exact text for a validated range of this source without normalization.
    pub fn text(&self, range: &HtmlSourceRange) -> Result<DocumentString, HtmlSourceRangeError> {
        if range.source != self.identity {
            return Err(HtmlSourceRangeError::ForeignSource);
        }
        // The range's fields are private and every constructor validates them
        // against its own source, but the offsets are checked again here so
        // that no slice can panic.
        let checked = self.range(range.start, range.end)?;
        let out_of_bounds = HtmlSourceRangeError::OutOfBounds {
            end: checked.end,
            length: self.len(),
        };
        match &self.text {
            SourceText::Utf8(text) => text
                .get(checked.start..checked.end)
                .map(DocumentString::from)
                .ok_or(out_of_bounds),
            SourceText::Utf16(text) => text
                .code_units()
                .get(checked.start..checked.end)
                .map(DocumentString::from_utf16)
                .ok_or(out_of_bounds),
        }
    }
}

/// A validated half-open range, qualified by source identity and coordinate space.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlSourceRange {
    source: HtmlSourceId,
    coordinates: HtmlSourceCoordinates,
    start: usize,
    end: usize,
}

impl HtmlSourceRange {
    /// Returns the owning source identity.
    pub fn source(&self) -> &HtmlSourceId {
        &self.source
    }
    /// Returns the input coordinate space.
    pub fn coordinates(&self) -> HtmlSourceCoordinates {
        self.coordinates
    }
    /// Returns the inclusive starting offset.
    pub fn start(&self) -> usize {
        self.start
    }
    /// Returns the exclusive ending offset.
    pub fn end(&self) -> usize {
        self.end
    }
    /// Returns the number of input units in this range.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
    /// Reports whether this is an empty range or end anchor.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A rejectable range or source-identity mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSourceRangeError {
    /// The supplied start followed its end.
    Reversed { start: usize, end: usize },
    /// The range ended beyond the source.
    OutOfBounds { end: usize, length: usize },
    /// An offset splits a UTF-8 scalar encoding.
    Utf8Boundary { offset: usize },
    /// The range belongs to a different source, even if its text is equal.
    ForeignSource,
}

impl fmt::Display for HtmlSourceRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reversed { start, end } => {
                write!(f, "source range start {start} follows end {end}")
            }
            Self::OutOfBounds { end, length } => {
                write!(f, "source range end {end} exceeds length {length}")
            }
            Self::Utf8Boundary { offset } => {
                write!(f, "source offset {offset} splits a UTF-8 scalar")
            }
            Self::ForeignSource => f.write_str("range belongs to another source"),
        }
    }
}
impl Error for HtmlSourceRangeError {}

#[cfg(test)]
mod text_tests {
    use super::*;
    use surgeist_html::values::DocumentString;

    /// A range that only this module can build: it names the source's own
    /// identity but was never validated against its text.
    fn forged(source: &HtmlSource, start: usize, end: usize) -> HtmlSourceRange {
        HtmlSourceRange {
            source: source.identity().clone(),
            coordinates: source.coordinates(),
            start,
            end,
        }
    }

    #[test]
    fn text_of_a_range_beyond_the_source_reports_out_of_bounds() {
        let utf8 = HtmlSource::utf8("ab");
        assert_eq!(
            utf8.text(&forged(&utf8, 0, 5)),
            Err(HtmlSourceRangeError::OutOfBounds { end: 5, length: 2 })
        );
        let utf16 = HtmlSource::utf16(DocumentString::from("ab"));
        assert_eq!(
            utf16.text(&forged(&utf16, 1, 3)),
            Err(HtmlSourceRangeError::OutOfBounds { end: 3, length: 2 })
        );
    }

    #[test]
    fn text_of_a_range_splitting_a_utf8_scalar_reports_the_boundary() {
        let source = HtmlSource::utf8("é");
        assert_eq!(
            source.text(&forged(&source, 1, 2)),
            Err(HtmlSourceRangeError::Utf8Boundary { offset: 1 })
        );
    }

    #[test]
    fn text_of_a_reversed_range_reports_it() {
        let source = HtmlSource::utf8("abc");
        assert_eq!(
            source.text(&forged(&source, 2, 1)),
            Err(HtmlSourceRangeError::Reversed { start: 2, end: 1 })
        );
    }

    #[test]
    fn text_of_a_validated_range_is_the_exact_slice_in_both_coordinates() {
        let utf8 = HtmlSource::utf8("a🙂b");
        assert_eq!(
            utf8.text(&utf8.range(1, 5).unwrap()).unwrap(),
            DocumentString::from("🙂")
        );
        let units = DocumentString::from_utf16(vec![0x61, 0xD800, 0x62]);
        let utf16 = HtmlSource::utf16(units);
        assert_eq!(
            utf16.text(&utf16.range(1, 3).unwrap()).unwrap(),
            DocumentString::from_utf16(vec![0xD800, 0x62])
        );
    }
}
