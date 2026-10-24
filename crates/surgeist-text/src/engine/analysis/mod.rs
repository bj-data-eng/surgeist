// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 analysis/mod.rs: original UTF-16 properties,
// separate segmentation input, usize indices, and occurrence-backed atomics.

pub(crate) mod cluster;
pub(crate) mod hyphenation;
pub(crate) mod wrapping;

use std::ops::Range;

use icu_normalizer::properties::{
    CanonicalComposition, CanonicalCompositionBorrowed, CanonicalDecomposition,
    CanonicalDecompositionBorrowed,
};
use icu_properties::props::{
    BidiClass, BidiMirroringGlyph, GeneralCategory, GraphemeClusterBreak, LineBreak, Script,
};
use icu_properties::{
    CodePointMapData, CodePointMapDataBorrowed, PropertyNamesShort, PropertyNamesShortBorrowed,
};
use icu_segmenter::options::{LineBreakOptions, LineBreakWordOption, WordBreakInvariantOptions};
use icu_segmenter::{LineSegmenter, WordSegmenter};
use parley_data::Properties;

use super::{bidi::BidiResolver, resolve::AnalyzedStyleRun};
use crate::protocol::paragraph::{ParagraphAnchorItem, ParagraphData};
use crate::{InlineBox, InlineBoxKind, Result, WordBreak, projection::SourceProjection};

#[derive(Default)]
pub(crate) struct AnalysisDataSources;

impl AnalysisDataSources {
    pub(crate) fn properties(&self, ch: char) -> Properties {
        Properties::get(ch)
    }

    fn original_properties(&self, cp: u32) -> Properties {
        if let Some(ch) = char::from_u32(cp) {
            return self.properties(ch);
        }
        // Only original isolated UTF-16 units reach this path. No rendering
        // replacement or segmentation representative supplies these properties.
        Properties::new(
            CodePointMapData::<Script>::new().get32(cp),
            CodePointMapData::<GeneralCategory>::new().get32(cp),
            CodePointMapData::<GraphemeClusterBreak>::new().get32(cp),
            CodePointMapData::<BidiClass>::new().get32(cp),
            false,
            false,
            false,
            false,
        )
    }

    pub(crate) fn script_short_name(&self) -> PropertyNamesShortBorrowed<'static, Script> {
        PropertyNamesShort::new()
    }
    fn composing_normalizer(&self) -> CanonicalCompositionBorrowed<'_> {
        CanonicalComposition::new()
    }
    fn decomposing_normalizer(&self) -> CanonicalDecompositionBorrowed<'_> {
        CanonicalDecomposition::new()
    }
    fn brackets(&self) -> CodePointMapDataBorrowed<'_, BidiMirroringGlyph> {
        CodePointMapData::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Boundary {
    None,
    Word,
    Line,
    Mandatory,
}

