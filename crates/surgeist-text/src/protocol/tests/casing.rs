//! Actual transformed native lines retain original custody across output cuts.
use super::*;
use crate::{CaseTransform, LineHeight, Range, TextTransform};

fn capture(text: &str, case: CaseTransform, locale: Option<&str>, font: Vec<u8>) -> PreparedText {
    capture_source(Source::new(text), case_style(case, locale), font)
}

fn case_style(case: CaseTransform, locale: Option<&str>) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        overflow_wrap: crate::OverflowWrap::Anywhere,
        locale: locale.map(str::to_owned),
        text_transform: TextTransform::Set(crate::TextTransformSet::none().with_case(case)),
        ..Style::default()
    }
}

fn capture_source(source: Source, base_style: Style, font: Vec<u8>) -> PreparedText {
    let len = source.text().len();
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font).unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn split(p: &PreparedText, at: usize) -> (FinalizedLine, FinalizedLine) {
    let mut first = request(p);
    let PreparedTermination::Ordinary(ceiling) = first.termination.clone() else {
        unreachable!()
    };
    let offers = p
        .conditional_search(
            &first.start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            None,
        )
        .and_then(|search| search.begin(&[]))
        .map(|batch| batch.opportunities().to_vec())
        .unwrap();
    let offer = offers
        .into_iter()
        .find(|offer| offer.boundary().view().participant_offset() == at)
        .expect("actual legal output EGC cut");
    first.end = offer.boundary();
    first.termination = PreparedTermination::Conditional(offer);
    first.participants.truncate(at);
    first.context.end = LineEnd::Soft;
    first.context.available_inline = Some(10.0);
    first.context.spacing = SpacingPolicy::None;
    let first = p
        .select(first)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let mut second = request(p);
    second.start = first.selected_line().request().end.clone();
    second.participants = p.participants().skip(at).collect();
    second.context.start = LineStart::Continuation;
    second.context.available_inline = Some(10.0);
    second.context.spacing = SpacingPolicy::None;
    let second = p
        .select(second)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    (first, second)
}

fn layout(p: PreparedText, lines: Vec<FinalizedLine>, range: bool) -> crate::Layout {
    let coverage = if range {
        PlacementCoverage::Range {
            start: lines
                .first()
                .unwrap()
                .selected_line()
                .request()
                .start
                .clone(),
            end: lines.last().unwrap().selected_line().request().end.clone(),
        }
    } else {
        PlacementCoverage::WholeParagraph
    };
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + index as f64 * 10.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}

fn ids(line: &FinalizedLine) -> Vec<u32> {
    line.run_views()
        .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
        .collect()
}

