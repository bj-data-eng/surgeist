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
        let policies = line_break::policies(&styles).unwrap();
        let line_units = super::super::line_units::LineBreakUnits::new(&projection, None).unwrap();
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
        let missing = line_break::apply(
            &projection,
            &styles,
            line_break::LineRules {
                policies: &policies,
                units: &line_units,
                graphemes: &units,
            },
            &offsets,
            0..projection.atoms.len(),
            &mut boundaries,
            &mut candidates,
        )
        .unwrap();
        assert!(boundaries.iter().all(|boundary| *boundary < Boundary::Line));
        assert!(candidates.is_empty());
        let mut proposed = vec![false; projection.atoms.len()];
        propose(
            missing,
            &projection,
            &styles,
            line_units.ranges(&units),
            &mut proposed,
        );
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
                crate::AnalysisTypography::HorizontalComposition,
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
        crate::AnalysisTypography::HorizontalComposition,
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
        let analyzed = AnalyzedSource::new(
            source(units),
            style(word_break, line_break, Some("km")),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
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

fn captured(
    original: Source,
    base_style: Style,
    ranges: &[crate::Range],
    controls: Vec<crate::ParagraphControl>,
    direction: crate::ParagraphBaseDirection,
    ancestor: crate::Wrap,
) -> AnalyzedSource {
    let mut anchors = controls
        .iter()
        .enumerate()
        .map(|(index, _)| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(index),
            affinity: crate::Affinity::Before,
        })
        .collect::<Vec<_>>();
    anchors.extend(
        original
            .boxes()
            .iter()
            .enumerate()
            .map(|(index, _)| crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::Atomic(index),
                affinity: crate::Affinity::Before,
            }),
    );
    anchors.extend(
        ranges
            .iter()
            .enumerate()
            .filter(|(_, range)| range.is_empty())
            .map(|(index, _)| crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(index),
                affinity: crate::Affinity::Before,
            }),
    );
    let connections = ranges
        .windows(2)
        .map(|pair| crate::ParagraphConnection {
            at: pair[0].end,
            kind: crate::ParagraphConnectionKind::Continuous,
        })
        .collect::<Vec<_>>();
    // Every source span in this fixture keeps base wrapping (the cross-style
    // case changes size only). Each actual continuous occurrence needs its
    // own captured wrapping row, including equal-offset empty-origin seams.
    let boundary_policies = (0..connections.len())
        .map(crate::ParagraphBoundaryDefinition::Connection)
        .chain(original.boxes().iter().enumerate().flat_map(|(index, _)| {
            [
                crate::ParagraphBoundaryDefinition::AtomicBefore(index),
                crate::ParagraphBoundaryDefinition::AtomicAfter(index),
            ]
        }))
        .map(|definition| crate::ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: base_style.wrap,
            following_wrap: base_style.wrap,
            ancestor_wrap: ancestor,
        })
        .collect();
    let input = crate::ParagraphInput {
        source: original.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: crate::ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: ranges
            .iter()
            .map(|&range| crate::ParagraphOrigin {
                source: original.clone(),
                local: range,
                aggregate: range,
                kind: crate::ParagraphOriginKind::Authored,
                text_node: None,
            })
            .collect(),
        connections,
        controls,
        boundary_policies,
        replacements: vec![],
        anchor_order: anchors,
    };
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        ancestor,
    )
}

#[test]
fn khmer_canonical_gb9c_stacks_and_all_attached_marks_keep_complete_fallback_units() {
    for (units, cut) in [
        (&[0x1780, 0x17d2, 0x1781, 0x1782][..], 3),
        (&[0x1780, 0x17d2, 0x1781, 0x17b7, 0x1782][..], 4),
        (
            &[0x1780, 0x17d2, 0x1781, 0x17d2, 0x1782, 0x17b7, 0x1783][..],
            6,
        ),
        (
            &[
                0x1780, 0x17d2, 0x1781, 0x17d2, 0x1782, 0x17b7, 0x17c6, 0x17b6, 0x1783,
            ][..],
            8,
        ),
    ] {
        // U17 GB9c joins canonical stacks; GB9/9a attach I/Nikahit/AA.
        // The AA here follows a complete stack, not an unresolved new grammar.
        let analyzed = AnalyzedSource::new(
            source(units),
            style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            analyzed.graphemes().collect::<Vec<_>>(),
            [
                crate::Range::new(0, cut),
                crate::Range::new(cut, units.len())
            ]
        );
        assert_eq!(fallback_offsets(&analyzed), [cut]);
        assert_eq!(ordinary_offsets(&prepare(&analyzed)), [cut]);
        assert_eq!(analyzed.source().text().code_units(), units);
    }
}

