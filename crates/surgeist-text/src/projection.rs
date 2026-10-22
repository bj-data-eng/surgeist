use icu_segmenter::GraphemeClusterSegmenter;

use super::{Error, ErrorCode, ErrorDetail, Range, Result, Source, Text};
use crate::engine::OriginalAtom;

/// A byte boundary in the glyph-only UTF-8 projection.
#[cfg(any(test, feature = "text-accessibility"))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProviderIndex(pub(crate) usize);

/// Immutable original-source associations. Analysis never reads glyph replacement
/// properties: isolated units retain their original code point in `atoms`.
#[derive(Clone, Debug)]
pub(crate) struct SourceProjection {
    pub(crate) text: String,
    pub(crate) segmentation: String,
    pub(crate) atoms: Vec<OriginalAtom>,
    pub(crate) graphemes: Vec<usize>,
    segmentation_offsets: Vec<(usize, usize)>,
}

impl SourceProjection {
    pub(crate) fn admit(source: &Source) -> Result<()> {
        for span in source.spans() {
            Self::admit_boundary(
                source.text(),
                span.range().start,
                super::ShapingBoundary::SpanStart,
            )?;
            Self::admit_boundary(
                source.text(),
                span.range().end,
                super::ShapingBoundary::SpanEnd,
            )?;
        }
        for box_ in source.boxes() {
            Self::admit_boundary(
                source.text(),
                box_.index(),
                super::ShapingBoundary::InlineBox,
            )?;
        }
        Ok(())
    }

    fn admit_boundary(text: &Text, index: usize, kind: super::ShapingBoundary) -> Result<()> {
        if !text.is_scalar_boundary(index) {
            return Err(Error::new(
                ErrorCode::UnsupportedFeature,
                "shaping boundary is inside a UTF-16 surrogate pair",
            )
            .with_detail(ErrorDetail::UnsupportedShapingBoundary { index, kind }));
        }
        Ok(())
    }

    pub(crate) fn new(source: &Source) -> Result<Self> {
        Self::admit(source)?;
        let mut text = String::new();
        let mut segmentation = String::new();
        let mut atoms = Vec::new();
        let mut segmentation_offsets = Vec::new();
        let mut unit: usize = 0;
        for decoded in char::decode_utf16(source.text().code_units().iter().copied()) {
            let (code_point, glyph, representative, length) = match decoded {
                Ok(ch) => (u32::from(ch), ch, ch, ch.len_utf16()),
                Err(error) => (
                    u32::from(error.unpaired_surrogate()),
                    '\u{fffd}',
                    '\u{a6}',
                    1,
                ),
            };
            segmentation_offsets.push((unit, segmentation.len()));
            let end = text
                .len()
                .checked_add(glyph.len_utf8())
                .ok_or_else(length_overflow)?;
            segmentation
                .len()
                .checked_add(representative.len_utf8())
                .ok_or_else(length_overflow)?;
            let source_end = unit.checked_add(length).ok_or_else(length_overflow)?;
            atoms.push(OriginalAtom {
                source: Range::new(unit, source_end),
                glyph_bytes: text.len()..end,
                code_point,
                glyph,
            });
            text.push(glyph);
            segmentation.push(representative);
            unit = source_end;
        }
        segmentation_offsets.push((unit, segmentation.len()));
        let mut projection = Self {
            text,
            segmentation,
            atoms,
            segmentation_offsets,
            graphemes: Vec::new(),
        };
        projection.graphemes = GraphemeClusterSegmenter::new()
            .segment_str(&projection.segmentation)
            .map(|byte| projection.segmentation_unit(byte))
            .collect::<Result<_>>()?;
        Ok(projection)
    }

    #[cfg(any(test, feature = "text-accessibility"))]
    pub(crate) fn byte(&self, unit: usize) -> ProviderIndex {
        if unit == self.atoms.last().map_or(0, |atom| atom.source.end) {
            return ProviderIndex(self.text.len());
        }
        let index = self
            .atoms
            .binary_search_by_key(&unit, |atom| atom.source.start)
            .expect("validated shaping boundary");
        ProviderIndex(self.atoms[index].glyph_bytes.start)
    }

    #[cfg(test)]
    pub(crate) fn unit(&self, byte: ProviderIndex) -> Result<usize> {
        if byte.0 == self.text.len() {
            return Ok(self.atoms.last().map_or(0, |atom| atom.source.end));
        }
        self.atoms
            .binary_search_by_key(&byte.0, |atom| atom.glyph_bytes.start)
            .map(|index| self.atoms[index].source.start)
            .map_err(|_| boundary_error(byte.0))
    }

    pub(crate) fn segmentation_unit(&self, byte: usize) -> Result<usize> {
        self.segmentation_offsets
            .binary_search_by_key(&byte, |(_, byte)| *byte)
            .map(|index| self.segmentation_offsets[index].0)
            .map_err(|_| boundary_error(byte))
    }

    #[cfg(test)]
    pub(crate) fn segmentation_byte(&self, unit: usize) -> usize {
        let index = self
            .segmentation_offsets
            .binary_search_by_key(&unit, |(unit, _)| *unit)
            .expect("validated segmentation boundary");
        self.segmentation_offsets[index].1
    }
}

fn boundary_error(byte: usize) -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "provider returned an invalid source boundary",
    )
    .with_detail(ErrorDetail::ProviderSourceRange {
        start: byte,
        end: byte,
    })
}

pub(crate) fn length_overflow() -> Error {
    Error::new(
        ErrorCode::SourceLengthOverflow,
        "source length is not representable",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_atoms_keep_isolated_units_distinct_from_both_representatives() {
        let source = Source::new(Text::from_code_units(vec![
            0xd800, 0xfffd, 0xa6, 0xd83d, 0xde00, 0xdc00,
        ]));
        let projection = SourceProjection::new(&source).unwrap();
        assert_eq!(projection.text, "\u{fffd}\u{fffd}\u{a6}😀\u{fffd}");
        assert_eq!(projection.segmentation, "\u{a6}\u{fffd}\u{a6}😀\u{a6}");
        assert_eq!(
            projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0xd800, 0xfffd, 0xa6, 0x1f600, 0xdc00]
        );
        assert_eq!(
            projection
                .atoms
                .iter()
                .map(|atom| atom.source)
                .collect::<Vec<_>>(),
            [
                Range::new(0, 1),
                Range::new(1, 2),
                Range::new(2, 3),
                Range::new(3, 5),
                Range::new(5, 6)
            ]
        );
        assert_eq!(
            projection
                .atoms
                .iter()
                .map(|atom| atom.glyph_bytes.clone())
                .collect::<Vec<_>>(),
            [0..3, 3..6, 6..8, 8..12, 12..15]
        );
        assert_eq!(
            [0, 1, 2, 3, 5, 6].map(|unit| projection.segmentation_byte(unit)),
            [0, 2, 5, 7, 11, 13]
        );
        for index in [0, 1, 2, 3, 5, 6] {
            assert_eq!(projection.unit(projection.byte(index)).unwrap(), index);
            assert_eq!(
                projection
                    .segmentation_unit(projection.segmentation_byte(index))
                    .unwrap(),
                index
            );
        }
        assert!(projection.unit(ProviderIndex(1)).is_err());
    }
}