#[test]
fn authored_forced_origin_seam_stops_lithuanian_dot_removal_and_unit_context() {
    use crate::{
        Affinity, ContextualSourceInput, DocumentSegmentBreaks, ParagraphBoundaryDefinition,
        ParagraphBoundaryPolicy, ParagraphConnectionKind, ParagraphControl, ParagraphControlKind,
        ParagraphOrigin, ParagraphOriginKind, SourceProcessingConnection, SourceProcessingEnvelope,
        SourceProcessingEvent as E, SourceProcessingParagraph, SourceProcessingScope,
        SourceProcessingTopology, WhiteSpaceTrim,
    };
    use std::sync::Arc;

    // Unicode17 SpecialCasing removes Lithuanian dot-above after soft-dotted
    // text in upper/title context. A genuine authored hard boundary ends that
    // context; the receiving standalone mark is still original source.
    for forced in [false, true] {
        let source = Source::new("i\u{307}");
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
        ];
        if forced {
            events.push(E::Control {
                control: 0,
                affinity: Affinity::Before,
            });
        }
        events.extend([
            E::Connection(0),
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::ExitParagraph(0),
            E::ExitScope(0),
        ]);
        let processed = Arc::new(
            crate::process_contextual_source(ContextualSourceInput {
                scope_alignment_limits: None,
                computed_line_boundaries: vec![],
                source: source.clone(),
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                envelope: SourceProcessingEnvelope::Complete,
                origins: [Range::new(0, 1), Range::new(1, 2)]
                    .into_iter()
                    .map(|range| ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: ParagraphConnectionKind::Continuous,
                }],
                controls: if forced {
                    vec![ParagraphControl {
                        at: 1,
                        kind: ParagraphControlKind::ForcedBreak,
                    }]
                } else {
                    Vec::new()
                },
                boundary_policies: vec![ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                }],
                replacements: Vec::new(),
                scopes: vec![SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: SourceProcessingTopology::Complete,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    trim: WhiteSpaceTrim::default(),
                    base_style: case_style(CaseTransform::Uppercase, Some("lt")),
                }],
                paragraphs: vec![SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        assert_eq!(processed.processed_text().code_units(), [0x69, 0x307]);
        let analyzed = AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.source().text().code_units(), [0x69, 0x307]);
        assert_eq!(
            analyzed.inner.projection.text,
            if forced { "I\u{307}" } else { "I" }
        );
        let correspondence = (0..analyzed.inner.transform.rows.len())
            .flat_map(|row| analyzed.inner.transform.original_pieces(row))
            .map(|piece| (piece.origin, piece.aggregate, piece.local))
            .collect::<Vec<_>>();
        assert_eq!(
            correspondence,
            [
                (0, Range::new(0, 1), Range::new(0, 1)),
                (1, Range::new(1, 2), Range::new(1, 2)),
            ]
        );
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(crate::shaping_test_fonts::casing_ahem())
            .unwrap();
        let prepared = system
            .prepare(
                &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
                PreparationOptions::default(),
            )
            .unwrap();
        assert_eq!(prepared.unit_views().count(), if forced { 2 } else { 1 });
        if forced {
            assert_eq!(
                prepared
                    .unit_views()
                    .map(|unit| unit.source_range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
            let control = prepared.control_views().next().unwrap();
            assert_eq!(control.kind(), &ParagraphControlKind::ForcedBreak);
            assert_eq!(control.source_position().get(), 1);
            assert!(prepared.control_views().nth(1).is_none());
        } else {
            assert!(prepared.control_views().next().is_none());
        }
        let mut start = prepared.boundary_views().next().unwrap().handle();
        let mut lines = Vec::new();
        let endings = if forced {
            vec![PreparedBreakKind::Forced, PreparedBreakKind::ParagraphEnd]
        } else {
            vec![PreparedBreakKind::ParagraphEnd]
        };
        for kind in endings {
            let offer = prepared
                .opportunity_views()
                .find(|offer| offer.kind() == kind)
                .unwrap()
                .handle();
            let mut selected = request(&prepared);
            selected.start = start;
            selected.end = offer.view().boundary();
            let from = selected.start.view().participant_offset();
            let to = selected.end.view().participant_offset();
            selected.participants = prepared.participants().skip(from).take(to - from).collect();
            selected.controls = selected
                .participants
                .iter()
                .filter_map(|participant| {
                    if let PreparedParticipant::Control(control) = participant {
                        Some(ControlAllocation {
                            occurrence: control.clone(),
                            strut: None,
                            boundary: None,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            selected.termination = PreparedTermination::Ordinary(offer);
            selected.context.start = if lines.is_empty() {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            };
            selected.context.end = if kind == PreparedBreakKind::Forced {
                LineEnd::Forced
            } else {
                LineEnd::Paragraph
            };
            let line = prepared
                .select(selected)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            start = line.selected_line().request().end.clone();
            lines.push(line);
        }
        let placed = layout(prepared, lines, false);
        assert_eq!(placed.lines().len(), if forced { 2 } else { 1 });
        assert_eq!(placed.source().text().code_units(), [0x69, 0x307]);
    }
}

#[test]
fn native_expansion_uses_two_real_output_participants_and_receiving_only_source_geometry() {
    // Actual unchanged Ahem maps ß to134, S to53; each advances10px.
    let p = capture(
        "ß",
        CaseTransform::Uppercase,
        None,
        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
    );
    assert_eq!(p.unit_views().count(), 2);
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        &[0xdf]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let (first, second) = split(&p, 1);
    assert_eq!((ids(&first), ids(&second)), (vec![53], vec![53]));
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    let receiver = layout(p.clone(), vec![second.clone()], true);
    let fragment = receiver
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(fragment.units().len(), 1);
    assert_eq!(
        fragment
            .units()
            .next()
            .unwrap()
            .pieces()
            .next()
            .unwrap()
            .rect()
            .size
            .width,
        10.0
    );
    let whole = layout(p, vec![first, second], false);
    let fragments = whole
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(fragments.units().len(), 2);
    assert_eq!(whole.source().text().code_units(), &[0xdf]);
    let positions: Vec<_> = whole.positions().collect();
    let selected = whole
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selected.source_text().code_units(), &[0xdf]);
}

#[test]
fn delayed_iota_receiving_range_retains_only_its_original_component_without_host_caret_fabrication()
{
    let p = capture(
        "\u{1f80}\u{301}",
        CaseTransform::Uppercase,
        None,
        crate::shaping_test_fonts::casing_ahem(),
    );
    assert_eq!(p.analyzed_source().inner.host_projection.graphemes, [0, 2]);
    assert_eq!(
        p.analyzed_source().inner.projection.text,
        "\u{1f08}\u{301}Ι"
    );
    let origin = p.origin_views().next().unwrap().handle();
    let (first, second) = split(&p, 2);
    assert_eq!((ids(&first), ids(&second)), (vec![189], vec![43]));
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    let receiver = layout(p.clone(), vec![second.clone()], true);
    let fragments = receiver
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(fragments.units().len(), 1);
    assert_eq!(
        fragments
            .units()
            .next()
            .unwrap()
            .pieces()
            .next()
            .unwrap()
            .rect()
            .size
            .width,
        10.0
    );
    assert!(matches!(
        receiver
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap_err()
            .detail(),
        Some(crate::ErrorDetail::SourceRangeFragments {
            reason: crate::SourceRangeFragmentError::OutsidePlacement { .. }
        })
    ));
    let positions: Vec<_> = receiver.positions().collect();
    assert_eq!(
        receiver
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x1f80]
    );
    let whole = layout(p, vec![first, second], false);
    assert_eq!(
        whole
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap()
            .units()
            .len(),
        1
    );
    let positions: Vec<_> = whole.positions().collect();
    assert_eq!(
        whole
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x1f80, 0x301]
    );
}

#[test]
fn outgoing_wrap_owns_the_actual_removed_dot_and_receiver_starts_after_its_cut() {
    let p = capture(
        "I\u{307}B",
        CaseTransform::Lowercase,
        Some("tr"),
        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "ib");
    assert_eq!(p.unit_views().count(), 2);
    let origin = p.origin_views().next().unwrap().handle();
    let (first, second) = split(&p, 1);
    assert_eq!((ids(&first), ids(&second)), (vec![75], vec![68]));
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    assert_eq!(first.inner.selected.presentation.source, Range::new(0, 2));
    assert_eq!(second.inner.selected.presentation.source, Range::new(2, 3));
    let outgoing = layout(p.clone(), vec![first.clone()], true);
    assert!(
        outgoing
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap()
            .units()
            .len()
            == 0
    );
    let positions: Vec<_> = outgoing.positions().collect();
    assert_eq!(
        outgoing
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x49, 0x307]
    );
    let receiver = layout(p.clone(), vec![second.clone()], true);
    assert!(
        receiver
            .source_range_fragments(&origin, Range::new(1, 2))
            .is_err()
    );
    let whole = layout(p, vec![first, second], false);
    assert_eq!(whole.source().text().code_units(), &[0x49, 0x307, 0x42]);
}

#[test]
fn expansion_preserves_normal_alphabetic_no_break_and_distinct_conditional_permission() {
    for (word_break, overflow, ordinary, conditional) in [
        (
            crate::WordBreak::Normal,
            crate::OverflowWrap::Normal,
            false,
            false,
        ),
        (
            crate::WordBreak::BreakAll,
            crate::OverflowWrap::Normal,
            true,
            false,
        ),
        (
            crate::WordBreak::Normal,
            crate::OverflowWrap::Anywhere,
            false,
            true,
        ),
    ] {
        let p = capture_source(
            Source::new("ß"),
            Style {
                word_break,
                overflow_wrap: overflow,
                ..case_style(CaseTransform::Uppercase, None)
            },
            include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        );
        assert_eq!(p.analyzed_source().inner.projection.text, "SS");
        let middle = p
            .opportunity_views()
            .find(|offer| offer.boundary().view().participant_offset() == 1);
        assert_eq!(middle.is_some(), ordinary);
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = p.opportunity_views().next().unwrap().handle();
        let offers = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                None,
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap();
        assert_eq!(
            offers
                .iter()
                .any(|offer| offer.boundary().view().participant_offset() == 1),
            conditional
        );
    }
}

#[test]
fn combined_greek_reorder_and_removal_keep_original_order_across_real_wrapping() {
    let mut source = Source::new("\u{1f80}\u{301}\u{313}");
    source.span(
        Range::new(2, 3),
        case_style(CaseTransform::Uppercase, Some("el")),
    );
    let p = capture_source(
        source.clone(),
        case_style(CaseTransform::Uppercase, None),
        crate::shaping_test_fonts::casing_ahem(),
    );
    assert_eq!(
        p.analyzed_source().inner.projection.text,
        "\u{1f08}\u{301}Ι"
    );
    assert_eq!(p.analyzed_source().source(), &source);
    assert_eq!(
        p.unit_views().count(),
        3,
        "the removed breathing is no participant"
    );
    let origin = p.origin_views().next().unwrap().handle();
    let (first, second) = split(&p, 2);
    assert_eq!((ids(&first), ids(&second)), (vec![189], vec![43]));
    assert_eq!(first.inner.selected.presentation.source, Range::new(0, 3));
    assert_eq!(second.inner.selected.presentation.source, Range::new(0, 1));
    let outgoing = layout(p.clone(), vec![first.clone()], true);
    assert_eq!(
        outgoing
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap()
            .units()
            .len(),
        0
    );
    let positions: Vec<_> = outgoing.positions().collect();
    assert_eq!(
        outgoing
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text(),
        source.text()
    );
    let whole = layout(p, vec![first, second], false);
    let positions: Vec<_> = whole.positions().collect();
    assert_eq!(
        whole
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text(),
        source.text()
    );
    assert_eq!(
        whole
            .source_range_fragments(&origin, Range::new(0, 3))
            .unwrap()
            .units()
            .len(),
        2
    );
}

#[test]
fn actual_standalone_casing_and_cache_identity_preserve_each_operation() {
    let mut system = system();
    for (case, expected) in [
        (CaseTransform::None, vec![134]),
        (CaseTransform::Uppercase, vec![53, 53]),
        (CaseTransform::Capitalize, vec![53, 85]),
        (CaseTransform::Lowercase, vec![134]),
        (CaseTransform::None, vec![134]),
    ] {
        let style = case_style(case, None);
        let options = crate::Options {
            quantize: false,
            ..crate::Options::default()
        };
        let source = Source::new("ß");
        let warm = system
            .layout(source.clone(), style.clone(), options)
            .unwrap();
        let mut fresh = super::system();
        let cold = fresh.layout(source.clone(), style, options).unwrap();
        let glyphs = |layout: &crate::Layout| {
            layout
                .run_views()
                .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        assert_eq!(glyphs(&warm), expected);
        assert_eq!(glyphs(&cold), expected);
        assert_eq!(warm.source(), &source);
        assert_eq!(cold.source(), &source);
    }
    for phase in [
        crate::TextStylePhase::Standalone,
        crate::TextStylePhase::ProcessedParagraph,
    ] {
        for feature in [
            crate::TextStyleFeature::Uppercase,
            crate::TextStyleFeature::Lowercase,
            crate::TextStyleFeature::Capitalize,
        ] {
            assert!(crate::TextStyleFeature::ALL.contains(&feature));
            assert_eq!(feature.support(phase), crate::TextStyleSupport::Supported);
        }
    }
}

#[test]
fn canonical_greek_title_sequences_use_the_same_real_glyph_without_changing_source_forms() {
    for original in ["\u{1f80}", "α\u{313}\u{345}"] {
        let p = capture(
            original,
            CaseTransform::Capitalize,
            None,
            crate::shaping_test_fonts::casing_ahem(),
        );
        let line = p
            .select(request(&p))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        // The fixture's independent None controls qualify both canonical forms
        // at the title glyph192, with a genuine10px advance.
        assert_eq!(ids(&line), [192]);
        assert_eq!(line.feedback().advance, 10.0);
        let origin = p.origin_views().next().unwrap().handle();
        let placed = layout(p, vec![line], false);
        assert_eq!(placed.source().text(), &crate::Text::from(original));
        for unit in 0..original.encode_utf16().count() {
            assert_eq!(
                placed
                    .source_range_fragments(&origin, Range::new(unit, unit + 1))
                    .unwrap()
                    .units()
                    .len(),
                1
            );
        }
        let positions: Vec<_> = placed.positions().collect();
        assert_eq!(
            placed
                .position_selection(positions.first().unwrap(), positions.last().unwrap())
                .unwrap()
                .source_text(),
            &crate::Text::from(original)
        );
    }
}

#[test]
fn irish_inserted_hyphen_and_mapped_vowel_keep_the_vowel_row_as_their_real_emitter() {
    let p = capture(
        "nÓ",
        CaseTransform::Lowercase,
        Some("ga"),
        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "n-ó");
    assert_eq!(
        p.unit_views()
            .map(|unit| unit.source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(1, 2)]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 30.0);
    let placed = layout(p, vec![line], false);
    assert_eq!(
        placed
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap()
            .units()
            .len(),
        2
    );
    let positions: Vec<_> = placed.positions().collect();
    let start = positions
        .iter()
        .find(|position| placed.position_geometry(position).unwrap().rect().origin.x == 0.0)
        .unwrap();
    let after_n = positions
        .iter()
        .find(|position| placed.position_geometry(position).unwrap().rect().origin.x == 10.0)
        .unwrap();
    assert_eq!(
        placed
            .position_selection(start, after_n)
            .unwrap()
            .source_text()
            .code_units(),
        &[0x6e]
    );
    assert_eq!(
        placed
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x6e, 0xd3]
    );
}

#[test]
fn word_source_observation_is_input_ordered_unique_custody_not_repeated_output_spelling() {
    for (source, case, locale, expected, pieces) in [
        (
            "Straße",
            CaseTransform::Uppercase,
            None,
            "STRASSE",
            vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)],
        ),
        (
            "I\u{307}B",
            CaseTransform::Lowercase,
            Some("tr"),
            "ib",
            vec![(0, 1), (1, 2), (2, 3)],
        ),
    ] {
        let p = capture(
            source,
            case,
            locale,
            include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        );
        let word = p.word_views().next().unwrap();
        assert_eq!(word.query(), Some(expected));
        assert_eq!(
            word.source_pieces()
                .map(|piece| piece.source())
                .collect::<Vec<_>>(),
            pieces
                .into_iter()
                .map(|(start, end)| Range::new(start, end))
                .collect::<Vec<_>>()
        );
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn receiving_only_greek_accessibility_retains_its_actual_original_component_and_geometry() {
    let p = capture(
        "\u{1f80}\u{301}",
        CaseTransform::Uppercase,
        None,
        crate::shaping_test_fonts::casing_ahem(),
    );
    let (_, second) = split(&p, 2);
    let receiver = layout(p, vec![second], true);
    let mut state = crate::Accessibility::new(20);
    let parent = 1000_u64.into();
    let update = receiver
        .accessibility_update(&mut state, parent, parent, crate::Point::new(0.0, 0.0))
        .unwrap();
    let at = receiver
        .access_position(&state, crate::Cursor::new(0, crate::Affinity::After))
        .unwrap()
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == at.node)
        .unwrap()
        .1;
    assert_eq!(node.value(), Some("\u{1f80}"));
    assert_eq!(node.character_lengths(), [3]);
    let bounds = node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (0.0, 0.0, 10.0, 10.0)
    );
}

