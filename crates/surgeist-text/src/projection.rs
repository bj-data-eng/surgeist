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
    /// Actual original-domain barriers; no rendering character represents them.
    pub(crate) barriers: Vec<usize>,
    /// Real variant-specific positions of captured anchor occurrences, when a
    /// selected replacement occupies a zero-authored insertion anchor.
    pub(crate) anchor_atoms: Vec<Option<usize>>,
    /// Canonical replacement style rows; absence resolves original source ownership.
    pub(crate) style_rows: Vec<Option<usize>>,
    /// Actual semantic component boundaries, independent of original offsets.
    pub(crate) barrier_atoms: Vec<usize>,
    pub(crate) transform: Option<std::sync::Arc<crate::transform::relation::TransformMap>>,
    segmentation_offsets: Vec<(usize, usize)>,
    /// Genuine zero-width original occurrences that need no incident atom.
    #[cfg(any(test, feature = "text-accessibility"))]
    original_points: Vec<(usize, usize)>,
}

/// Visits pinned ICU EGC boundaries in adjacent caller-supplied components.
/// All positions are UTF-8 bytes in `text`. The caller supplies its actual
/// scalar representation and hard cuts, including coverage endpoints, and owns
/// cut normalization, semantic mapping, seam deduplication and empty fallback.
/// This operation chooses neither control representatives nor barrier policy.
pub(crate) fn visit_grapheme_boundaries(
    text: &str,
    hard_cuts: &[usize],
    mut visit: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    if hard_cuts.first().copied() != Some(0) || hard_cuts.last().copied() != Some(text.len()) {
        return Err(Error::new(
            ErrorCode::InvalidSourceProcessing,
            "grapheme hard cuts do not cover the supplied UTF-8 stream",
        ));
    }
    for (index, &cut) in hard_cuts.iter().enumerate() {
        if !text.is_char_boundary(cut) || index > 0 && hard_cuts[index - 1] > cut {
            return Err(Error::new(
                ErrorCode::InvalidSourceProcessing,
                "grapheme hard cuts are not ordered UTF-8 scalar boundaries",
            ));
        }
    }
    for pair in hard_cuts.windows(2) {
        for byte in GraphemeClusterSegmenter::new().segment_str(&text[pair[0]..pair[1]]) {
            visit(pair[0].checked_add(byte).ok_or_else(length_overflow)?)?;
        }
    }
    Ok(())
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
                processed: Range::new(unit, source_end),
                presentation: Range::new(unit, source_end),
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
            #[cfg(any(test, feature = "text-accessibility"))]
            original_points: Vec::new(),
            graphemes: Vec::new(),
            barriers: Vec::new(),
            anchor_atoms: Vec::new(),
            style_rows: Vec::new(),
            barrier_atoms: Vec::new(),
            transform: None,
        };
        projection.graphemes = GraphemeClusterSegmenter::new()
            .segment_str(&projection.segmentation)
            .map(|byte| projection.segmentation_unit(byte))
            .collect::<Result<_>>()?;
        Ok(projection)
    }

    /// Original host/source projection, independent of lexical deletion or
    /// replacement. Genuine Wbr also separates decoding without resetting context.
    pub(crate) fn host(
        source: &Source,
        paragraph: &crate::protocol::paragraph::ParagraphContext,
    ) -> Result<Self> {
        if paragraph.base_direction.is_none() {
            return Self::new(source);
        }
        let barriers = paragraph_barriers(source, paragraph);
        let decode_boundaries = paragraph_decode_boundaries(paragraph, &barriers);
        admit_connected(source, &decode_boundaries, paragraph)?;
        let mut atoms = Vec::new();
        for range in paragraph.intersections(Range::new(0, source.text().len())) {
            decode_copy(
                &source.text().code_units()[range.start..range.end],
                range.start,
                range.start,
                &decode_boundaries,
                &mut atoms,
            )?;
        }
        let barrier_atoms = barriers
            .iter()
            .map(|at| atoms.partition_point(|atom| atom.source.end <= *at))
            .collect();
        let projection = Self::from_atoms_inner(
            atoms,
            paragraph.extent_in(source).end,
            barriers,
            barrier_atoms,
            false,
        )?;
        admit_physical_endpoint_cuts(&projection.atoms, paragraph)?;
        #[cfg(any(test, feature = "text-accessibility"))]
        let projection = {
            let mut projection = projection;
            let extent = paragraph.extent_in(source);
            let mut points: Vec<_> = paragraph
                .anchor_order
                .iter()
                .map(|anchor| {
                    crate::engine::analysis::anchor_position(anchor.item, paragraph, source.boxes())
                })
                .chain([extent.start, extent.end])
                .collect();
            points.sort_unstable();
            points.dedup();
            projection.original_points = points
                .into_iter()
                .map(|at| {
                    let index = projection
                        .atoms
                        .partition_point(|atom| atom.source.end <= at);
                    let byte = projection
                        .atoms
                        .get(index)
                        .map_or(projection.text.len(), |atom| atom.glyph_bytes.start);
                    (at, byte)
                })
                .collect();
            projection
        };
        Ok(projection)
    }

    /// Builds semantic input from actual executed correspondence, decoding Copy
    /// pieces before any removed original units can falsely join surrogate halves.
    pub(crate) fn from_processed(
        processed: &crate::ProcessedSource,
        paragraph: &crate::protocol::paragraph::ParagraphContext,
    ) -> Result<Self> {
        use crate::processing::MappingKind;
        let source = processed.source();
        let barriers = paragraph_barriers(source, paragraph);
        let decode_boundaries = paragraph_decode_boundaries(paragraph, &barriers);
        admit_connected(source, &decode_boundaries, paragraph)?;
        let mut atoms = Vec::new();
        for piece in processed.operations() {
            match piece.kind {
                MappingKind::Copy => {
                    for range in paragraph.intersections(piece.source) {
                        decode_copy(
                            &source.text().code_units()[range.start..range.end],
                            range.start,
                            piece.processed + range.start - piece.source.start,
                            &decode_boundaries,
                            &mut atoms,
                        )?;
                    }
                }
                MappingKind::Delete(_) => {}
                MappingKind::Replace => {
                    if !paragraph.contains(source, piece.source) {
                        continue;
                    }
                    let cp = u32::from(processed.processed_text().code_units()[piece.processed]);
                    let glyph = char::from_u32(cp).ok_or_else(length_overflow)?;
                    let range = Range::new(piece.processed, piece.processed_end());
                    atoms.push(OriginalAtom {
                        source: piece.source,
                        processed: range,
                        presentation: range,
                        code_point: cp,
                        glyph,
                        glyph_bytes: 0..0,
                    });
                }
            }
        }
        let projection = Self::from_atoms(
            atoms,
            processed.project(paragraph.extent(processed).end, crate::Affinity::After),
            barriers,
        )?;
        admit_physical_endpoint_cuts(&projection.atoms, paragraph)?;
        Ok(projection)
    }

    /// Reuses exact admitted atom provenance for a selected substituted sequence.
    pub(crate) fn from_atoms(
        atoms: Vec<OriginalAtom>,
        semantic_len: usize,
        barriers: Vec<usize>,
    ) -> Result<Self> {
        let barrier_atoms = barriers
            .iter()
            .map(|at| atoms.partition_point(|atom| atom.source.end <= *at))
            .collect();
        Self::from_atoms_at_barriers(atoms, semantic_len, barriers, barrier_atoms)
    }

    pub(crate) fn from_atoms_at_barriers(
        atoms: Vec<OriginalAtom>,
        semantic_len: usize,
        barriers: Vec<usize>,
        barrier_atoms: Vec<usize>,
    ) -> Result<Self> {
        Self::from_atoms_inner(atoms, semantic_len, barriers, barrier_atoms, true)
    }

    fn from_atoms_inner(
        mut atoms: Vec<OriginalAtom>,
        semantic_len: usize,
        barriers: Vec<usize>,
        barrier_atoms: Vec<usize>,
        control_presentation: bool,
    ) -> Result<Self> {
        let mut text = String::new();
        let mut segmentation = String::new();
        let mut segmentation_offsets = Vec::with_capacity(atoms.len() + 1);
        for atom in &mut atoms {
            segmentation_offsets.push((atom.presentation.start, segmentation.len()));
            let start = text.len();
            start
                .checked_add(atom.glyph.len_utf8())
                .ok_or_else(length_overflow)?;
            text.push(atom.glyph);
            atom.glyph_bytes = start..text.len();
            let representative = if (0xd800..=0xdfff).contains(&atom.code_point) {
                '\u{a6}'
            } else if control_presentation && crate::control::is_visible(atom.code_point) {
                crate::control::SEGMENTATION_SYMBOL
            } else {
                atom.glyph
            };
            segmentation
                .len()
                .checked_add(representative.len_utf8())
                .ok_or_else(length_overflow)?;
            segmentation.push(representative);
        }
        segmentation_offsets.push((semantic_len, segmentation.len()));
        let mut projection = Self {
            text,
            segmentation,
            atoms,
            graphemes: Vec::new(),
            barriers,
            anchor_atoms: Vec::new(),
            style_rows: Vec::new(),
            barrier_atoms,
            transform: None,
            segmentation_offsets,
            #[cfg(any(test, feature = "text-accessibility"))]
            original_points: Vec::new(),
        };
        let mut stops: Vec<_> = projection
            .barrier_atoms
            .iter()
            .map(|&atom| projection.segmentation_offsets[atom].1)
            .collect();
        stops.extend([0, projection.segmentation.len()]);
        stops.sort_unstable();
        stops.dedup();
        let mut graphemes = Vec::new();
        visit_grapheme_boundaries(&projection.segmentation, &stops, |byte| {
            let at = projection.segmentation_unit(byte)?;
            if graphemes.last() != Some(&at) {
                graphemes.push(at);
            }
            Ok(())
        })?;
        if graphemes.is_empty() {
            graphemes.push(semantic_len);
        }
        projection.graphemes = graphemes;
        Ok(projection)
    }

    #[cfg(any(test, feature = "text-accessibility"))]
    pub(crate) fn byte(&self, unit: usize) -> ProviderIndex {
        if let Ok(index) = self
            .original_points
            .binary_search_by_key(&unit, |point| point.0)
        {
            return ProviderIndex(self.original_points[index].1);
        }
        if unit
            == self
                .segmentation_offsets
                .last()
                .expect("projection terminal endpoint")
                .0
        {
            return ProviderIndex(self.text.len());
        }
        if let Ok(index) = self
            .atoms
            .binary_search_by_key(&unit, |atom| atom.source.start)
        {
            ProviderIndex(self.atoms[index].glyph_bytes.start)
        } else {
            let index = self
                .atoms
                .binary_search_by_key(&unit, |atom| atom.source.end)
                .expect("validated original piece endpoint");
            ProviderIndex(self.atoms[index].glyph_bytes.end)
        }
    }

    #[cfg(test)]
    pub(crate) fn unit(&self, byte: ProviderIndex) -> Result<usize> {
        if byte.0 == self.text.len() {
            return Ok(self
                .segmentation_offsets
                .last()
                .expect("projection terminal endpoint")
                .0);
        }
        self.atoms
            .binary_search_by_key(&byte.0, |atom| atom.glyph_bytes.start)
            .map(|index| self.atoms[index].source.start)
            .map_err(|_| boundary_error(byte.0))
    }

    /// Terminal coordinate of this actual semantic presentation, not Phase I.
    pub(crate) fn presentation_end(&self) -> usize {
        self.segmentation_offsets
            .last()
            .expect("projection endpoint")
            .0
    }

    pub(crate) fn segmentation_unit(&self, byte: usize) -> Result<usize> {
        self.segmentation_offsets
            .binary_search_by_key(&byte, |(_, byte)| *byte)
            .map(|index| self.segmentation_offsets[index].0)
            .map_err(|_| boundary_error(byte))
    }

    pub(crate) fn segmentation_byte(&self, unit: usize) -> usize {
        let index = self
            .segmentation_offsets
            .binary_search_by_key(&unit, |(unit, _)| *unit)
            .expect("validated segmentation boundary");
        self.segmentation_offsets[index].1
    }
}