/// The hard-break role of the actual semantic code point at this phase.
/// Native callers pass the executed processed atom, so a collapsed document
/// LF that became SPACE is not mistaken for an untouched hard separator.
pub(crate) fn is_hard_separator(code_point: u32) -> bool {
    matches!(
        CodePointMapData::<LineBreak>::new().get32(code_point),
        LineBreak::MandatoryBreak
            | LineBreak::CarriageReturn
            | LineBreak::LineFeed
            | LineBreak::NextLine
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CharInfo {
    pub(crate) boundary: Boundary,
    pub(crate) script: Script,
    pub(crate) grapheme_cluster_break: GraphemeClusterBreak,
    pub(crate) bidi_class: BidiClass,
    pub(crate) bracket: BidiMirroringGlyph,
    control: bool,
    emoji: bool,
    contributes: bool,
    normalize: bool,
}

impl CharInfo {
    pub(crate) fn is_control(self) -> bool {
        self.control
    }
    pub(crate) fn is_emoji_or_pictograph(self) -> bool {
        self.emoji
    }
    pub(crate) fn contributes_to_shaping(self) -> bool {
        self.contributes
    }
    pub(crate) fn force_normalize(self) -> bool {
        self.normalize
    }
}

#[derive(Clone)]
pub(crate) struct Analysis {
    pub(crate) infos: Vec<(CharInfo, usize)>,
    pub(crate) levels: Vec<u8>,
    pub(crate) box_levels: Vec<u8>,
    pub(crate) control_levels: Vec<u8>,
    pub(crate) base_level: u8,
    /// Actual segmentation boundaries expressed as half-open atom intervals.
    pub(crate) graphemes: Vec<Range<usize>>,
}

pub(crate) fn analyze(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    boxes: &[InlineBox],
    paragraph: &ParagraphData,
) -> Result<(Analysis, Vec<Range<usize>>)> {
    let sources = AnalysisDataSources;
    for (policy, row) in paragraph.boundary_policies.iter().enumerate() {
        let at = row.at(paragraph, boxes);
        let original_end = styles
            .iter()
            .filter_map(|style| style.source_range())
            .map(|range| range.end)
            .max()
            .unwrap_or(0);
        let wrap_at = |at| {
            styles[crate::engine::resolve::source_style(styles, at)]
                .style
                .authored()
                .wrap
        };
        let before_at = match row.definition {
            crate::ParagraphBoundaryDefinition::Connection(i) => paragraph
                .connection_ends
                .as_ref()
                .map_or(at, |ends| ends[i].0),
            _ => at,
        };
        let preceding = paragraph.coverage.as_ref().map_or_else(
            || before_at.checked_sub(1),
            |pieces| {
                pieces
                    .iter()
                    .rev()
                    .find(|piece| !piece.is_empty() && piece.start < before_at)
                    .map(|piece| piece.end.min(before_at) - 1)
            },
        );
        let following = paragraph.coverage.as_ref().map_or_else(
            || (at < original_end).then_some(at),
            |pieces| {
                pieces
                    .iter()
                    .find(|piece| !piece.is_empty() && piece.end > at)
                    .map(|piece| piece.start.max(at))
            },
        );
        if preceding.is_some_and(|at| wrap_at(at) != row.preceding_wrap)
            || following.is_some_and(|at| wrap_at(at) != row.following_wrap)
        {
            return Err(crate::protocol::protocol_error(
                crate::protocol::ProtocolError::InvalidBoundaryPolicy { policy, at },
            ));
        }
    }
    let offsets: Vec<_> = projection
        .segmentation
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(projection.segmentation.len()))
        .collect();
    let atom_at = |byte: usize| {
        offsets
            .binary_search(&byte)
            .expect("ICU segmentation returns scalar boundaries in its admitted projection")
    };
    let grapheme_stops: Vec<_> = projection
        .graphemes
        .iter()
        .map(|unit| {
            projection
                .atoms
                .partition_point(|atom| atom.presentation.start < *unit)
        })
        .collect();
    let graphemes: Vec<_> = grapheme_stops
        .windows(2)
        .map(|pair| pair[0]..pair[1])
        .collect();
    let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
    let mut word_ranges = Vec::new();
    let mut component_stops = projection.barrier_atoms.clone();
    component_stops.extend([0, projection.atoms.len()]);
    component_stops.sort_unstable();
    component_stops.dedup();
    for component in component_stops.windows(2) {
        let component_start = component[0];
        let component_end = component[1];
        let start_byte = offsets[component_start];
        let end_byte = offsets[component_end];
        let mut previous = component_start;
        for (byte, kind) in
            WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default())
                .segment_str(&projection.segmentation[start_byte..end_byte])
                .iter_with_word_type()
        {
            let atom = atom_at(byte + start_byte);
            boundaries[atom] = Boundary::Word;
            if previous < atom && kind.is_word_like() {
                word_ranges.push(previous..atom);
            }
            previous = atom;
        }
        // Preserve complete neighboring atoms for a change of word-break mode,
        // but never borrow text across an actual disconnected/in-flow edge.
        let mut first = component_start;
        while first < component_end {
            let style_index = style_at(projection, styles, first);
            let mode = styles[style_index].style.authored().word_break;
            let mut end = first + 1;
            while end < component_end {
                let next = style_at(projection, styles, end);
                if styles[next].style.authored().word_break != mode {
                    break;
                }
                end += 1;
            }
            let begin_context = first.saturating_sub(1).max(component_start);
            let end_context = end.saturating_add(1).min(component_end);
            let start_byte = offsets[begin_context];
            let end_byte = offsets[end_context];
            let mut options = LineBreakOptions::default();
            options.word_option = Some(match mode {
                WordBreak::Normal => LineBreakWordOption::Normal,
                WordBreak::BreakAll => LineBreakWordOption::BreakAll,
                WordBreak::KeepAll => LineBreakWordOption::KeepAll,
            });
            for byte in LineSegmenter::new_for_non_complex_scripts(options)
                .segment_str(&projection.segmentation[start_byte..end_byte])
            {
                let atom = atom_at(byte + start_byte);
                if atom > 0 && atom >= first && atom < end {
                    boundaries[atom] = Boundary::Line;
                }
            }
            first = end;
        }
    }
    let mut infos = Vec::with_capacity(projection.atoms.len());
    let mut mandatory = false;
    for (index, atom) in projection.atoms.iter().enumerate() {
        let props = sources.original_properties(atom.code_point);
        let script = props.script();
        let gcb = props.grapheme_cluster_break();
        let style = style_at(projection, styles, index);
        let info = CharInfo {
            boundary: if mandatory {
                Boundary::Mandatory
            } else {
                boundaries[index]
            },
            script,
            grapheme_cluster_break: gcb,
            // One pinned ICU authority reads the original code point for both
            // scalars and isolated units, independently of either projection.
            bidi_class: CodePointMapData::<BidiClass>::new().get32(atom.code_point),
            bracket: sources.brackets().get32(atom.code_point),
            control: props.general_category() == GeneralCategory::Control,
            emoji: props.is_emoji_or_pictograph(),
            contributes: contributes_to_shaping(props.general_category(), script),
            normalize: (gcb == GraphemeClusterBreak::Extend
                && atom.code_point != 0x200c
                && !props.is_variation_selector())
                || gcb == GraphemeClusterBreak::SpacingMark,
        };
        mandatory = is_hard_separator(atom.code_point);
        infos.push((info, style));
    }
    let mut original = Vec::with_capacity(infos.len() + paragraph.anchor_order.len());
    let mut owners = Vec::with_capacity(original.capacity());
    let positions: Vec<_> = (0..paragraph.anchor_order.len())
        .map(|anchor| anchor_atom(anchor, paragraph, boxes, projection))
        .collect();
    for (position, pair) in positions.windows(2).enumerate() {
        if pair[0] > pair[1] {
            let item = paragraph.anchor_order[position + 1].item;
            let (kind, occurrence) = anchor_kind(item);
            return Err(crate::protocol::protocol_error(
                crate::protocol::ProtocolError::InvalidAnchorOrder {
                    position: position + 1,
                    kind,
                    occurrence,
                },
            ));
        }
    }
    let mut next_anchor = paragraph.anchor_order.iter().zip(&positions).peekable();
    for (index, atom) in projection.atoms.iter().enumerate() {
        while next_anchor.peek().is_some_and(|(_, at)| **at <= index) {
            let (anchor, _) = next_anchor.next().expect("peek established an anchor");
            push_bidi_anchor(
                anchor.item,
                paragraph,
                boxes,
                &sources,
                &mut original,
                &mut owners,
            );
        }
        original.push((
            atom.glyph,
            (infos[index].0.bidi_class, infos[index].0.bracket),
        ));
        owners.push(BidiOwner::Atom(index));
    }
    for (anchor, _) in next_anchor {
        push_bidi_anchor(
            anchor.item,
            paragraph,
            boxes,
            &sources,
            &mut original,
            &mut owners,
        );
    }
    let mut bidi = BidiResolver::new();
    bidi.resolve(
        original.into_iter(),
        paragraph.base_direction.map(|direction| direction.level()),
    );
    let mut levels = vec![bidi.base_level(); infos.len()];
    let mut box_levels = vec![bidi.base_level(); boxes.len()];
    let mut control_levels = vec![bidi.base_level(); paragraph.controls.len()];
    for (owner, &level) in owners.into_iter().zip(bidi.levels()) {
        match owner {
            BidiOwner::Atom(index) => levels[index] = level,
            BidiOwner::Atomic(index) => box_levels[index] = level,
            BidiOwner::Control(index) => control_levels[index] = level,
        }
    }
    for (anchor, &at) in paragraph.anchor_order.iter().zip(&positions) {
        let inherited = match anchor.affinity {
            crate::Affinity::Before => at.checked_sub(1).and_then(|i| levels.get(i)),
            crate::Affinity::After => levels.get(at),
        }
        .copied()
        .unwrap_or(bidi.base_level());
        match anchor.item {
            ParagraphAnchorItem::Atomic(i) if boxes[i].kind() == InlineBoxKind::OutOfFlow => {
                box_levels[i] = inherited
            }
            ParagraphAnchorItem::Control(i)
                if paragraph.controls[i].kind.code_point().is_none()
                    && paragraph.controls[i].kind
                        != crate::protocol::paragraph::ParagraphControlKind::ForcedBreak =>
            {
                control_levels[i] = inherited
            }
            _ => {}
        }
    }
    Ok((
        Analysis {
            infos,
            levels,
            box_levels,
            control_levels,
            base_level: bidi.base_level(),
            graphemes,
        },
        word_ranges,
    ))
}