#[test]
fn actual_casing_after_phase_i_deletion_never_fuses_two_original_opaque_halves() {
    use skrifa::MetadataProvider;
    // The real Phase-I operation removes SPACE, making the two opaque source
    // halves adjacent in processed UTF16. They are still separate original
    // atoms when the casing emitter receives them, never U+10000.
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[58, 58],
        &[(0x20, 3), (0x58, 58), (0xfffd, 35)],
    );
    assert_eq!(
        skrifa::FontRef::new(&bytes)
            .unwrap()
            .charmap()
            .map(0xfffd_u32)
            .unwrap()
            .to_u32(),
        35
    );
    let source = Source::new(crate::Text::from_code_units(vec![0xd800, 0x20, 0xdc00]));
    let p = capture_source(
        source.clone(),
        Style {
            white_space: WhiteSpace::Discard,
            ..case_style(CaseTransform::Uppercase, None)
        },
        bytes,
    );
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        &[0xd800, 0xdc00]
    );
    assert_eq!(
        p.analyzed_source()
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [(0xd800, Range::new(0, 1)), (0xdc00, Range::new(2, 3))]
    );
    assert_eq!(
        p.analyzed_source().inner.projection.text,
        "\u{fffd}\u{fffd}"
    );
    assert!(p.analyzed_source().inner.transform.events().iter().any(|event| matches!(event, crate::transform::relation::RelationEvent::PhaseIDeleted { source, .. } if *source == Range::new(1, 2))));
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(ids(&line), [35, 35]);
    assert_eq!(line.feedback().advance, 20.0);
    let placed = layout(p, vec![line], false);
    assert_eq!(placed.source(), &source);
    let positions: Vec<_> = placed.positions().collect();
    assert_eq!(
        placed
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0xd800, 0x20, 0xdc00]
    );
}