/// Exact physical events require an actual decoded/processed atom seam.
/// Deleted intervals retain their distinct events; a retained scalar or Phase-I
/// replacement cannot be divided by choosing an affinity. In particular, use
/// admitted atoms rather than raw UTF-16 so genuine disconnected halves stay
/// separate original units.
fn admit_physical_endpoint_cuts(
    atoms: &[OriginalAtom],
    paragraph: &crate::protocol::paragraph::ParagraphContext,
) -> Result<()> {
    if paragraph.base_direction.is_none() {
        return Ok(());
    }
    for (control, row) in paragraph.controls.iter().enumerate() {
        if !crate::protocol::boundary::is_endpoint(&row.kind) {
            continue;
        }
        let cut = atoms.partition_point(|atom| atom.source.end <= row.at);
        if atoms
            .get(cut)
            .is_some_and(|atom| atom.source.start < row.at)
        {
            return Err(crate::protocol::protocol_error(
                crate::protocol::ProtocolError::InvalidControl {
                    control,
                    at: row.at,
                },
            ));
        }
    }
    Ok(())
}

fn paragraph_barriers(
    source: &Source,
    paragraph: &crate::protocol::paragraph::ParagraphContext,
) -> Vec<usize> {
    let mut barriers: Vec<_> = paragraph.disconnections().collect();
    barriers.extend(
        source
            .boxes()
            .iter()
            .enumerate()
            .filter(|(i, item)| {
                item.kind() == crate::InlineBoxKind::InFlow
                    && paragraph.anchor_order.iter().any(|anchor| {
                        anchor.item == crate::protocol::paragraph::SemanticAnchorItem::Atomic(*i)
                    })
            })
            .map(|(_, item)| item.index()),
    );
    barriers.extend(
        paragraph
            .controls
            .iter()
            .filter(|control| {
                control.kind == crate::protocol::paragraph::ParagraphControlKind::ForcedBreak
            })
            .map(|control| control.at),
    );
    barriers.extend(
        paragraph
            .computed_boundaries
            .iter()
            .map(|boundary| boundary.at),
    );
    barriers.sort_unstable();
    barriers.dedup();
    barriers
}

