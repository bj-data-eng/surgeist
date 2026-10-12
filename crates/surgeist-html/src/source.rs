//! Immutable Unicode sources and validated coordinates, independent of URL context.

use crate::DocumentString;
use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

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

/// A boundary established by the private cursor's actual decoded source span.
/// Unlike an offset, this carries the source and coordinate-space proof.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConsumedSourceBoundary {
    source: HtmlSourceId,
    coordinates: HtmlSourceCoordinates,
    offset: usize,
}

#[cfg(test)]
impl ConsumedSourceBoundary {
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }
}

#[cfg(test)]
enum DecodedSourceUnits {
    Utf8 { units: [u8; 4], length: usize },
    Utf16 { units: [u16; 2], length: usize },
}

/// Fixed native encoding prepared without inspecting source payload. The
/// cursor admits the exact bounded payload reads before completing this proof.
#[cfg(test)]
pub(crate) struct ConsumedBoundaryAdvance {
    start: ConsumedSourceBoundary,
    end: usize,
    units: DecodedSourceUnits,
}

#[cfg(test)]
impl ConsumedBoundaryAdvance {
    pub(crate) fn source_reads(&self) -> usize {
        self.end - self.start.offset
    }

    pub(crate) fn comparison_bytes(&self) -> usize {
        match self.units {
            DecodedSourceUnits::Utf8 { length, .. } => length,
            DecodedSourceUnits::Utf16 { length, .. } => length * 2,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumedBoundaryError {
    ForeignSource,
    ForeignCoordinates,
    InvalidScalar,
    Unavailable,
    InvalidBoundary,
    ContentMismatch,
}

/// An immutable, independently identified already-decoded Unicode source.
///
/// A literal U+FEFF remains supplied content. This is not a byte decoder, encoding
/// sniffer, document URL, base URL, security origin or permission policy.
#[derive(Clone, Debug)]
pub struct HtmlSource {
    identity: HtmlSourceId,
    text: SourceText,
}

impl HtmlSource {
    /// Supplies valid UTF-8 with UTF-8 byte coordinates and a fresh identity.
    pub fn utf8(text: impl Into<Arc<str>>) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf8(text.into()),
        }
    }

    /// Supplies exact UTF-16 units with unit coordinates and a fresh identity.
    pub fn utf16(text: DocumentString) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf16(text),
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

    /// Obtains exact text for a validated range of this source without normalization.
    pub fn text(&self, range: &HtmlSourceRange) -> Result<DocumentString, HtmlSourceRangeError> {
        if range.source != self.identity {
            return Err(HtmlSourceRangeError::ForeignSource);
        }
        match &self.text {
            SourceText::Utf8(text) => Ok(DocumentString::from(&text[range.start..range.end])),
            SourceText::Utf16(text) => Ok(DocumentString::from_utf16(
                &text.code_units()[range.start..range.end],
            )),
        }
    }

    /// Zero is established without a payload read. There is no integer-offset
    /// constructor for consumed-boundary proofs.
    #[cfg(test)]
    pub(crate) fn initial_consumed_boundary(&self) -> ConsumedSourceBoundary {
        ConsumedSourceBoundary {
            source: self.identity.clone(),
            coordinates: self.coordinates(),
            offset: 0,
        }
    }

    /// Prepares the actual decoded raw scalar's canonical supplied units.
    /// Newline preprocessing must happen after this raw boundary operation.
    /// UTF-16 literal surrogates are retained as one supplied unit.
    #[cfg(test)]
    pub(crate) fn prepare_consumed_boundary(
        &self,
        start: &ConsumedSourceBoundary,
        decoded: u32,
        available_through: usize,
    ) -> Result<ConsumedBoundaryAdvance, ConsumedBoundaryError> {
        self.require_consumed_boundary(start)?;
        if available_through > self.len() || start.offset > available_through {
            return Err(ConsumedBoundaryError::Unavailable);
        }
        let units = match self.coordinates() {
            HtmlSourceCoordinates::Utf8Bytes => {
                let scalar = char::from_u32(decoded).ok_or(ConsumedBoundaryError::InvalidScalar)?;
                let mut units = [0; 4];
                let length = scalar.encode_utf8(&mut units).len();
                DecodedSourceUnits::Utf8 { units, length }
            }
            HtmlSourceCoordinates::Utf16CodeUnits => {
                let mut units = [0; 2];
                let length = if (0xD800..=0xDFFF).contains(&decoded) {
                    units[0] = decoded as u16;
                    1
                } else {
                    let scalar =
                        char::from_u32(decoded).ok_or(ConsumedBoundaryError::InvalidScalar)?;
                    scalar.encode_utf16(&mut units).len()
                };
                DecodedSourceUnits::Utf16 { units, length }
            }
        };
        let length = match units {
            DecodedSourceUnits::Utf8 { length, .. } | DecodedSourceUnits::Utf16 { length, .. } => {
                length
            }
        };
        let end = start
            .offset
            .checked_add(length)
            .filter(|end| *end <= available_through)
            .ok_or(ConsumedBoundaryError::Unavailable)?;
        Ok(ConsumedBoundaryAdvance {
            start: start.clone(),
            end,
            units,
        })
    }

    /// Completes a proof by comparing every prepared unit exactly once. The
    /// cursor must precharge source_reads()/comparison_bytes() before entry.
    /// No source byte/unit at the exclusive end is inspected, even on failure.
    #[cfg(test)]
    pub(crate) fn complete_consumed_boundary(
        &self,
        advance: ConsumedBoundaryAdvance,
    ) -> Result<ConsumedSourceBoundary, ConsumedBoundaryError> {
        self.require_consumed_boundary(&advance.start)?;
        if advance.end > self.len() {
            return Err(ConsumedBoundaryError::InvalidBoundary);
        }
        let mut equal = true;
        match (&self.text, advance.units) {
            (SourceText::Utf8(text), DecodedSourceUnits::Utf8 { units, length }) => {
                let supplied = &text.as_bytes()[advance.start.offset..advance.end];
                for index in 0..length {
                    // Do not short-circuit: every precharged source read and
                    // comparison is performed, including a mismatch.
                    equal &= supplied[index] == units[index];
                }
            }
            (SourceText::Utf16(text), DecodedSourceUnits::Utf16 { units, length }) => {
                let supplied = &text.code_units()[advance.start.offset..advance.end];
                for index in 0..length {
                    equal &= supplied[index] == units[index];
                }
            }
            _ => return Err(ConsumedBoundaryError::ForeignCoordinates),
        }
        if !equal {
            return Err(ConsumedBoundaryError::ContentMismatch);
        }
        Ok(ConsumedSourceBoundary {
            source: advance.start.source,
            coordinates: advance.start.coordinates,
            offset: advance.end,
        })
    }

    /// Builds a range from actual established boundaries, without validating
    /// endpoints through the first byte beyond either consumed span.
    #[cfg(test)]
    pub(crate) fn range_between_consumed_boundaries(
        &self,
        start: &ConsumedSourceBoundary,
        end: &ConsumedSourceBoundary,
    ) -> Result<HtmlSourceRange, ConsumedBoundaryError> {
        self.require_consumed_boundary(start)?;
        self.require_consumed_boundary(end)?;
        if start.offset > end.offset {
            return Err(ConsumedBoundaryError::InvalidBoundary);
        }
        Ok(HtmlSourceRange {
            source: self.identity.clone(),
            coordinates: self.coordinates(),
            start: start.offset,
            end: end.offset,
        })
    }

    #[cfg(test)]
    pub(crate) fn require_consumed_boundary(
        &self,
        boundary: &ConsumedSourceBoundary,
    ) -> Result<(), ConsumedBoundaryError> {
        if boundary.source != self.identity {
            return Err(ConsumedBoundaryError::ForeignSource);
        }
        if boundary.coordinates != self.coordinates() {
            return Err(ConsumedBoundaryError::ForeignCoordinates);
        }
        if boundary.offset > self.len() {
            return Err(ConsumedBoundaryError::InvalidBoundary);
        }
        Ok(())
    }
}

#[cfg(test)]
mod consumed_boundary_tests {
    use super::*;