struct UnitProfile<'a> {
    source: &'a [u16],
    editing: &'a [crate::Range],
    outside: usize,
}
#[test]
fn khmer_post_matra_line_restriction_merges_whole_egcs_transitively_through_incb_extend() {
    for UnitProfile {
        source: units,
        editing,
        outside,
    } in [
        UnitProfile {
            source: &[0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782],
            editing: &[
                crate::Range::new(0, 3),
                crate::Range::new(3, 4),
                crate::Range::new(4, 5),
            ],
            outside: 4,
        },
        UnitProfile {
            source: &[0x1780, 0x17b6, 0x17d2, 0x17b7, 0x1781, 0x1782],
            editing: &[
                crate::Range::new(0, 4),
                crate::Range::new(4, 5),
                crate::Range::new(5, 6),
            ],
            outside: 5,
        },
        UnitProfile {
            source: &[
                0x1780, 0x17b6, 0x17d2, 0x1781, 0x17b6, 0x17d2, 0x1782, 0x17b7, 0x1783,
            ],
            editing: &[
                crate::Range::new(0, 3),
                crate::Range::new(3, 6),
                crate::Range::new(6, 8),
                crate::Range::new(8, 9),
            ],
            outside: 8,
        },
    ] {
        for word in [WordBreak::Normal, WordBreak::KeepAll] {
            let analyzed = AnalyzedSource::new(
                source(units),
                style(word, crate::LineBreak::Auto, Some("km")),
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), editing);
            assert_eq!(fallback_offsets(&analyzed), [outside]);
            assert_eq!(ordinary_offsets(&prepare(&analyzed)), [outside]);
        }
        let analyzed = AnalyzedSource::new(
            source(units),
            style(WordBreak::Manual, crate::LineBreak::Anywhere, Some("km")),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(fallback_offsets(&analyzed).is_empty());
        assert_eq!(ordinary_offsets(&prepare(&analyzed)), [outside]);
        assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), editing);
    }
}