/// Decoding cuts are consumed before original atoms are frozen. Only the real
/// context barriers are retained by the rebuilt projection and its consumers.
fn paragraph_decode_boundaries(
    paragraph: &crate::protocol::paragraph::ParagraphContext,
    barriers: &[usize],
) -> Vec<usize> {
    let mut cuts = barriers.to_vec();
    cuts.extend(paragraph.wbr_decode_boundaries());
    cuts.sort_unstable();
    cuts.dedup();
    cuts
}

fn admit_connected(
    source: &Source,
    decode_boundaries: &[usize],
    paragraph: &crate::protocol::paragraph::ParagraphContext,
) -> Result<()> {
    let owned_text = paragraph
        .coverage
        .as_ref()
        .map(|_| paragraph.intersections(paragraph.extent_in(source)));
    for span in source.spans() {
        for (at, kind) in [
            (span.range().start, super::ShapingBoundary::SpanStart),
            (span.range().end, super::ShapingBoundary::SpanEnd),
        ] {
            // Scalar boundaries constrain incident original text. An authentic
            // parent occurrence inside a child hole owns only its point; it
            // cannot import the child's span boundary into parent shaping.
            let incident_text = owned_text.as_ref().is_none_or(|pieces| {
                pieces
                    .iter()
                    .any(|piece| piece.start < at && at < piece.end)
            });
            if incident_text && decode_boundaries.binary_search(&at).is_err() {
                SourceProjection::admit_boundary(source.text(), at, kind)?;
            }
        }
    }
    Ok(())
}

