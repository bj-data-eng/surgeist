// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 analysis/mod.rs: original UTF-16 properties,
// separate segmentation input, usize indices, and occurrence-backed atomics.

pub(crate) mod cluster;
pub(crate) mod direction;
pub(crate) mod hyphenation;
mod lexical_fallback;
mod line_break;
pub(crate) mod line_units;
pub(crate) mod phrase;
pub(crate) mod scope;
pub(crate) mod unit_properties;
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
use icu_segmenter::WordSegmenter;
use icu_segmenter::options::WordBreakInvariantOptions;
use parley_data::Properties;

use super::{bidi::BidiResolver, resolve::AnalyzedStyleRun};
use crate::protocol::paragraph::{ParagraphData, SemanticAnchorItem};
use crate::{InlineBox, InlineBoxKind, Result, projection::SourceProjection};

#[derive(Default)]
pub(crate) struct AnalysisDataSources;

impl AnalysisDataSources {
    pub(crate) fn properties(&self, ch: char) -> Properties {
        if crate::control::is_visible(ch as u32) {
            crate::control::properties()
        } else {
            Properties::get(ch)
        }
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
    /// Text-owned ordinary candidate that still requires an actual mixed pair.
    Anywhere,
    Mandatory,
}

/// Actual pre-preference line causes; prepared authenticates mixed custody and
/// suppresses only the eligible implicit owner's cause, never a merged bit.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum LineCause {
    Implicit,
    RequiredLineBreak,
    SourceZwsp { atom: usize },
    Whitespace { atom: usize },
    LexicalFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LineCandidate {
    pub(crate) atom: usize,
    pub(crate) style: usize,
    pub(crate) cause: LineCause,
}

pub(super) fn capacity() -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::TextAnalysisCapacity,
        "text analysis could not retain its complete result",
    )
}

fn issue_candidate(rows: &mut Vec<LineCandidate>, row: LineCandidate) -> Result<()> {
    rows.try_reserve(1).map_err(|_| capacity())?;
    rows.push(row);
    Ok(())
}

/// The hard-break role of the actual semantic code point at this phase.
/// Native callers pass the executed processed atom, so a collapsed document
/// LF that became SPACE is not mistaken for an untouched hard separator.
pub(crate) fn is_hard_separator(code_point: u32) -> bool {
    if crate::control::is_visible(code_point) {
        return false;
    }
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
    /// Missing-lexical line proposal; never a detected word boundary.
    lexical_fallback_before: bool,
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
    pub(crate) fn lexical_fallback_before(self) -> bool {
        self.lexical_fallback_before
    }
    pub(crate) fn has_ordinary_line_candidate(self) -> bool {
        self.boundary >= Boundary::Line || self.lexical_fallback_before
    }
}

#[derive(Clone)]
pub(crate) struct Analysis {
    pub(crate) infos: Vec<(CharInfo, usize)>,
    pub(crate) levels: Vec<u8>,
    pub(crate) box_levels: Vec<u8>,
    pub(crate) control_levels: Vec<u8>,
    pub(crate) computed_levels: Vec<u8>,
    pub(crate) base_level: u8,
    /// Actual segmentation boundaries expressed as half-open atom intervals.
    pub(crate) graphemes: Vec<Range<usize>>,
    /// Operation-specific restrictions; editing/shaping EGCs stay unchanged.
    pub(crate) line_units: line_units::LineBreakUnits,
    pub(crate) line_candidates: Vec<LineCandidate>,
    pub(crate) phrases: phrase::PhraseCatalog,
}