#[test]
fn khmer_non_extend_interrupts_linker_suffix_and_leading_marks_gain_no_lexical_status() {
    // AA is SpacingMark but retained InCB=None. It cannot be skipped by the
    // adopted COENG + InCB=Extend* suffix, unlike I in the positive case.
    let units = [0x1780, 0x17b6, 0x17d2, 0x17b6, 0x1781];
    let analyzed = AnalyzedSource::new(
        source(&units),
        style(WordBreak::Manual, crate::LineBreak::Anywhere, Some("km")),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [crate::Range::new(0, 4), crate::Range::new(4, 5)]
    );
    assert_eq!(ordinary_offsets(&prepare(&analyzed)), [4]);
    for units in [
        &[0x17b7, 0x1780][..],
        &[0x17b7, 0x17c6][..],
        &[0x17d2, 0x1781, 0x1782][..],
        &[0x20, 0x17b7, 0x1780][..],
        &[0x200e][..],
    ] {
        let analyzed = AnalyzedSource::new(
            source(units),
            style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(fallback_offsets(&analyzed).is_empty(), "{units:x?}");
        assert_eq!(analyzed.source().text().code_units(), units);
    }
    // A leading COENG can merge with KHA for line indivisibility without
    // turning that mark-based unit into a lexical letter unit.
    let analyzed = AnalyzedSource::new(
        source(&[0x17d2, 0x1781, 0x1782]),
        style(WordBreak::BreakAll, crate::LineBreak::Auto, Some("km")),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [
            crate::Range::new(0, 1),
            crate::Range::new(1, 2),
            crate::Range::new(2, 3)
        ]
    );
    assert_eq!(ordinary_offsets(&prepare(&analyzed)), [2]);
}

#[test]
fn khmer_glue_and_zwj_keep_real_prohibitions_while_anywhere_keeps_its_own_authority() {
    let classes = CodePointMapData::<LineBreak>::new();
    let categories = CodePointMapData::<GeneralCategory>::new();
    let graphemes = CodePointMapData::<icu_properties::props::GraphemeClusterBreak>::new();
    let glue_mark = classes
        .iter_ranges_for_value(LineBreak::Glue)
        .flatten()
        .find(|cp| {
            categories.get32(*cp) == GeneralCategory::NonspacingMark
                && graphemes.get32(*cp) == icu_properties::props::GraphemeClusterBreak::Extend
        })
        .and_then(char::from_u32)
        .unwrap();
    let mut inherited = vec![0x1780];
    inherited.extend(glue_mark.encode_utf16(&mut [0; 2]).iter().copied());
    inherited.extend([0x17b7, 0x1781]);
    for units in [
        &[0x1780, 0x2060, 0x1781][..],
        &[0x1780, 0xa0, 0x1781][..],
        &[0x1780, 0x2011, 0x1781][..],
        &[0x1780, 0x200d, 0x1781][..],
        inherited.as_slice(),
    ] {
        // Total GL/WJ/ZWJ inventory belongs to the native captured profile;
        // raw standalone deliberately keeps a separate SPACE/NBSP policy.
        let analyzed = captured(
            source(units),
            style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
            &[crate::Range::new(0, units.len())],
            vec![],
            crate::ParagraphBaseDirection::LeftToRight,
            crate::Wrap::Word,
        );
        assert!(fallback_offsets(&analyzed).is_empty(), "{units:x?}");
        assert!(
            ordinary_offsets(&prepare(&analyzed)).is_empty(),
            "{units:x?}"
        );
    }
    let analyzed = captured(
        source(&[0x1780, 0xa0, 0x1781]),
        style(WordBreak::Manual, crate::LineBreak::Anywhere, Some("km")),
        &[crate::Range::new(0, 3)],
        vec![],
        crate::ParagraphBaseDirection::LeftToRight,
        crate::Wrap::Word,
    );
    assert_eq!(ordinary_offsets(&prepare(&analyzed)), [1, 2]);
    let analyzed = captured(
        source(&[0x1780, 0x1781, 0x2060, 0x41, 0x41]),
        style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
        &[crate::Range::new(0, 5)],
        vec![],
        crate::ParagraphBaseDirection::LeftToRight,
        crate::Wrap::Word,
    );
    assert_eq!(fallback_offsets(&analyzed), [1]);
    assert_eq!(ordinary_offsets(&prepare(&analyzed)), [1]);
    assert!(
        !analyzed
            .inner
            .analysis
            .line_candidates
            .iter()
            .any(|row| row.atom == 2)
    );
}

fn line_request(
    prepared: &crate::PreparedText,
    end: crate::PreparedBoundary,
    termination: crate::PreparedTermination,
    ending: crate::LineEnd,
    width: Option<f64>,
) -> crate::LineSelectionRequest {
    let start = prepared.boundary_views().next().unwrap().handle();
    let first = start.view().participant_offset();
    let last = end.view().participant_offset();
    let rtl = prepared.analyzed_source().direction() == crate::Direction::RightToLeft;
    crate::LineSelectionRequest {
        inline_boxes: None,
        segment: crate::test_wrapping::only_source_segment(prepared),
        start,
        end,
        termination,
        incoming: None,
        alternative: None,
        participants: prepared
            .participants()
            .skip(first)
            .take(last - first)
            .collect(),
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_box_struts: vec![],
        line_strut: None,
        context: crate::LineContext {
            formatted_line: crate::FormattedLineContext::default(),
            base_direction: if rtl {
                crate::ParagraphBaseDirection::RightToLeft
            } else {
                crate::ParagraphBaseDirection::LeftToRight
            },
            line_box_direction: if rtl {
                crate::LineBoxDirection::RightToLeft
            } else {
                crate::LineBoxDirection::LeftToRight
            },
            orientation: crate::LineOrientation::Horizontal,
            start: crate::LineStart::Paragraph,
            end: ending,
            available_inline: width,
            alignment_insets: crate::LineAlignmentInsets::ZERO,
            indent: 0.0,
            alignment: crate::Alignment::Start,
            tabs: crate::TabContext::NoTabs,
            spacing: crate::SpacingPolicy::None,
        },
    }
}
fn conditional_offsets(
    prepared: &crate::PreparedText,
    purpose: crate::ConditionalPurpose,
) -> Vec<usize> {
    let start = prepared.boundary_views().next().unwrap().handle();
    let ceiling = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    assert!(ordinary_offsets(prepared).is_empty());
    let frame = if purpose == crate::ConditionalPurpose::Overflow {
        let line = prepared
            .select(line_request(
                prepared,
                ceiling.view().boundary(),
                crate::PreparedTermination::Ordinary(ceiling.clone()),
                crate::LineEnd::Paragraph,
                Some(0.0),
            ))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: crate::SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        // Real full source/font finalization fails the finite zero capacity.
        // No nominal Khmer scalar width is assumed to manufacture exhaustion.
        assert!(line.feedback().advance > 0.0);
        Some(line)
    } else {
        None
    };
    let search = crate::test_wrapping::only_source_segment(prepared)
        .conditional_search(
            &start,
            None,
            &ceiling,
            purpose,
            &[],
            match frame.as_ref() {
                Some(frame) => crate::ConditionalLineContext::Finalized(frame),
                None => crate::ConditionalLineContext::Unframed { inline_boxes: None },
            },
        )
        .unwrap();
    let batch = search.begin(&[]).unwrap();
    assert_eq!(batch.stage(), crate::ConditionalStage::Arbitrary);
    let mut offsets = vec![];
    for offer in batch.opportunities() {
        assert_eq!(offer.prepared_text(), *prepared);
        assert_eq!(offer.start(), start);
        assert_eq!(offer.ceiling(), ceiling);
        assert_eq!(offer.purpose(), purpose);
        assert!(offer.alternative().is_none());
        let end = offer.boundary();
        offsets.push(end.view().source_after().get());
        let selected = prepared
            .select(line_request(
                prepared,
                end,
                crate::PreparedTermination::Conditional(offer.clone()),
                crate::LineEnd::Soft,
                if purpose == crate::ConditionalPurpose::MinContent {
                    None
                } else {
                    Some(0.0)
                },
            ))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: crate::SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(selected.prepared_text(), *prepared);
        assert!(selected.has_in_flow_content());
    }
    offsets
}
#[test]
fn khmer_actual_anywhere_and_break_word_conditional_admission_respects_shared_complete_unit() {
    for direction in [
        crate::ParagraphBaseDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft,
    ] {
        for overflow in [
            crate::OverflowWrap::Anywhere,
            crate::OverflowWrap::BreakWord,
        ] {
            let base = Style {
                overflow_wrap: overflow,
                ..style(WordBreak::Manual, crate::LineBreak::Auto, Some("km"))
            };
            let analyzed = captured(
                source(&[0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782]),
                base,
                &[crate::Range::new(0, 5)],
                vec![],
                direction,
                crate::Wrap::Word,
            );
            assert_eq!(
                analyzed.graphemes().collect::<Vec<_>>(),
                [
                    crate::Range::new(0, 3),
                    crate::Range::new(3, 4),
                    crate::Range::new(4, 5)
                ]
            );
            let prepared = prepare(&analyzed);
            assert_eq!(
                conditional_offsets(&prepared, crate::ConditionalPurpose::MinContent),
                if overflow == crate::OverflowWrap::Anywhere {
                    vec![4]
                } else {
                    vec![]
                }
            );
            assert_eq!(
                conditional_offsets(&prepared, crate::ConditionalPurpose::Overflow),
                [4]
            );
            assert_eq!(
                prepared.analyzed_source().source().text().code_units(),
                [0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782]
            );
        }
    }
}

#[test]
fn khmer_connected_cross_style_origin_and_empty_seams_do_not_split_the_line_unit() {
    for direction in [
        crate::ParagraphBaseDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft,
    ] {
        let base = style(WordBreak::BreakAll, crate::LineBreak::Auto, Some("km"));
        let mut original = source(&[0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782]);
        original.span(
            crate::Range::new(0, 3),
            Style {
                size: 12.0,
                ..base.clone()
            },
        );
        for ranges in [
            &[crate::Range::new(0, 5)][..],
            &[crate::Range::new(0, 3), crate::Range::new(3, 5)][..],
            &[
                crate::Range::new(0, 3),
                crate::Range::new(3, 3),
                crate::Range::new(3, 5),
            ][..],
        ] {
            let analyzed = captured(
                original.clone(),
                base.clone(),
                ranges,
                vec![],
                direction,
                crate::Wrap::Word,
            );
            assert_eq!(
                analyzed.graphemes().collect::<Vec<_>>(),
                [
                    crate::Range::new(0, 3),
                    crate::Range::new(3, 4),
                    crate::Range::new(4, 5)
                ]
            );
            let prepared = prepare(&analyzed);
            assert_eq!(ordinary_offsets(&prepared), [4]);
            assert_eq!(prepared.origin_views().count(), ranges.len());
            assert_eq!(
                prepared.analyzed_source().source().text().code_units(),
                original.text().code_units()
            );
            assert_eq!(
                prepared.analyzed_source().direction(),
                if direction == crate::ParagraphBaseDirection::LeftToRight {
                    crate::Direction::LeftToRight
                } else {
                    crate::Direction::RightToLeft
                }
            );
        }
    }
}

#[test]
fn khmer_raw_oof_at_protected_offset_cannot_donate_atomic_side_permission() {
    let units = [0x1780, 0x17b6, 0x17d2, 0x1781, 0x1782];
    for (word, line) in [
        (WordBreak::BreakAll, crate::LineBreak::Auto),
        (WordBreak::Manual, crate::LineBreak::Anywhere),
    ] {
        let mut original = source(&units);
        original.inline_box(crate::InlineBox::new(
            Id::from_u64(150409),
            crate::InlineBoxKind::OutOfFlow,
            3,
            crate::Size::new(0.0, 0.0),
        ));
        let analyzed = AnalyzedSource::new(
            original,
            style(word, line, Some("km")),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            analyzed.graphemes().collect::<Vec<_>>(),
            [
                crate::Range::new(0, 3),
                crate::Range::new(3, 4),
                crate::Range::new(4, 5)
            ]
        );
        let prepared = prepare(&analyzed);
        assert_eq!(ordinary_offsets(&prepared), [4]);
        let atomics = prepared.atomic_views().collect::<Vec<_>>();
        assert_eq!(atomics.len(), 1);
        assert_eq!(atomics[0].original_box().id(), Id::from_u64(150409));
        assert_eq!(
            atomics[0].original_box().kind(),
            crate::InlineBoxKind::OutOfFlow
        );
        assert_eq!(atomics[0].source_position().get(), 3);
        assert_eq!(
            prepared.analyzed_source().source().text().code_units(),
            units
        );
    }
    // A REAL in-flow atomic is a semantic barrier and independently owns its
    // ordinary None alternative. Preserve it even at zero size; no fake Text.
    let mut original = source(&units);
    original.inline_box(crate::InlineBox::new(
        Id::from_u64(150410),
        crate::InlineBoxKind::InFlow,
        3,
        crate::Size::new(0.0, 0.0),
    ));
    let analyzed = AnalyzedSource::new(
        original,
        style(WordBreak::Manual, crate::LineBreak::Auto, Some("km")),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let prepared = prepare(&analyzed);
    assert!(ordinary_offsets(&prepared).contains(&3));
    assert_eq!(
        prepared
            .atomic_views()
            .next()
            .unwrap()
            .original_box()
            .kind(),
        crate::InlineBoxKind::InFlow
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        units
    );
}

#[test]
fn khmer_explicit_controls_keep_actual_order_and_forced_breaks_survive_nowrap() {
    for direction in [
        crate::ParagraphBaseDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft,
    ] {
        let base = style(WordBreak::Manual, crate::LineBreak::Auto, Some("km"));
        let original = source(&[0x1780, 0x1781, 0x1782]);
        let analyzed = captured(
            original.clone(),
            base.clone(),
            &[crate::Range::new(0, 3)],
            vec![
                crate::ParagraphControl {
                    at: 1,
                    kind: crate::ParagraphControlKind::Wbr {
                        style: Box::new(base.clone()),
                    },
                },
                crate::ParagraphControl {
                    at: 2,
                    kind: crate::ParagraphControlKind::ForcedBreak,
                },
            ],
            direction,
            crate::Wrap::Word,
        );
        assert!(fallback_offsets(&analyzed).is_empty());
        let prepared = prepare(&analyzed);
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|offer| (offer.boundary().view().source_after().get(), offer.kind()))
                .collect::<Vec<_>>(),
            [
                (1, PreparedBreakKind::Allowed),
                (2, PreparedBreakKind::Forced),
                (3, PreparedBreakKind::ParagraphEnd)
            ]
        );
        let controls = prepared.control_views().collect::<Vec<_>>();
        assert_eq!(controls.len(), 2);
        assert!(matches!(
            controls[0].kind(),
            crate::ParagraphControlKind::Wbr { .. }
        ));
        assert_eq!(controls[0].source_position().get(), 1);
        assert_eq!(
            *controls[1].kind(),
            crate::ParagraphControlKind::ForcedBreak
        );
        assert_eq!(controls[1].source_position().get(), 2);
        assert_eq!(
            prepared.analyzed_source().source().text().code_units(),
            original.text().code_units()
        );

        let analyzed = captured(
            source(&[0x1780, 0x1781]),
            style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
            &[crate::Range::new(0, 2)],
            vec![],
            direction,
            crate::Wrap::None,
        );
        assert!(
            analyzed
                .wrapping_relations()
                .all(|relation| relation.ancestor_wrap() == Some(crate::Wrap::None))
        );
        assert!(ordinary_offsets(&prepare(&analyzed)).is_empty());
        let analyzed = captured(
            source(&[0x1780, 0xa, 0x1781]),
            style(WordBreak::Normal, crate::LineBreak::Auto, Some("km")),
            &[crate::Range::new(0, 3)],
            vec![],
            direction,
            crate::Wrap::None,
        );
        let prepared = prepare(&analyzed);
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|offer| (offer.boundary().view().source_after().get(), offer.kind()))
                .collect::<Vec<_>>(),
            [
                (2, PreparedBreakKind::Forced),
                (3, PreparedBreakKind::ParagraphEnd)
            ]
        );
        assert_eq!(
            prepared.analyzed_source().source().text().code_units(),
            [0x1780, 0xa, 0x1781]
        );

        // Each explicit occurrence at otherwise protected offset3 retains
        // its own authority; automatic suppression is not handle erasure.
        for (kind, expected) in [
            (
                crate::ParagraphControlKind::Wbr {
                    style: Box::new(base.clone()),
                },
                PreparedBreakKind::Allowed,
            ),
            (
                crate::ParagraphControlKind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                },
                PreparedBreakKind::Allowed,
            ),
            (
                crate::ParagraphControlKind::ForcedBreak,
                PreparedBreakKind::Forced,
            ),
        ] {
            let analyzed = captured(
                source(&[0x1780, 0x17b6, 0x17d2, 0x1781]),
                base.clone(),
                &[crate::Range::new(0, 4)],
                vec![crate::ParagraphControl { at: 3, kind }],
                direction,
                crate::Wrap::Word,
            );
            let prepared = prepare(&analyzed);
            let control = prepared.control_views().next().unwrap().handle();
            let offer = prepared
                .opportunity_views()
                .find(|offer| {
                    offer.kind() == expected && offer.boundary().view().source_after().get() == 3
                })
                .expect("actual explicit control must keep its own protected-offset opportunity");
            let prefix = offer.boundary().view().participant_offset();
            assert!(
                matches!(prepared.participants().nth(prefix - 1), Some(crate::PreparedParticipant::Control(actual)) if actual == control)
            );
        }
    }
}