fn decode_copy(
    units: &[u16],
    original_start: usize,
    processed_start: usize,
    decode_boundaries: &[usize],
    atoms: &mut Vec<OriginalAtom>,
) -> Result<()> {
    if units.is_empty() {
        return Ok(());
    }
    let original_end = original_start
        .checked_add(units.len())
        .ok_or_else(length_overflow)?;
    let first = decode_boundaries.partition_point(|&at| at <= original_start);
    let end = decode_boundaries.partition_point(|&at| at < original_end);
    let mut starts = std::iter::once(original_start)
        .chain(decode_boundaries[first..end].iter().copied())
        .chain(std::iter::once(original_end));
    let Some(mut start) = starts.next() else {
        return Ok(());
    };
    for end in starts {
        let mut at = start;
        for decoded in char::decode_utf16(
            units[start - original_start..end - original_start]
                .iter()
                .copied(),
        ) {
            let (cp, glyph, len) = match decoded {
                Ok(ch) => (u32::from(ch), ch, ch.len_utf16()),
                Err(error) => (u32::from(error.unpaired_surrogate()), '\u{fffd}', 1),
            };
            let next = at.checked_add(len).ok_or_else(length_overflow)?;
            let p = processed_start
                .checked_add(at - original_start)
                .ok_or_else(length_overflow)?;
            let q = p.checked_add(len).ok_or_else(length_overflow)?;
            atoms.push(OriginalAtom {
                source: Range::new(at, next),
                processed: Range::new(p, q),
                presentation: Range::new(p, q),
                glyph_bytes: 0..0,
                code_point: cp,
                glyph,
            });
            at = next;
        }
        start = end;
    }
    Ok(())
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

    #[test]
    fn canonical_graphemes_use_supplied_scalars_without_control_presentation() {
        // Unicode 17 GB4/GB5 break around controls; GB9 attaches Extend.
        for (text, expected) in [
            ("a\u{301}", vec![0, 3]),
            ("a\0\u{301}", vec![0, 1, 2, 4]),
            ("9\u{fe0f}\u{20e3}", vec![0, 7]),
        ] {
            let mut boundaries = Vec::new();
            visit_grapheme_boundaries(text, &[0, text.len()], |byte| {
                boundaries.push(byte);
                Ok(())
            })
            .unwrap();
            assert_eq!(boundaries, expected);
        }
    }

    #[test]
    fn canonical_graphemes_preserve_only_the_callers_actual_hard_cuts() {
        let text = "a\u{301}";
        let mut boundaries = Vec::new();
        visit_grapheme_boundaries(text, &[0, 1, text.len()], |byte| {
            boundaries.push(byte);
            Ok(())
        })
        .unwrap();
        assert_eq!(boundaries, [0, 1, 1, 3]);
    }

    #[test]
    fn malformed_grapheme_cuts_refuse_before_any_partial_boundary_output() {
        let text = "a\u{301}";
        for cuts in [
            vec![],
            vec![0, 1],
            vec![1, 3],
            vec![0, 2, 3],
            vec![0, 3, 1, 3],
            vec![0, 4, 3],
        ] {
            let mut boundaries = Vec::new();
            let error = visit_grapheme_boundaries(text, &cuts, |byte| {
                boundaries.push(byte);
                Ok(())
            })
            .unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidSourceProcessing);
            assert!(boundaries.is_empty());
        }
    }
}