pub(crate) fn analyze(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    boxes: &[InlineBox],
    paragraph: &ParagraphData,
) -> Result<(Analysis, Vec<Range<usize>>, direction::DirectionBody)> {
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
    let line_units = line_units::LineBreakUnits::new(projection)?;
    let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
    let mut word_ranges = Vec::new();
    let mut line_candidates = Vec::new();
    let line_policies = line_break::policies(styles)?;
    let mut lexical_fallbacks = vec![false; projection.atoms.len()];
    let mut grapheme_cursor = 0;
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
            boundaries[atom] = boundaries[atom].max(Boundary::Word);
            if previous < atom && kind.is_word_like() {
                word_ranges.push(previous..atom);
            }
            previous = atom;
        }
        let missing_lexical = line_break::apply(
            projection,
            styles,
            line_break::LineRules {
                policies: &line_policies,
                units: &line_units,
            },
            &offsets,
            component_start..component_end,
            &mut boundaries,
            &mut line_candidates,
        )?;
        let mut grapheme_end = grapheme_cursor;
        while grapheme_end < graphemes.len() && graphemes[grapheme_end].end <= component_end {
            grapheme_end += 1;
        }
        lexical_fallback::propose(
            missing_lexical,
            projection,
            styles,
            line_units.ranges(&graphemes[grapheme_cursor..grapheme_end]),
            &mut lexical_fallbacks,
        );
        grapheme_cursor = grapheme_end;
    }
    for (atom, fallback) in lexical_fallbacks.iter().copied().enumerate() {
        if fallback {
            issue_candidate(
                &mut line_candidates,
                LineCandidate {
                    atom,
                    style: style_at(projection, styles, atom),
                    cause: LineCause::LexicalFallback,
                },
            )?;
        }
    }
    line_candidates.sort_unstable_by_key(|row| (row.atom, row.style, row.cause));
    line_candidates.dedup();
    let phrases = phrase::analyze(projection, styles, boxes, paragraph, &graphemes)?;
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
            // The native presentation profile overrides eligible Cc. Every
            // other scalar and isolated unit retains its original ICU class,
            // independently of the glyph and segmentation projections.
            bidi_class: props.bidi_class(),
            bracket: sources.brackets().get32(atom.code_point),
            control: props.general_category() == GeneralCategory::Control,
            emoji: props.is_emoji_or_pictograph(),
            contributes: contributes_to_shaping(props.general_category(), script),
            normalize: (gcb == GraphemeClusterBreak::Extend
                && atom.code_point != 0x200c
                && !props.is_variation_selector())
                || gcb == GraphemeClusterBreak::SpacingMark,
            lexical_fallback_before: lexical_fallbacks[index],
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
    let direction =
        direction::DirectionBody::new(original.iter().map(|(_, (class, _))| *class), owners)?;
    // Raw Standalone retains one source-wide automatic base. Captured native
    // input supplies its fixed base; neither policy changes across a body B.
    let base_level = paragraph.base_direction.map_or_else(
        || BidiResolver::default_level(original.iter().map(|(_, (class, _))| *class)),
        |direction| direction.level(),
    );
    let mut levels = vec![base_level; infos.len()];
    let mut box_levels = vec![base_level; boxes.len()];
    let mut control_levels = vec![base_level; paragraph.controls.len()];
    let mut computed_levels = vec![base_level; paragraph.computed_boundaries.len()];
    let mut bidi = BidiResolver::new();
    let mut owner_start = 0;
    for body in original.split_inclusive(|(_, (class, _))| *class == BidiClass::ParagraphSeparator)
    {
        // P1 makes B part of the preceding paragraph. All contextual UBA work,
        // including neutral runs and isolate sequences, stays in this body.
        bidi.resolve(body.iter().copied(), Some(base_level));
        let owner_end = owner_start + body.len();
        for (owner, &level) in direction.owners()[owner_start..owner_end]
            .iter()
            .copied()
            .zip(bidi.levels())
        {
            match owner {
                BidiOwner::Atom(index) => levels[index] = level,
                BidiOwner::Atomic(index) => box_levels[index] = level,
                BidiOwner::Control(index) => control_levels[index] = level,
                BidiOwner::ComputedLineBoundary(index) => computed_levels[index] = level,
            }
        }
        owner_start = owner_end;
    }
    for (anchor, &at) in paragraph.anchor_order.iter().zip(&positions) {
        let inherited = match anchor.affinity {
            crate::Affinity::Before => at.checked_sub(1).and_then(|i| levels.get(i)),
            crate::Affinity::After => levels.get(at),
        }
        .copied()
        .unwrap_or(base_level);
        match anchor.item {
            SemanticAnchorItem::Atomic(i) if boxes[i].kind() == InlineBoxKind::OutOfFlow => {
                box_levels[i] = inherited
            }
            SemanticAnchorItem::Control(i)
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
            computed_levels,
            base_level,
            graphemes,
            line_units,
            line_candidates,
            phrases,
        },
        word_ranges,
        direction,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BidiOwner {
    Atom(usize),
    Atomic(usize),
    Control(usize),
    ComputedLineBoundary(usize),
}

pub(crate) fn anchor_position(
    item: SemanticAnchorItem,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
) -> usize {
    match item {
        SemanticAnchorItem::EmptyOrigin(index) => paragraph.origins[index].aggregate.start,
        SemanticAnchorItem::Atomic(index) => boxes[index].index(),
        SemanticAnchorItem::Control(index) => paragraph.controls[index].at,
        SemanticAnchorItem::ComputedLineBoundary(index) => paragraph.computed_boundaries[index].at,
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
    let source = anchor_position(paragraph.anchor_order[occurrence].item, paragraph, boxes);
    let index = projection
        .atoms
        .partition_point(|atom| atom.source.end <= source);
    project_anchor_atom(occurrence, paragraph, boxes, projection, index)
}

/// Projects an authentic semantic component cut against the actual rebuilt
/// typographic units. It does not reorder capture events or semantic barriers.
pub(crate) fn project_anchor_atom(
    occurrence: usize,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
    projection: &SourceProjection,
    mut index: usize,
) -> usize {
    let anchor = paragraph.anchor_order[occurrence];
    let source = anchor_position(anchor.item, paragraph, boxes);
    if projection
        .atoms
        .get(index)
        .is_some_and(|atom| atom.source.start < source)
        && anchor.affinity == crate::Affinity::After
    {
        index += 1;
    }
    if matches!(anchor.item, SemanticAnchorItem::Atomic(i) if boxes[i].kind() == InlineBoxKind::InFlow)
    {
        return index;
    }
    // The raw standalone Source front historically orders transparent boxes at
    // its admitted scalar endpoint, even inside a CRLF/combining grapheme.
    // Explicit native SemanticAnchor affinity still projects typography stops.
    if paragraph.base_direction.is_none()
        && matches!(anchor.item,SemanticAnchorItem::Atomic(i) if boxes[i].kind()==InlineBoxKind::OutOfFlow)
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

fn anchor_kind(item: SemanticAnchorItem) -> (crate::protocol::ProtocolHandleKind, usize) {
    match item {
        SemanticAnchorItem::EmptyOrigin(i) => (crate::protocol::ProtocolHandleKind::Origin, i),
        SemanticAnchorItem::Atomic(i) => (crate::protocol::ProtocolHandleKind::Atomic, i),
        SemanticAnchorItem::Control(i) => (crate::protocol::ProtocolHandleKind::Control, i),
        SemanticAnchorItem::ComputedLineBoundary(i) => {
            (crate::protocol::ProtocolHandleKind::ComputedLineBoundary, i)
        }
    }
}

fn push_bidi_anchor(
    item: SemanticAnchorItem,
    paragraph: &ParagraphData,
    boxes: &[InlineBox],
    sources: &AnalysisDataSources,
    original: &mut Vec<(char, (BidiClass, BidiMirroringGlyph))>,
    owners: &mut Vec<BidiOwner>,
) {
    let (cp, owner) = match item {
        SemanticAnchorItem::Atomic(index) if boxes[index].kind() == InlineBoxKind::InFlow => {
            (0xfffc, BidiOwner::Atomic(index))
        }
        SemanticAnchorItem::ComputedLineBoundary(index) => {
            (0xa, BidiOwner::ComputedLineBoundary(index))
        }
        SemanticAnchorItem::Control(index) => {
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

    fn bidi_analysis(text: &str, base: Option<crate::ParagraphBaseDirection>) -> Analysis {
        let source = crate::Source::new(text);
        let projection = SourceProjection::new(&source).expect("actual source projection");
        let styles = [AnalyzedStyleRun {
            owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                0,
                source.text().len(),
            )),
            style: crate::ValidatedStyle::try_from(crate::Style::default()).expect("default style"),
        }];
        let mut paragraph = ParagraphData::standalone(&source);
        paragraph.base_direction = base;
        analyze(&projection, &styles, &[], &paragraph)
            .expect("actual bidi analysis")
            .0
    }

    #[test]
    fn every_b_terminated_body_keeps_its_own_neutral_context_and_owner_levels() {
        use crate::ParagraphBaseDirection::RightToLeft;
        for (text, expected) in [
            ("", &[][..]),
            ("\n", &[1][..]),
            ("\n\n", &[1, 1][..]),
            ("A\n.A", &[2, 1, 1, 2][..]),
            ("\nA\n\n.A\n", &[1, 2, 1, 1, 1, 2, 1][..]),
        ] {
            let analysis = bidi_analysis(text, Some(RightToLeft));
            assert_eq!(analysis.base_level, 1, "{text:?}");
            assert_eq!(analysis.levels, expected, "{text:?}");
        }
    }

    #[test]
    fn standalone_automatic_base_stays_source_wide_across_bodies() {
        for (text, expected_base) in [
            ("A\n.א", 0),
            ("א\n.A", 1),
            ("123\nא", 1),
            // Preserve the raw front's existing whole-source isolate scan;
            // this operation does not introduce per-paragraph automatic bases.
            ("\u{2067}A\nא", 0),
        ] {
            let analysis = bidi_analysis(text, None);
            assert_eq!(analysis.base_level, expected_base, "{text:?}");
            assert_eq!(analysis.levels.len(), text.chars().count(), "{text:?}");
        }
        let analyzed =
            crate::AnalyzedSource::new(crate::Source::new("א\n.A"), crate::Style::default())
                .expect("raw public analysis");
        assert_eq!(analyzed.direction(), crate::Direction::RightToLeft);
        assert_eq!(analyzed.inner.analysis.levels, [1, 1, 1, 2]);
    }

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
        for (cp, expected) in [(0xd800, 0), (0xfffd, 1)] {
            let source = crate::Source::new(crate::Text::from_code_units(vec![cp, 0x05d0]));
            let analyzed = crate::AnalyzedSource::new(source, crate::Style::default())
                .expect("original-property analysis");
            assert_eq!(analyzed.inner.analysis.base_level, expected);
            assert_eq!(analyzed.inner.analysis.levels.len(), 2);
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
        let (analysis, _, direction) = analyze(
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
        let events = (0..projection.atoms.len())
            .map(|atom| wrapping::LogicalEvent {
                atom,
                source: projection.atoms[atom].source.start,
                kind: wrapping::Event::Text(atom),
            })
            .collect::<Vec<_>>();
        let direction = direction.bind(&events).unwrap();
        assert_eq!(direction.query(0, 2), Some((Some(0), true)));
        assert_eq!(direction.query(3, 4), Some((Some(0), true)));
    }
}