fn style_at(projection: &SourceProjection, styles: &[AnalyzedStyleRun], atom: usize) -> usize {
    projection
        .style_rows
        .get(atom)
        .and_then(|value| *value)
        .unwrap_or_else(|| {
            crate::engine::resolve::source_style(styles, projection.atoms[atom].source.start)
        })
}

enum BidiOwner {
    Atom(usize),
    Atomic(usize),
    Control(usize),
}

pub(crate) fn anchor_position(
    item: ParagraphAnchorItem,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
) -> usize {
    match item {
        ParagraphAnchorItem::EmptyOrigin(index) => paragraph.origins[index].aggregate.start,
        ParagraphAnchorItem::Atomic(index) => boxes[index].index(),
        ParagraphAnchorItem::Control(index) => paragraph.controls[index].at,
    }
}

pub(crate) fn anchor_atom(
    occurrence: usize,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
    projection: &SourceProjection,
) -> usize {
    if let Some(Some(atom)) = projection.anchor_atoms.get(occurrence) {
        return *atom;
    }
    let anchor = paragraph.anchor_order[occurrence];
    let source = anchor_position(anchor.item, paragraph, boxes);
    let mut index = projection
        .atoms
        .partition_point(|atom| atom.source.end <= source);
    if projection
        .atoms
        .get(index)
        .is_some_and(|atom| atom.source.start < source)
        && anchor.affinity == crate::Affinity::After
    {
        index += 1;
    }
    if matches!(anchor.item, ParagraphAnchorItem::Atomic(i) if boxes[i].kind() == InlineBoxKind::InFlow)
    {
        return index;
    }
    // The raw standalone Source front historically orders transparent boxes at
    // its admitted scalar endpoint, even inside a CRLF/combining grapheme.
    // Explicit native ParagraphAnchor affinity still projects typography stops.
    if paragraph.base_direction.is_none()
        && matches!(anchor.item,ParagraphAnchorItem::Atomic(i) if boxes[i].kind()==InlineBoxKind::OutOfFlow)
    {
        return index;
    }
    let unit = projection.atoms.get(index).map_or_else(
        || {
            projection
                .atoms
                .last()
                .map_or(0, |atom| atom.presentation.end)
        },
        |atom| atom.presentation.start,
    );
    match projection.graphemes.binary_search(&unit) {
        Ok(_) => index,
        Err(next) => {
            let stop = match anchor.affinity {
                crate::Affinity::Before => projection.graphemes[next.saturating_sub(1)],
                crate::Affinity::After => projection.graphemes[next],
            };
            projection
                .atoms
                .partition_point(|atom| atom.presentation.start < stop)
        }
    }
}

