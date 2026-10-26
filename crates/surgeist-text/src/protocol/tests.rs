//! Functional qualification of the new public phase front.

mod after_continuation;
mod after_standalone;
mod alignment_insets;
mod allocated_box_metrics;
mod auto_phrase;
mod automatic_model;
mod automatic_opportunities;
mod casing;
mod computed_boundaries;
mod conditional;
mod count_admission;
mod hyphenation_limits;
mod hyphenation_policy;
mod incidence_cost;
mod joining_context;
mod joining_positioning;
mod kana;
mod line_box_direction;
mod mandatory_breaks;
mod manual_hyphenation;
mod math;
mod metric_heights;
mod negative_tabs;
mod phase_two;
mod raw_automatic_hyphenation;
mod scope_incidence_locality;
mod separators;
mod tab_stops;
mod wbr;
mod whitespace_edges;
mod width;

use super::*;
use crate::{
    AnalyzedSource, Font, FontCatalogueMode, ParagraphStrut, PresentationMappingKind, Source,
    Style, System, SystemOptions, WhiteSpace,
};

fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    system
}
fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        ..Style::default()
    }
}
fn prepared(system: &mut System, text: &str, whitespace: WhiteSpace) -> PreparedText {
    let source = Source::new(text);
    let input = ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        paragraph_strut: ParagraphStrut::DefaultFont,
        base_style: Style {
            white_space: whitespace,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: crate::Range::new(0, text.encode_utf16().count()),
            aggregate: crate::Range::new(0, text.encode_utf16().count()),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if text.is_empty() {
            vec![paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::After,
            }]
        } else {
            vec![]
        },
    };
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn request(prepared: &PreparedText) -> LineSelectionRequest {
    let start = prepared.boundary_views().next().unwrap().handle();
    let termination = prepared
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    LineSelectionRequest {
        boundary_context: Vec::new(),
        incoming: None,
        start,
        end: termination.view().boundary(),
        termination: crate::PreparedTermination::Ordinary(termination),
        alternative: None,
        participants: prepared.participants().collect(),
        atomics: vec![],
        controls: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: LineContext {
            alignment_insets: crate::LineAlignmentInsets::ZERO,
            formatted_line: crate::FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: crate::LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Horizontal,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: Some(200.0),
            indent: 0.0,
            alignment: crate::Alignment::Left,
            tabs: TabContext::NoTabs,

            spacing: SpacingPolicy::Spaces,
        },
    }
}
fn place(
    prepared: PreparedText,
    line: FinalizedLine,
    x: f64,
    y: f64,
) -> crate::Result<crate::Layout> {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: x,
                baseline: y,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
}
fn reason(error: &crate::Error) -> &ProtocolError {
    match error.detail() {
        Some(crate::ErrorDetail::InvalidTextProtocol { reason }) => reason,
        _ => panic!("unexpected error {error:?}"),
    }
}
#[test]
fn prepared_selected_finalized_placed_share_actual_resources_and_fractional_anchor() {
    let mut system = system();
    let prepared = prepared(&mut system, "A A", WhiteSpace::Preserve);
    let stats = system.stats();
    let selected = prepared.select(request(&prepared)).unwrap();
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), 1);
    let line = selected
        .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: slots[0].clone(),
            amount: 0.25,
        }]))
        .unwrap();
    assert_eq!(line.feedback().advance, 60.25);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run.glyphs().map(|g| g.advance()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![20.0, 20.0, 20.0]
    );
    assert_eq!(
        line.run_views()
            .flat_map(|run| run.glyphs().map(|g| g.pen()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![0.0, 20.0, 40.25]
    );
    assert_eq!(line.selected_line(), selected);
    assert_eq!(system.stats(), stats);
    drop(system);
    let layout = place(prepared.clone(), line.clone(), 100.25, 200.5).unwrap();
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        crate::Point::new(100.25, 200.5)
    );
    assert_eq!(layout.source().text().code_units(), &[65, 32, 65]);
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(line.feedback().advance, 60.25);
}
#[test]
fn selection_refuses_foreign_and_incomplete_captures_and_spacing_custody() {
    let mut system = system();
    let a = prepared(&mut system, "A A", WhiteSpace::Preserve);
    let b = prepared(&mut system, "A A", WhiteSpace::Preserve);
    let mut foreign = request(&a);
    foreign.end = request(&b).end;
    assert!(matches!(
        reason(&a.select(foreign).unwrap_err()),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Boundary,
            ..
        }
    ));
    let mut missing = request(&a);
    missing.participants.pop();
    assert!(matches!(
        reason(&a.select(missing).unwrap_err()),
        ProtocolError::InvalidBoundaryRange { .. }
    ));
    let a_selected = a.select(request(&a)).unwrap();
    let b_selected = b.select(request(&b)).unwrap();
    assert!(matches!(
        reason(
            &a_selected
                .finalize(SpacingAllocations::Complete(vec![]))
                .unwrap_err()
        ),
        ProtocolError::MissingSpacingSlot { slot: 0 }
    ));
    let slot = b_selected.spacing_slots().next().unwrap();
    assert_eq!(
        reason(
            &a_selected
                .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                    slot,
                    amount: 1.0
                }]))
                .unwrap_err()
        ),
        &ProtocolError::ForeignSelectedLine
    );
    let slot = a_selected.spacing_slots().next().unwrap();
    assert!(matches!(
        reason(
            &a_selected
                .finalize(SpacingAllocations::Complete(vec![
                    SpacingAllocation {
                        slot: slot.clone(),
                        amount: 1.0
                    },
                    SpacingAllocation { slot, amount: 2.0 }
                ]))
                .unwrap_err()
        ),
        ProtocolError::DuplicateSpacingSlot { slot: 0 }
    ));
}
#[test]
fn explicit_placement_retains_exact_preparation_and_rejects_foreign_lines() {
    let mut system = system();
    let a = prepared(&mut system, "A", WhiteSpace::Preserve);
    let b = prepared(&mut system, "A", WhiteSpace::Preserve);
    let line = a
        .select(request(&a))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(matches!(
        reason(&place(b, line.clone(), 0.0, 16.0).unwrap_err()),
        ProtocolError::ForeignFinalizedLine { line: 0 }
    ));
    let first = place(a.clone(), line.clone(), 0.125, 16.25).unwrap();
    let second = place(a, line, 0.125, 16.25).unwrap();
    let position = first.positions().next().unwrap();
    assert!(matches!(
        reason(&second.position_geometry(&position).unwrap_err()),
        ProtocolError::ForeignPlacement
    ));
    assert!(first.clone().position_geometry(&position).is_ok());
}
#[test]
fn composed_whitespace_executes_correspondence_and_phase_catalogue_is_truthful() {
    let mut system = system();
    let p = prepared(&mut system, " A  B ", WhiteSpace::Collapse);
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 60.0);
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        &[32, 65, 32, 32, 66, 32]
    );
    assert!(
        line.mapping_views()
            .any(|mapping| mapping.kind() == PresentationMappingKind::Deleted)
    );
    for feature in [
        crate::TextStyleFeature::WhiteSpaceCollapse,
        crate::TextStyleFeature::WhiteSpaceDiscard,
        crate::TextStyleFeature::WhiteSpacePreserveSpaces,
    ] {
        assert_eq!(
            feature.support(crate::TextStylePhase::ProcessedParagraph),
            crate::TextStyleSupport::Supported
        );
        assert!(matches!(
            feature.support(crate::TextStylePhase::Standalone),
            crate::TextStyleSupport::Unsupported(_)
        ));
    }
}

