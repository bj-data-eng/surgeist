//! Actual computed box cuts and generated indications retain Mixed source roles.
use super::*;
use crate::{
    Affinity, ComputedLineBoundaryInput, ContextualSourceInput, Decoration, DecorationKind,
    DocumentSegmentBreaks, ErrorDetail, FinalizedFragmentOwner, HyphenationPlacement, Hyphens,
    NativeHyphenationConvention, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphConnectionKind, ParagraphOrigin, ParagraphOriginKind, SourceBoxSide,
    SourceProcessingConnection, SourceProcessingEnvelope, SourceProcessingEvent as E,
    SourceProcessingParagraph, SourceProcessingScope, SourceProcessingTopology, Text,
    TextStyleOwner, UprightDecorationRole, WhiteSpaceTrim, Wrap,
};
use std::sync::Arc;

fn own() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

// A real inline scope starts at `at`. Source processing alone issues the
// computed boundary; the tests never construct a cut, cause or participant.
fn captured(text: &str, at: usize, style: Style, force: bool) -> Arc<crate::ProcessedSource> {
    let source = Source::new(text);
    let ranges = [Range::new(0, at), Range::new(at, source.text().len())];
    let input = ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: ranges
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
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        controls: vec![],
        computed_line_boundaries: if force {
            vec![ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: SourceBoxSide::Before,
            }]
        } else {
            vec![]
        },
        replacements: vec![],
        scopes: vec![
            SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: style,
                used_direction: ParagraphBaseDirection::LeftToRight,
            },
            SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: WhiteSpaceTrim::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(1),
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    };
    Arc::new(crate::process_contextual_source(input).unwrap())
}
fn prepare(system: &mut System, processed: &Arc<crate::ProcessedSource>) -> PreparedText {
    let analyzed = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}
fn ending(
    p: &PreparedText,
    from: PreparedBoundary,
    offer: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut value = request(p);
    value.start = from;
    value.end = offer.view().boundary();
    let first = value.start.view().participant_offset();
    let last = value.end.view().participant_offset();
    value.participants = p.participants().skip(first).take(last - first).collect();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.context.orientation = LineOrientation::Vertical;
    value.context.base_direction = ParagraphBaseDirection::LeftToRight;
    value.context.line_box_direction = LineBoxDirection::LeftToRight;
    value.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = match offer.view().kind() {
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => LineEnd::Soft,
    };
    value.context.available_inline = None;
    value.context.alignment = crate::Alignment::Start;
    value.context.spacing = SpacingPolicy::None;
    value
}
fn terminal(p: &PreparedText) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn place(p: &PreparedText, lines: Vec<FinalizedLine>) -> crate::Result<crate::Layout> {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline: 100.0 - 20.0 * index as f64,
                    },
                    orientation: LineOrientation::Vertical,
                    atomics: vec![],
                })
            })
            .collect(),
    })
}
fn controls(bytes: &[u8]) {
    premises(bytes, true);
    assert_eq!(
        native(bytes, "X", false, surgeist_shaper::script::LATIN, &[]),
        [(274, 0, 10.0, 0.0, 0.0)]
    );
    assert_eq!(
        native(bytes, "一", true, surgeist_shaper::script::HAN, &[]),
        [(82, 0, 12.0, -6.0, 1.0)]
    );
}

#[test]
fn mixed_real_computed_scope_cut_retains_cause_and_each_selected_native_basis() {
    let bytes = mixed_fonts::fixture(true);
    controls(&bytes);
    let processed = captured("X一", 1, own(), true);
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(cause.source_position().get(), 1);
    assert_eq!(
        cause.target(),
        crate::CapturedComputedLineBoundaryTarget::Scope {
            scope: processed.captured_scope(1).unwrap(),
            side: SourceBoxSide::Before,
        }
    );
    let p = prepare(&mut system_with(&bytes), &processed);
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let forced = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap();
    let participant = forced.computed_line_boundary().unwrap();
    assert_eq!(participant.view().causes().collect::<Vec<_>>(), [cause]);
    assert_eq!(participant.view().source_position().get(), 1);
    assert_eq!(
        p.participants()
            .filter(|part| matches!(part,
        PreparedParticipant::ComputedLineBoundary(actual) if *actual == participant))
            .count(),
        1
    );
    let first = p
        .select(ending(
            &p,
            p.boundary_views().next().unwrap().handle(),
            forced.handle(),
        ))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let second = p
        .select(ending(&p, forced.boundary(), terminal(&p)))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        packets(&first, &bytes),
        [(
            Range::new(0, 1),
            GlyphBasis::Clockwise,
            274,
            0.0,
            0.0,
            0.0,
            10.0
        )]
    );
    assert_eq!(
        packets(&second, &bytes),
        [(
            Range::new(1, 2),
            GlyphBasis::Unrotated,
            82,
            0.0,
            -6.0,
            1.0,
            12.0
        )]
    );
    for line in [&first, &second] {
        let replay = line.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(replay.selected_line(), line.selected_line());
        assert_eq!(packets(&replay, &bytes), packets(line, &bytes));
    }
    let source = p.analyzed_source().source().clone();
    let layout = place(&p, vec![first, second]).unwrap();
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    copy_source(&layout, &source);
}