#[test]
fn consecutive_greek_removals_retain_distinct_relation_cuts_and_exact_source() {
    // Greek uppercasing removes each applicable accent/breathing after alpha;
    // neither removed mark is a glyph, participant, or output-progress step.
    let p = capture(
        "α\u{301}\u{313}",
        CaseTransform::Uppercase,
        Some("el"),
        crate::shaping_test_fonts::casing_ahem(),
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "Α");
    assert_eq!(p.unit_views().count(), 1);
    assert!(matches!(
        p.analyzed_source().inner.transform.events(),
        [
            crate::transform::relation::RelationEvent::Text { atom: 0 },
            crate::transform::relation::RelationEvent::Removal { row: 1 },
            crate::transform::relation::RelationEvent::Removal { row: 2 },
        ]
    ));
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(ids(&line), vec![35]);
    let placed = layout(p, vec![line], false);
    let at = |cut| {
        placed
            .positions()
            .find(|position| position.source_cursor().index() == cut)
            .unwrap()
    };
    let before = at(1);
    let between = at(2);
    let after = at(3);
    assert_eq!(
        placed.position_geometry(&before).unwrap(),
        placed.position_geometry(&between).unwrap()
    );
    assert_eq!(
        placed.position_geometry(&between).unwrap(),
        placed.position_geometry(&after).unwrap()
    );
    for (a, b, expected) in [
        (&before, &between, vec![0x301]),
        (&between, &after, vec![0x313]),
        (&before, &after, vec![0x301, 0x313]),
        (&after, &before, vec![0x301, 0x313]),
    ] {
        let selected = placed.position_selection(a, b).unwrap();
        assert!(!selected.is_collapsed());
        assert_eq!(selected.source_text().code_units(), expected);
        assert!(selected.geometry().rects().is_empty());
    }
    let whole = placed.position_selection(&at(0), &after).unwrap();
    assert_eq!(whole.source_text().code_units(), &[0x3b1, 0x301, 0x313]);
    assert_eq!(whole.geometry().rects()[0].rect().size.width, 10.0);
    assert!(
        placed
            .position_selection(&between, &between)
            .unwrap()
            .is_collapsed()
    );
}

