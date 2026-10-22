// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 analysis/mod.rs: original UTF-16 properties,
// separate segmentation input, usize indices, and occurrence-backed atomics.

pub(crate) mod cluster;

use std::ops::Range;

use icu_normalizer::properties::{
    CanonicalComposition, CanonicalCompositionBorrowed, CanonicalDecomposition,
    CanonicalDecompositionBorrowed,
};
use icu_properties::props::{
    BidiClass, BidiMirroringGlyph, GeneralCategory, GraphemeClusterBreak, Script,
};
use icu_properties::{
    CodePointMapData, CodePointMapDataBorrowed, PropertyNamesShort, PropertyNamesShortBorrowed,
};
use icu_segmenter::options::{LineBreakOptions, LineBreakWordOption, WordBreakInvariantOptions};
use icu_segmenter::{LineSegmenter, WordSegmenter};
use parley_data::Properties;

use super::{bidi::BidiResolver, resolve::AnalyzedStyleRun};
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

pub(crate) struct Analysis {
    pub(crate) infos: Vec<(CharInfo, usize)>,
    pub(crate) levels: Vec<u8>,
    pub(crate) box_levels: Vec<u8>,
    pub(crate) base_level: u8,
    /// Actual segmentation boundaries expressed as half-open atom intervals.
    pub(crate) graphemes: Vec<Range<usize>>,
}

pub(crate) fn analyze(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    boxes: &[InlineBox],
) -> Result<Analysis> {
    let sources = AnalysisDataSources;
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
                .partition_point(|atom| atom.source.start < *unit)
        })
        .collect();
    let graphemes = grapheme_stops
        .windows(2)
        .map(|pair| pair[0]..pair[1])
        .collect();
    let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
    for byte in WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default())
        .segment_str(&projection.segmentation)
    {
        boundaries[atom_at(byte)] = Boundary::Word;
    }
    // Keep complete neighboring scalars at word-break style boundaries, as in
    // the upstream analyzer. This preserves contextual line-break decisions.
    let mut segments: Vec<(crate::Range, WordBreak)> = Vec::new();
    for run in styles {
        let mode = run.style.authored().word_break;
        if let Some((range, previous)) = segments.last_mut()
            && *previous == mode
            && range.end == run.range.start
        {
            range.end = run.range.end;
            continue;
        }
        segments.push((run.range, mode));
    }
    for (range, mode) in segments {
        let first = projection
            .atoms
            .partition_point(|atom| atom.source.end <= range.start);
        let end = projection
            .atoms
            .partition_point(|atom| atom.source.start < range.end);
        if first == end {
            continue;
        }
        let begin_context = first.saturating_sub(1);
        let end_context =
            (end + usize::from(end < projection.atoms.len())).min(projection.atoms.len());
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
    }
    let mut infos = Vec::with_capacity(projection.atoms.len());
    let mut mandatory = false;
    for (index, atom) in projection.atoms.iter().enumerate() {
        let props = sources.original_properties(atom.code_point);
        let script = props.script();
        let gcb = props.grapheme_cluster_break();
        let style = styles
            .partition_point(|run| run.range.end <= atom.source.start)
            .min(styles.len().saturating_sub(1));
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
        mandatory = props.is_mandatory_linebreak();
        infos.push((info, style));
    }
    let mut box_order: Vec<_> = boxes
        .iter()
        .enumerate()
        .filter(|(_, item)| item.kind() == InlineBoxKind::InFlow)
        .collect();
    box_order.sort_by_key(|(ordinal, item)| (item.index(), *ordinal));
    let mut original = Vec::with_capacity(infos.len() + box_order.len());
    let mut owners = Vec::with_capacity(original.capacity());
    let mut next_box = box_order.into_iter().peekable();
    for (index, atom) in projection.atoms.iter().enumerate() {
        while next_box
            .peek()
            .is_some_and(|(_, item)| item.index() <= atom.source.start)
        {
            let (ordinal, _) = next_box.next().expect("peek established a box");
            original.push((
                '\u{fffc}',
                (BidiClass::OtherNeutral, sources.brackets().get('\u{fffc}')),
            ));
            owners.push((true, ordinal));
        }
        original.push((
            atom.glyph,
            (infos[index].0.bidi_class, infos[index].0.bracket),
        ));
        owners.push((false, index));
    }
    for (ordinal, _) in next_box {
        original.push((
            '\u{fffc}',
            (BidiClass::OtherNeutral, sources.brackets().get('\u{fffc}')),
        ));
        owners.push((true, ordinal));
    }
    let mut bidi = BidiResolver::new();
    bidi.resolve(original.into_iter(), None);
    let mut levels = vec![bidi.base_level(); infos.len()];
    let mut box_levels = vec![bidi.base_level(); boxes.len()];
    for ((is_box, index), &level) in owners.into_iter().zip(bidi.levels()) {
        if is_box {
            box_levels[index] = level;
        } else {
            levels[index] = level;
        }
    }
    Ok(Analysis {
        infos,
        levels,
        box_levels,
        base_level: bidi.base_level(),
        graphemes,
    })
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
            range: crate::Range::new(0, 4),
            style: crate::ValidatedStyle::try_from(crate::Style::default()).expect("default style"),
        }];
        let analysis = analyze(&projection, &styles, &[]).expect("original analysis");
        assert_eq!(analysis.base_level, 0);
        assert_eq!(analysis.infos[0].0.bidi_class, BidiClass::LeftToRight);
        assert_eq!(analysis.infos[2].0.bidi_class, BidiClass::LeftToRight);
        assert_eq!(analysis.graphemes, [0..2, 2..3, 3..4]);
        assert_eq!(projection.atoms[0].code_point, 0xd800);
        assert_eq!(projection.atoms[2].code_point, 0xdc00);
        assert_eq!(projection.atoms[0].glyph, '\u{fffd}');
    }
}