#[test]
fn mixed_real_before_after_both_indications_derive_roles_from_generated_source() {
    let bytes = mixed_fonts::fixture(true);
    controls(&bytes);
    for (text, basis, id, advance, x, y) in [
        ("X", GlyphBasis::Clockwise, 274, 10.0, 0.0, 0.0),
        ("一", GlyphBasis::Unrotated, 82, 12.0, -6.0, 1.0),
    ] {
        for mode in [
            HyphenationPlacement::LogicalBefore,
            HyphenationPlacement::LogicalAfter,
            HyphenationPlacement::BothSides,
        ] {
            let mut style = own();
            style.hyphens = Hyphens::Manual;
            style.locale = Some("und-Latn".into());
            style.native_hyphenation_convention = Some(
                NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    Text::from(text),
                    mode,
                )
                .unwrap(),
            );
            // Five ordinary letters, with two before/three after SHY, satisfy
            // the actual native 5/2/2 minima. Every host letter is VO=R.
            let processed = captured("XX\u{ad}XXX", 3, style, false);
            let p = prepare(&mut system_with(&bytes), &processed);
            let whole = p
                .select(ending(
                    &p,
                    p.boundary_views().next().unwrap().handle(),
                    terminal(&p),
                ))
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert_eq!(whole.feedback().advance, 50.0);
            assert_eq!(
                packets(&whole, &bytes),
                [0, 1, 3, 4, 5]
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, source)| (
                        Range::new(source, source + 1),
                        GlyphBasis::Clockwise,
                        274,
                        10.0 * ordinal as f64,
                        0.0,
                        0.0,
                        10.0
                    ))
                    .collect::<Vec<_>>()
            );
            assert!(
                !whole
                    .mapping_views()
                    .any(|m| m.kind() == crate::PresentationMappingKind::Replacement)
            );
            assert!(
                whole
                    .run_views()
                    .all(|run| run.glyph_basis() == GlyphBasis::Clockwise)
            );
            let offer = p
                .hyphenation_cut_views()
                .find(|cut| cut.boundary().view().source_before().get() == 3)
                .unwrap()
                .opportunity()
                .unwrap();
            let chosen = offer.view().alternatives().next().unwrap().handle();
            assert_eq!(chosen.view().source_range(), Some(Range::new(2, 3)));
            assert_eq!(chosen.view().insertion_anchor().get(), 3);
            assert_eq!(
                chosen.view().text().code_units(),
                text.encode_utf16().collect::<Vec<_>>()
            );
            let mut r = ending(
                &p,
                p.boundary_views().next().unwrap().handle(),
                offer.clone(),
            );
            r.alternative = Some(chosen.clone());
            let first = p
                .select(r)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            let before = mode != HyphenationPlacement::LogicalAfter;
            let after = mode != HyphenationPlacement::LogicalBefore;
            assert_eq!(first.hyphenation_continuation().is_some(), after);
            let mut r = ending(&p, offer.view().boundary(), terminal(&p));
            r.incoming = first.hyphenation_continuation();
            let second = p
                .select(r)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert!(second.hyphenation_continuation().is_none());
            let mut first_packets = vec![
                (
                    Range::new(0, 1),
                    GlyphBasis::Clockwise,
                    274,
                    0.0,
                    0.0,
                    0.0,
                    10.0,
                ),
                (
                    Range::new(1, 2),
                    GlyphBasis::Clockwise,
                    274,
                    10.0,
                    0.0,
                    0.0,
                    10.0,
                ),
            ];
            if before {
                first_packets.push((Range::new(3, 3), basis, id, 20.0, x, y, advance));
            }
            let mut second_packets = vec![];
            if after {
                second_packets.push((Range::new(3, 3), basis, id, 0.0, x, y, advance));
            }
            let added = if after { advance } else { 0.0 };
            for offset in 0..3 {
                second_packets.push((
                    Range::new(3 + offset, 4 + offset),
                    GlyphBasis::Clockwise,
                    274,
                    added + 10.0 * offset as f64,
                    0.0,
                    0.0,
                    10.0,
                ));
            }
            assert_eq!(packets(&first, &bytes), first_packets, "{text:?}/{mode:?}");
            assert_eq!(
                packets(&second, &bytes),
                second_packets,
                "{text:?}/{mode:?}"
            );
            assert_eq!(
                first.feedback().advance,
                20.0 + if before { advance } else { 0.0 }
            );
            assert_eq!(second.feedback().advance, 30.0 + added);
            for (line, emits) in [(&first, before), (&second, after)] {
                if emits {
                    assert!(
                        line.fragment_views()
                            .any(|f| f.owner()
                                == FinalizedFragmentOwner::Alternative(chosen.clone()))
                    );
                    assert!(
                        line.mapping_views()
                            .any(|m| m.alternative() == Some(chosen.clone()))
                    );
                }
                let replay = line.with_spacing(SpacingAllocations::None).unwrap();
                assert_eq!(replay.selected_line(), line.selected_line());
                assert_eq!(packets(&replay, &bytes), packets(line, &bytes));
            }
            let source = p.analyzed_source().source().clone();
            let layout = place(&p, vec![first, second]).unwrap();
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 3), Range::new(3, 6)]
            );
            copy_source(&layout, &source);
        }
    }
}