    #[test]
    fn available_decoded_spans_prove_exact_ranges_without_an_end_byte() {
        for (source, value, width, comparisons) in [
            (HtmlSource::utf8("🙂x"), 0x1F642, 4, 4),
            (
                HtmlSource::utf16(DocumentString::from("🙂x")),
                0x1F642,
                2,
                4,
            ),
        ] {
            let start = source.initial_consumed_boundary();
            assert_eq!(start.offset(), 0);
            let prepared = source
                .prepare_consumed_boundary(&start, value, width)
                .unwrap();
            assert_eq!(prepared.source_reads(), width);
            assert_eq!(prepared.comparison_bytes(), comparisons);
            let end = source.complete_consumed_boundary(prepared).unwrap();
            assert_eq!(end.offset(), width);
            let range = source
                .range_between_consumed_boundaries(&start, &end)
                .unwrap();
            assert_eq!(range, source.range(0, width).unwrap());
            let anchor = source
                .range_between_consumed_boundaries(&end, &end)
                .unwrap();
            assert_eq!(anchor, source.range(width, width).unwrap());
            assert_eq!(
                source.range_between_consumed_boundaries(&end, &start),
                Err(ConsumedBoundaryError::InvalidBoundary)
            );
        }
    }

    #[test]
    fn proofs_reject_unavailable_spans_and_mismatching_decoded_values() {
        let source = HtmlSource::utf8("¢x");
        let start = source.initial_consumed_boundary();
        assert!(matches!(
            source.prepare_consumed_boundary(&start, 0xA2, 1),
            Err(ConsumedBoundaryError::Unavailable)
        ));
        assert!(matches!(
            source.prepare_consumed_boundary(&start, 0x61, 4),
            Err(ConsumedBoundaryError::Unavailable)
        ));
        let wrong = source.prepare_consumed_boundary(&start, 0xA3, 2).unwrap();
        assert_eq!(
            source.complete_consumed_boundary(wrong),
            Err(ConsumedBoundaryError::ContentMismatch)
        );
        assert!(matches!(
            source.prepare_consumed_boundary(&start, 0xD800, 2),
            Err(ConsumedBoundaryError::InvalidScalar)
        ));
        assert!(matches!(
            source.prepare_consumed_boundary(&start, 0x110000, 2),
            Err(ConsumedBoundaryError::InvalidScalar)
        ));
    }