#[test]
fn font_advance_tab_is_the_actual_retained_glyph_not_a_space_approximation() {
    let mut system = system();
    let p = prepared(&mut system, "A\tB", WhiteSpace::Preserve);
    assert!(matches!(
        reason(&p.select(request(&p)).unwrap_err()),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::TabStops
        }
    ));
    let mut native = request(&p);
    native.context.tabs = TabContext::FontAdvance;
    let line = p
        .select(native)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 60.0);
    let glyphs: Vec<_> = line
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| (g.id(), g.advance(), g.pen()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        glyphs,
        vec![(35, 20.0, 0.0), (0, 20.0, 20.0), (36, 20.0, 40.0)]
    );
    let tab = prepared(&mut system, "\t", WhiteSpace::Preserve);
    let mut native = request(&tab);
    native.context.tabs = TabContext::FontAdvance;
    let line = tab
        .select(native)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.feedback().trailing_whitespace, 20.0);
}

#[test]
fn native_capture_requires_its_complete_line_front_before_any_cache_publication() {
    let mut system = system();
    let p = prepared(&mut system, "A", WhiteSpace::Preserve);
    let before = system.stats();
    let error = system
        .layout_analyzed(&p.analyzed_source(), crate::Options::default())
        .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::UnsupportedStandaloneAnalysis
    );
    assert_eq!(system.stats(), before);
    let raw = system
        .layout(
            Source::new("A"),
            style(),
            crate::Options {
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    assert_eq!(raw.metrics().size(), crate::Size::new(20.0, 20.0));
}

fn selected_empty_control(policy: ParagraphStrut) -> SelectedLine {
    let mut system = system();
    let mut source = Source::new("A");
    source.span(
        crate::Range::new(0, 1),
        Style {
            size: 30.0,
            ..style()
        },
    );
    let analysis = AnalyzedSource::from_paragraph(ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            size: 10.0,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        paragraph_strut: policy,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: crate::Range::new(0, 1),
            aggregate: crate::Range::new(0, 1),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![paragraph::ParagraphControl {
            at: 0,
            kind: paragraph::ParagraphControlKind::ForcedBreak,
        }],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(0),
            affinity: crate::Affinity::Before,
        }],
    })
    .unwrap();
    let p = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        p.unit_views()
            .filter(|u| !u.source_range().is_empty())
            .map(|u| u.advance())
            .sum::<f64>(),
        30.0
    );
    let termination = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let end = termination.view().boundary();
    let participants: Vec<_> = p
        .participants()
        .take(end.view().participant_offset())
        .collect();
    let controls = participants
        .iter()
        .filter_map(|item| match item {
            PreparedParticipant::Control(control) => Some({
                let occurrence = control.clone();
                ControlAllocation {
                    boundary: matches!(
                        occurrence.view().kind(),
                        crate::ParagraphControlKind::InlineBoundaryStart
                            | crate::ParagraphControlKind::InlineBoundaryEnd
                    )
                    .then_some(crate::InlineBoundaryAllocation::ZERO),
                    occurrence,
                    strut: if policy == ParagraphStrut::AllocatedContributions {
                        Some(ControlStrut {
                            baseline: 0.0,
                            line_extent: 0.0,
                            alignment: LineBoxAlignment::Baseline,
                        })
                    } else {
                        None
                    },
                }
            }),
            _ => None,
        })
        .collect();
    p.select(LineSelectionRequest {
        boundary_context: Vec::new(),
        incoming: None,
        start: p.boundary_views().next().unwrap().handle(),
        end,
        termination: crate::PreparedTermination::Ordinary(termination),
        alternative: None,
        participants,
        atomics: vec![],
        controls,
        line_box_struts: Vec::new(),
        line_strut: None,
        context: LineContext {
            end: LineEnd::Forced,
            ..request(&p).context
        },
    })
    .unwrap()
}