#[test]
fn outgoing_wrap_owns_both_consecutive_removals_at_the_actual_local_seam() {
    let p = capture(
        "α\u{301}\u{313}X",
        CaseTransform::Uppercase,
        Some("el"),
        crate::shaping_test_fonts::casing_ahem(),
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "ΑX");
    assert_eq!(p.unit_views().count(), 2);
    let origin = p.origin_views().next().unwrap().handle();
    let (first, second) = split(&p, 1);
    assert_eq!((ids(&first), ids(&second)), (vec![35], vec![58]));
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    assert_eq!(first.inner.selected.presentation.source, Range::new(0, 3));
    assert_eq!(second.inner.selected.presentation.source, Range::new(3, 4));
    assert_eq!(
        first.selected_line().request().end,
        second.selected_line().request().start
    );
    let outgoing = layout(p.clone(), vec![first.clone()], true);
    for (start, end) in [(1, 2), (2, 3), (1, 3)] {
        assert!(
            outgoing
                .source_range_fragments(&origin, Range::new(start, end))
                .unwrap()
                .units()
                .len()
                == 0
        );
    }
    let positions: Vec<_> = outgoing.positions().collect();
    let at = |index| {
        positions
            .iter()
            .find(|position| position.source_cursor().index() == index)
            .unwrap()
    };
    for (start, end, expected) in [
        (1, 2, vec![0x301]),
        (2, 3, vec![0x313]),
        (1, 3, vec![0x301, 0x313]),
    ] {
        let selected = outgoing.position_selection(at(start), at(end)).unwrap();
        assert_eq!(selected.source_text().code_units(), expected);
        assert!(!selected.is_collapsed());
        assert!(selected.geometry().rects().is_empty());
    }
    assert_eq!(
        outgoing
            .position_selection(at(0), at(3))
            .unwrap()
            .source_text()
            .code_units(),
        &[0x3b1, 0x301, 0x313]
    );
    let receiver = layout(p.clone(), vec![second.clone()], true);
    for range in [Range::new(1, 2), Range::new(2, 3), Range::new(1, 3)] {
        assert!(receiver.source_range_fragments(&origin, range).is_err());
    }
    let positions: Vec<_> = receiver.positions().collect();
    assert_eq!(
        receiver
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x58]
    );
    let whole = layout(p, vec![first, second], false);
    assert_eq!(whole.lines().len(), 2);
    let positions: Vec<_> = whole.positions().collect();
    assert_eq!(
        whole
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap()
            .source_text()
            .code_units(),
        &[0x3b1, 0x301, 0x313, 0x58]
    );
}