fn anchor_kind(item: ParagraphAnchorItem) -> (crate::protocol::ProtocolHandleKind, usize) {
    match item {
        ParagraphAnchorItem::EmptyOrigin(i) => (crate::protocol::ProtocolHandleKind::Origin, i),
        ParagraphAnchorItem::Atomic(i) => (crate::protocol::ProtocolHandleKind::Atomic, i),
        ParagraphAnchorItem::Control(i) => (crate::protocol::ProtocolHandleKind::Control, i),
    }
}

fn push_bidi_anchor(
    item: ParagraphAnchorItem,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
    sources: &AnalysisDataSources,
    original: &mut Vec<(char, (BidiClass, BidiMirroringGlyph))>,
    owners: &mut Vec<BidiOwner>,
) {
    let (cp, owner) = match item {
        ParagraphAnchorItem::Atomic(index) if boxes[index].kind() == InlineBoxKind::InFlow => {
            (0xfffc, BidiOwner::Atomic(index))
        }
        ParagraphAnchorItem::Control(index) => {
            let control = &paragraph.controls[index];
            let cp =
                if control.kind == crate::protocol::paragraph::ParagraphControlKind::ForcedBreak {
                    Some(0xa)
                } else {
                    control.kind.code_point()
                };
            let Some(cp) = cp else { return };
            (cp, BidiOwner::Control(index))
        }
        _ => return,
    };
    let ch = char::from_u32(cp).expect("closed semantic controls are Unicode scalars");
    original.push((
        ch,
        (
            CodePointMapData::<BidiClass>::new().get32(cp),
            sources.brackets().get32(cp),
        ),
    ));
    owners.push(owner);
}