#[test]
fn allocated_empty_line_keeps_explicit_zero_without_borrowing_body_font_metrics() {
    let selected = selected_empty_control(ParagraphStrut::AllocatedContributions);
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.feedback().line_height, 0.0);
    assert_eq!(line.feedback().baseline, 0.0);
    assert_eq!(line.run_views().count(), 0);
}

#[test]
fn default_empty_line_uses_the_captured_base_font_instead_of_overriding_body_span() {
    let selected = selected_empty_control(ParagraphStrut::DefaultFont);
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.feedback().line_height, 10.0);
    assert_eq!(line.feedback().baseline, 8.0);
    assert_eq!(line.run_views().next().unwrap().font_size(), 10.0);
    let request = selected.request();
    let placed = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: line.prepared_text(),
        coverage: PlacementCoverage::Range {
            start: request.start.clone(),
            end: request.end.clone(),
        },
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    let runs = placed.glyph_runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0].style(),
        &Style {
            size: 10.0,
            ..style()
        }
    );
}

#[test]
fn raw_out_of_flow_crlf_anchor_keeps_the_published_scalar_line_and_rectangle() {
    let mut system = system();
    for (text, anchor, ranges) in [
        (
            "\r\n",
            1,
            vec![
                crate::Range::new(0, 1),
                crate::Range::new(1, 2),
                crate::Range::new(2, 2),
            ],
        ),
        (
            "A\r\nB",
            2,
            vec![
                crate::Range::new(0, 2),
                crate::Range::new(2, 3),
                crate::Range::new(3, 4),
            ],
        ),
    ] {
        let mut source = Source::new(text);
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(51),
            crate::InlineBoxKind::OutOfFlow,
            anchor,
            crate::Size::new(3.0, 4.0),
        ));
        let layout = system
            .layout(
                source,
                Style {
                    size: 10.0,
                    line_height: crate::LineHeight::FontSizeRelative(1.0),
                    ..style()
                },
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            ranges
        );
        let boxes = layout.inline_boxes();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].line(), 1);
        assert_eq!(boxes[0].rect(), crate::Rect::new(0.0, 14.0, 3.0, 4.0));
        assert_eq!(layout.source().boxes()[0].index(), anchor);
    }
}