#[test]
fn mixed_real_computed_empty_cut_uses_its_selected_metric_only_decoration_owner() {
    let bytes = mixed_fonts::fixture(true);
    controls(&bytes);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('\u{2065}'),
        VerticalOrientation::Upright
    );
    // The actual U default-ignorable is residual, not a fabricated deleted
    // packet. Source projection retains it even though Text allocates no glyph.
    let face = surgeist_shaper::FontRef::new(&bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    input.add('\u{2065}', 91);
    input.set_direction(surgeist_shaper::Direction::TopToBottom);
    input.set_script(surgeist_shaper::script::LATIN);
    let native = shaper.shape(input, &[]);
    assert_eq!(
        native.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    assert_eq!(
        native
            .default_ignorable_receipt()
            .unwrap()
            .residual_input_positions(),
        [0]
    );
    assert!(
        native
            .default_ignorable_receipt()
            .unwrap()
            .deleted_input_positions()
            .is_empty()
    );
    let mut decorated = own();
    decorated.underline = Decoration::solid();
    // Same source, style, ordinary paragraph and catalogue: the uncut line has
    // an actual visible R body and ordinary horizontal auto decoration metrics.
    let clear = captured("\u{2065}X", 1, decorated.clone(), false);
    let mut system = system_with(&bytes);
    let p = prepare(&mut system, &clear);
    let line = p
        .select(ending(
            &p,
            p.boundary_views().next().unwrap().handle(),
            terminal(&p),
        ))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        packets(&line, &bytes),
        [(
            Range::new(1, 2),
            GlyphBasis::Clockwise,
            274,
            0.0,
            0.0,
            0.0,
            10.0
        )]
    );
    let ordinary = place(&p, vec![line]).unwrap();
    assert_eq!(ordinary.decorations().len(), 1);
    let forced = captured("\u{2065}X", 1, decorated, true);
    let cause = forced.captured_computed_line_boundary(0).unwrap();
    assert_eq!(cause.source_position().get(), 1);
    let p = prepare(&mut system, &forced);
    let offer = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap();
    assert_eq!(
        offer
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    // The captured paragraph still contains X. It cannot make this selected
    // all-residual prefix's DefaultFont U strut an unused paragraph style row.
    let error = p
        .select(ending(
            &p,
            p.boundary_views().next().unwrap().handle(),
            offer.handle(),
        ))
        .and_then(|selected| {
            selected.finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
        })
        .and_then(|line| {
            crate::Layout::from_finalized_lines(PlacementRequest {
                prepared: p.clone(),
                coverage: PlacementCoverage::Range {
                    start: line.selected_line().request().start.clone(),
                    end: line.selected_line().request().end.clone(),
                },
                entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline: 100.0,
                    },
                    orientation: LineOrientation::Vertical,
                    atomics: vec![],
                })],
            })
        })
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UprightDecoration {
            owner: TextStyleOwner::Paragraph,
            kind: DecorationKind::Underline,
            role: UprightDecorationRole::OffsetAndThickness,
        })
    );
}

#[path = "mixed_selections/indication_fragments.rs"]
mod indication_fragments;