pub(crate) fn contributes_to_shaping(category: GeneralCategory, script: Script) -> bool {
    !matches!(
        category,
        GeneralCategory::Control
            | GeneralCategory::LineSeparator
            | GeneralCategory::ParagraphSeparator
    ) && !(category == GeneralCategory::Format && script != Script::Inherited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_original_surrogate_has_original_left_to_right_property() {
        let sources = AnalysisDataSources;
        for cp in 0xd800..=0xdfff {
            let props = sources.original_properties(cp);
            assert_eq!(props.bidi_class(), BidiClass::LeftToRight);
            assert_eq!(props.general_category(), GeneralCategory::Surrogate);
            assert_eq!(props.script(), Script::Unknown);
        }
        assert_eq!(
            sources.properties('\u{fffd}').bidi_class(),
            BidiClass::OtherNeutral
        );
    }

    #[test]
    fn original_surrogate_and_literal_replacement_choose_distinct_bidi_bases() {
        let sources = AnalysisDataSources;
        let mut bidi = BidiResolver::new();
        for (cp, expected) in [(0xd800, 0), (0xfffd, 1)] {
            let characters = [(cp, '\u{fffd}'), (0x05d0, '\u{05d0}')];
            bidi.resolve(
                characters.into_iter().map(|(cp, glyph)| {
                    (
                        glyph,
                        (
                            sources.original_properties(cp).bidi_class(),
                            sources.brackets().get32(cp),
                        ),
                    )
                }),
                None,
            );
            assert_eq!(bidi.base_level(), expected);
            assert_eq!(bidi.levels().len(), 2);
        }
    }

    #[test]
    fn segmentation_projection_does_not_supply_original_bidi_or_scalar_identity() {
        let source = crate::Source::new(crate::Text::from_code_units(vec![
            0xd800, 0x0301, 0xdc00, 0x05d0,
        ]));
        let projection = SourceProjection::new(&source).expect("original units are retained");
        let styles = [AnalyzedStyleRun {
            owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(0, 4)),
            style: crate::ValidatedStyle::try_from(crate::Style::default()).expect("default style"),
        }];
        let (analysis, _) = analyze(
            &projection,
            &styles,
            &[],
            &ParagraphData::standalone(&source),
        )
        .expect("original analysis");
        assert_eq!(analysis.base_level, 0);
        assert_eq!(analysis.infos[0].0.bidi_class, BidiClass::LeftToRight);
        assert_eq!(analysis.infos[2].0.bidi_class, BidiClass::LeftToRight);
        assert_eq!(analysis.graphemes, [0..2, 2..3, 3..4]);
        assert_eq!(projection.atoms[0].code_point, 0xd800);
        assert_eq!(projection.atoms[2].code_point, 0xdc00);
        assert_eq!(projection.atoms[0].glyph, '\u{fffd}');
    }
}
