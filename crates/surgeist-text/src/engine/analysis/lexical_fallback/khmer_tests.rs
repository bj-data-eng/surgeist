//! Independent source/line witnesses; editing remains on retained Unicode17 EGCs.
use super::super::{Boundary, LineCause, line_break};
use super::*;
use crate::{
    AnalyzedSource, Font, Id, LineHeight, PreparedBreakKind, Source, Style, Text, WordBreak,
};

fn style(word_break: WordBreak, line_break: crate::LineBreak, locale: Option<&str>) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: crate::WhiteSpace::Preserve,
        wrap: crate::Wrap::Word,
        hyphens: crate::Hyphens::None,
        word_break,
        line_break,
        locale: locale.map(str::to_owned),
        ..Style::default()
    }
}
fn source(units: &[u16]) -> Source {
    Source::identified(
        Text::from_code_units(units.to_vec()),
        Id::from_u64(150400),
        19,
    )
}
fn fallback_offsets(analyzed: &AnalyzedSource) -> Vec<usize> {
    analyzed
        .inner
        .analysis
        .infos
        .iter()
        .enumerate()
        .filter_map(|(at, (info, _))| info.lexical_fallback_before().then_some(at))
        .collect()
}
fn prepare(analyzed: &AnalyzedSource) -> crate::PreparedText {
    let mut system = crate::System::new(crate::SystemOptions {
        font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .to_vec(),
        )
        .unwrap();
    let prepared = system
        .prepare(analyzed, crate::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    prepared
}
fn ordinary_offsets(prepared: &crate::PreparedText) -> Vec<usize> {
    prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

#[test]
fn khmer_actual_missing_constructor_not_empty_detector_supplies_letter_pair_assumption() {
    for (units, expected) in [
        (&[0x41, 0x41][..], &[][..]),
        (&[0x1780, 0x1781][..], &[1][..]),
    ] {
        let original = source(units);
        let projection = SourceProjection::new(&original).unwrap();
        let styles = vec![AnalyzedStyleRun {
            owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                0,
                units.len(),
            )),
            style: crate::ValidatedStyle::try_from(style(
                WordBreak::Normal,
                crate::LineBreak::Auto,
                Some("km"),
            ))
            .unwrap(),
        }];
        let offsets = projection
            .segmentation
            .char_indices()
            .map(|(byte, _)| byte)
            .chain(std::iter::once(projection.segmentation.len()))
            .collect::<Vec<_>>();
        let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
        let mut candidates = Vec::new();
        // Consume the opaque witness RETURNED by the real provider owner.
        // Neither an empty cut inventory nor this test constructs absence.
        let missing = line_break::apply(
            &projection,
            &styles,
            &line_break::policies(&styles).unwrap(),
            &offsets,
            0..projection.atoms.len(),
            &mut boundaries,
            &mut candidates,
        )
        .unwrap();
        assert!(boundaries.iter().all(|boundary| *boundary < Boundary::Line));
        assert!(candidates.is_empty());
        let units = projection
            .graphemes
            .windows(2)
            .map(|pair| {
                projection
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[0])
                    ..projection
                        .atoms
                        .partition_point(|atom| atom.presentation.start < pair[1])
            })
            .collect::<Vec<_>>();
        let mut proposed = vec![false; projection.atoms.len()];
        propose(missing, &projection, &styles, &units, &mut proposed);
        assert_eq!(
            proposed
                .iter()
                .enumerate()
                .filter_map(|(at, &value)| value.then_some(at))
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn khmer_simple_complete_letter_units_gain_only_logical_fallback_for_known_or_unknown_locale() {
    for locale in [None, Some("km"), Some("und"), Some("en-US")] {
        for word_break in [WordBreak::Normal, WordBreak::KeepAll] {
            let analyzed = AnalyzedSource::new(
                source(&[0x1780, 0x1781]),
                style(word_break, crate::LineBreak::Auto, locale),
            )
            .unwrap();
            assert_eq!(analyzed.source().text().code_units(), [0x1780, 0x1781]);
            assert_eq!(analyzed.source().id(), Some(Id::from_u64(150400)));
            assert_eq!(analyzed.source().revision(), 19);
            assert_eq!(analyzed.style().locale.as_deref(), locale);
            assert_eq!(
                analyzed.graphemes().collect::<Vec<_>>(),
                [crate::Range::new(0, 1), crate::Range::new(1, 2)]
            );
            assert_eq!(fallback_offsets(&analyzed), [1]);
            assert_eq!(
                analyzed
                    .inner
                    .analysis
                    .line_candidates
                    .iter()
                    .map(|row| (row.atom, row.cause))
                    .collect::<Vec<_>>(),
                [(1, LineCause::LexicalFallback)]
            );
            // Fallback is a line cause, not a detected lexical word boundary.
            assert!(analyzed.inner.analysis.infos[1].0.boundary < Boundary::Line);
            assert_eq!(ordinary_offsets(&prepare(&analyzed)), [1]);
        }
    }
}

#[test]
fn khmer_manual_preserves_source_and_suppresses_only_missing_lexical_cause() {
    let analyzed = AnalyzedSource::new(
        source(&[0x1780, 0x1781]),
        style(WordBreak::Manual, crate::LineBreak::Auto, Some("km")),
    )
    .unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [crate::Range::new(0, 1), crate::Range::new(1, 2)]
    );
    assert!(fallback_offsets(&analyzed).is_empty());
    let prepared = prepare(&analyzed);
    assert!(ordinary_offsets(&prepared).is_empty());
    assert_eq!(
        prepared
            .opportunity_views()
            .map(|offer| (offer.boundary().view().source_after().get(), offer.kind()))
            .collect::<Vec<_>>(),
        [(2, PreparedBreakKind::ParagraphEnd)]
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [0x1780, 0x1781]
    );
}

fn post_matra(word_break: WordBreak, line_break: crate::LineBreak) {
    for (units, editing, expected) in [
        (
            &[0x1780, 0x17b6, 0x17d2, 0x1781][..],
            &[crate::Range::new(0, 3), crate::Range::new(3, 4)][..],
            &[][..],
        ),
        (
            &[0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782][..],
            &[
                crate::Range::new(0, 3),
                crate::Range::new(3, 4),
                crate::Range::new(4, 5),
            ][..],
            &[4][..],
        ),
    ] {
        let analyzed =
            AnalyzedSource::new(source(units), style(word_break, line_break, Some("km"))).unwrap();
        // Unicode17 editing EGCs remain unchanged. Adopted shared LINE units
        // merge the first two whole EGCs via actual COENG Linker before KHA.
        assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), editing);
        assert_eq!(analyzed.source().text().code_units(), units);
        assert!(fallback_offsets(&analyzed).is_empty());
        let prepared = prepare(&analyzed);
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            editing
        );
        assert_eq!(
            ordinary_offsets(&prepared),
            expected,
            "automatic opportunity must not split the adopted Khmer line unit at3; word={word_break:?}, line={line_break:?}"
        );
        assert_eq!(
            prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
                .map(|offer| offer.boundary().view().source_after().get())
                .collect::<Vec<_>>(),
            [units.len()]
        );
    }
}
#[test]
fn khmer_post_matra_break_all_blocks_internal_line_cut_without_changing_editing_egcs() {
    post_matra(WordBreak::BreakAll, crate::LineBreak::Auto);
}
#[test]
fn khmer_post_matra_anywhere_blocks_internal_line_cut_even_with_manual_lexical_policy() {
    post_matra(WordBreak::Manual, crate::LineBreak::Anywhere);
}