#[test]
fn native_unforced_break_keeps_arabic_joining_on_both_sides() {
    fn logical_glyphs(line: &FinalizedLine, fixture: &[u8]) -> Vec<(crate::Range, u32)> {
        let mut glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), fixture);
            assert_eq!(run.font_size(), 20.0);
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(instance.font_size(), 20.0);
            glyphs.extend(run.glyphs().map(|glyph| (glyph.range(), glyph.id())));
        }
        // RTL paint order cannot identify which logical side of the cut owns a form.
        glyphs.sort_by_key(|(source, _)| (source.start, source.end));
        glyphs
    }

    let fixture = crate::shaping_test_fonts::arabic_ahem();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture.clone()).unwrap();
    let source = Source::new("\u{628}\u{628}\u{628}");
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            word_break: crate::WordBreak::BreakAll,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::RightToLeft,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: crate::Range::new(0, 3),
            aggregate: crate::Range::new(0, 3),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [
            crate::Range::new(0, 1),
            crate::Range::new(1, 2),
            crate::Range::new(2, 3)
        ]
    );
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    drop(system);
    let mut whole_request = request(&prepared);
    whole_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    whole_request.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    let whole = prepared
        .select(whole_request.clone())
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    // arabic_ahem's literal Beh forms are isol35/init100/medi277/fina82.
    // CSS Text4 §6.5 requires joining forms as if the word remained whole
    // across an unforced within-word wrap (pinned reference, line3200).
    assert_eq!(
        logical_glyphs(&whole, &fixture),
        [
            (crate::Range::new(0, 1), 100),
            (crate::Range::new(1, 2), 277),
            (crate::Range::new(2, 3), 82),
        ]
    );
    let soft = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_before()
                    == crate::SourcePosition::try_new(analyzed.source().text(), 1).unwrap()
        })
        .unwrap()
        .handle();
    let cut = soft.view().boundary();
    let offset = cut.view().participant_offset();
    let participants: Vec<_> = prepared.participants().collect();
    assert!(offset > 0 && offset < participants.len());
    let mut first_request = whole_request.clone();
    first_request.end = cut.clone();
    first_request.termination = crate::PreparedTermination::Ordinary(soft);
    first_request.participants = participants[..offset].to_vec();
    first_request.context.end = LineEnd::Soft;
    first_request.context.available_inline = Some(20.0);
    let first = prepared
        .select(first_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let mut continuation_request = whole_request;
    continuation_request.start = cut;
    continuation_request.participants = participants[offset..].to_vec();
    continuation_request.context.start = LineStart::Continuation;
    continuation_request.context.available_inline = Some(40.0);
    let continuation = prepared
        .select(continuation_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.selected_line().request().context.end, LineEnd::Soft);
    assert_eq!(
        continuation.selected_line().request().context.start,
        LineStart::Continuation
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        &[0x628, 0x628, 0x628]
    );
    assert_eq!(
        [
            logical_glyphs(&first, &fixture),
            logical_glyphs(&continuation, &fixture)
        ],
        [
            vec![(crate::Range::new(0, 1), 100)],
            vec![
                (crate::Range::new(1, 2), 277),
                (crate::Range::new(2, 3), 82)
            ]
        ]
    );
}

mod line_box_struts;

mod nonformatted_placement;