    #[test]
    fn proofs_are_qualified_by_actual_identity_and_coordinate_basis() {
        let source = HtmlSource::utf8("a");
        let start = source.initial_consumed_boundary();
        let foreign = HtmlSource::utf8("a");
        assert!(matches!(
            foreign.prepare_consumed_boundary(&start, 0x61, 1),
            Err(ConsumedBoundaryError::ForeignSource)
        ));
        assert_eq!(
            foreign.range_between_consumed_boundaries(&start, &start),
            Err(ConsumedBoundaryError::ForeignSource)
        );
        let clone = source.clone();
        let prepared = clone.prepare_consumed_boundary(&start, 0x61, 1).unwrap();
        let end = source.complete_consumed_boundary(prepared).unwrap();
        assert_eq!(
            source
                .range_between_consumed_boundaries(&start, &end)
                .unwrap(),
            source.range(0, 1).unwrap()
        );
        let mut wrong_basis = start.clone();
        wrong_basis.coordinates = HtmlSourceCoordinates::Utf16CodeUnits;
        assert!(matches!(
            source.prepare_consumed_boundary(&wrong_basis, 0x61, 1),
            Err(ConsumedBoundaryError::ForeignCoordinates)
        ));
        let mut invalid = start.clone();
        invalid.offset = 2;
        assert_eq!(
            source.require_consumed_boundary(&invalid),
            Err(ConsumedBoundaryError::InvalidBoundary)
        );
    }

    #[test]
    fn utf16_literal_surrogate_proofs_do_not_claim_scalar_validity() {
        let source = HtmlSource::utf16(DocumentString::from_utf16(vec![0xD800, 0x61]));
        let start = source.initial_consumed_boundary();
        let prepared = source.prepare_consumed_boundary(&start, 0xD800, 1).unwrap();
        assert_eq!(prepared.source_reads(), 1);
        assert_eq!(prepared.comparison_bytes(), 2);
        let end = source.complete_consumed_boundary(prepared).unwrap();
        assert_eq!(
            source
                .range_between_consumed_boundaries(&start, &end)
                .unwrap(),
            source.range(0, 1).unwrap()
        );
        let prepared = source.prepare_consumed_boundary(&end, 0x61, 2).unwrap();
        let last = source.complete_consumed_boundary(prepared).unwrap();
        assert_eq!(last.offset(), 2);
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
        self.end - self.start
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
