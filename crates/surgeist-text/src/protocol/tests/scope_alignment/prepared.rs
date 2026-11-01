//! Real capture-bound font resources and exact selected-scope allocations.
//! BASE coordinates and Ahem metrics are authored table facts, not sampled
//! execution output. This new API is qualified functionally.
use super::capture::{captured_input, generous, reference, style};
use crate::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const KERN: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/kern-ahem/KernAhem.ttf");

fn reason(error: &Error) -> ScopeAlignmentError {
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected scope error, got {error:?}")
    };
    *reason
}
fn inline(
    value: &mut ContextualSourceInput,
    index: usize,
    boundary: Option<ScopeAlignmentBoundary>,
    font: Option<ScopeFontReference>,
    used: Option<Style>,
) {
    let SourceProcessingScope::Inline {
        style,
        alignment_boundary,
        font_reference,
        ..
    } = &mut value.scopes[index]
    else {
        unreachable!()
    };
    *style = used;
    *alignment_boundary = boundary;
    *font_reference = font;
}
fn domain(value: &mut ContextualSourceInput, used: Style, font: Option<ScopeFontReference>) {
    let SourceProcessingScope::Block {
        base_style,
        font_reference,
        ..
    } = &mut value.scopes[0]
    else {
        unreachable!()
    };
    *base_style = used;
    *font_reference = font;
}
fn raw_source(value: &mut ContextualSourceInput, raw: &str) {
    let source = Source::identified(raw, Id::from_u64(1578020), 31);
    value.source = source.clone();
    value.origins = vec![ParagraphOrigin {
        text_node: None,
        source,
        local: Range::new(0, raw.encode_utf16().count()),
        aggregate: Range::new(0, raw.encode_utf16().count()),
        kind: ParagraphOriginKind::Authored,
    }];
}
fn author_child_body(value: &mut ContextualSourceInput) {
    let SourceProcessingScope::Inline {
        style: Some(style), ..
    } = &value.scopes[1]
    else {
        unreachable!()
    };
    // Inline.Style is a font context, not an authored text-style projection.
    // These particular fixtures also require that same declared body style.
    value
        .source
        .span(Range::new(0, value.source.text().len()), style.clone());
    for origin in &mut value.origins {
        origin.source = value.source.clone();
    }
}
fn active() -> ContextualSourceInput {
    let mut value = captured_input(1);
    raw_source(&mut value, "X");
    value.scope_alignment_limits = Some(generous());
    value.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    let mut parent = reference();
    parent.script = FontBaselineScript::from_iso15924(*b"Hani").unwrap();
    domain(&mut value, style(20.0, 24.0), Some(parent));
    let mut child = style(10.0, 10.0);
    child.font = Font::new().family("Surgeist Kern Ahem");
    inline(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
        Some(child),
    );
    value
}
fn scoped_fonts() -> (Vec<u8>, Vec<u8>) {
    use crate::baseline_test_fonts::{Axis, Coord, Script, base};
    // One parent font has distinct real script rows: Hani250 != Latn100.
    // The child has a genuinely different family and romn coordinate-100.
    let parent_axis = Axis {
        tags: vec![*b"romn"],
        scripts: vec![
            Script {
                tag: *b"hani",
                default_index: 0,
                coordinates: vec![Coord::Design(250)],
            },
            Script {
                tag: *b"latn",
                default_index: 0,
                coordinates: vec![Coord::Design(100)],
            },
        ],
    };
    let child_axis = Axis {
        tags: vec![*b"romn"],
        scripts: vec![Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Design(-100)],
        }],
    };
    (
        crate::test_fonts::replace_tables(
            AHEM,
            vec![(*b"BASE", base(0, Some(parent_axis), None, None))],
        ),
        crate::test_fonts::replace_tables(
            KERN,
            vec![(*b"BASE", base(0, Some(child_axis), None, None))],
        ),
    )
}
fn system(parent: &[u8], child: &[u8]) -> System {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(parent.to_vec()).unwrap();
    s.register_fonts(child.to_vec()).unwrap();
    s
}
fn analyze(value: ContextualSourceInput) -> (Arc<ProcessedSource>, AnalyzedSource) {
    let capture = Arc::new(process_contextual_source(value).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    (capture, analyzed)
}
fn selected_request(
    p: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut r = super::super::request(p);
    r.start = start;
    r.end = offer.view().boundary();
    let a = r.start.view().participant_offset();
    let b = r.end.view().participant_offset();
    r.participants = p.participants().skip(a).take(b - a).collect();
    r.context.start = if a == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = match offer.view().kind() {
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => LineEnd::Soft,
    };
    r.context.available_inline = None;
    r.context.spacing = SpacingPolicy::None;
    r.context.alignment = Alignment::Start;
    r.termination = PreparedTermination::Ordinary(offer);
    r
}
fn select(p: &PreparedText) -> SelectedLine {
    let start = p.boundary_views().next().unwrap().handle();
    let end = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    p.select(selected_request(p, start, end)).unwrap()
}
fn allocations(scope: SelectedScope, amount: f64) -> LineAllocations {
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
            scope,
            displacement: BaselineShift::try_new(amount).unwrap(),
        }]),
    }
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| (g.id(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn place(p: &PreparedText, rows: Vec<(FinalizedLine, f64)>) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: rows
            .into_iter()
            .map(|(line, y)| {
                PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: y,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}
fn available(outcome: Option<FontBaselineOutcome>, coordinate: f64, script: [u8; 4]) {
    let Some(FontBaselineOutcome::Available(value)) = outcome else {
        panic!("actual BASE coordinate must be retained")
    };
    assert_eq!(value.coordinate(), coordinate);
    assert_eq!(value.base_script(), Some(script));
}

#[test]
fn glyph_free_parent_retains_its_real_family_script_size_and_reference_before_selection() {
    let (parent_bytes, child_bytes) = scoped_fonts();
    for bytes in [&parent_bytes, &child_bytes] {
        let font = skrifa::FontRef::new(bytes).unwrap();
        assert_eq!(font.head().unwrap().units_per_em(), 1000);
        assert_eq!(
            (
                font.hhea().unwrap().ascender().to_i16(),
                font.hhea().unwrap().descender().to_i16()
            ),
            (800, -200)
        );
        assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
        assert!(font.base().is_ok());
    }
    let mut s = system(&parent_bytes, &child_bytes);
    for scale in [1.0_f32, 2.0] {
        for named in [false, true] {
            let mut input = active();
            author_child_body(&mut input);
            let (capture, analysis) = analyze(input);
            let parent = capture.captured_scope(0).unwrap();
            let child = capture.captured_scope(1).unwrap();
            let early = child.scope_alignment_usage();
            let mut options =
                PreparationOptions::try_new(scale, PreparationOptions::default().tabs()).unwrap();
            if named {
                options = options.with_baseline_reference(FontBaseline::Alphabetic);
            }
            let p = s.prepare(&analysis, options).unwrap();
            let stats = s.stats();
            let parent_metrics = p.scope_metrics(&parent).unwrap();
            let child_metrics = p.scope_metrics(&child).unwrap();
            let m = f64::from(scale);
            let parent_native = if named { 5.0 * m } else { 0.0 };
            let child_native = if named { -m } else { 0.0 };
            assert_eq!(parent_metrics.captured_scope(), parent);
            assert_eq!(child_metrics.captured_scope(), child);
            assert_eq!(parent_metrics.font_size(), 20.0 * scale);
            assert_eq!(child_metrics.font_size(), 10.0 * scale);
            assert_eq!(parent_metrics.line_height(), 24.0 * m);
            assert_eq!(child_metrics.line_height(), 10.0 * m);
            assert_eq!(parent_metrics.native_reference_coordinate(), parent_native);
            assert_eq!(child_metrics.native_reference_coordinate(), child_native);
            assert_eq!(
                (parent_metrics.over(), parent_metrics.under()),
                (16.0 * m - parent_native, 4.0 * m + parent_native)
            );
            assert_eq!(
                (child_metrics.over(), child_metrics.under()),
                (8.0 * m - child_native, 2.0 * m + child_native)
            );
            assert_eq!(
                parent_metrics.matching_coordinate(FontBaseline::Alphabetic),
                Some(5.0 * m - parent_native)
            );
            assert_eq!(
                child_metrics.matching_coordinate(FontBaseline::Alphabetic),
                Some(-m - child_native)
            );
            available(
                parent_metrics.baseline_outcome(FontBaseline::Alphabetic),
                5.0 * m,
                *b"hani",
            );
            available(
                child_metrics.baseline_outcome(FontBaseline::Alphabetic),
                -m,
                *b"latn",
            );
            assert_eq!(
                parent_metrics.common_baseline_reference(),
                if named {
                    Some(FontBaseline::Alphabetic)
                } else {
                    None
                }
            );
            assert_eq!(parent_metrics.orientation(), LineOrientation::Horizontal);
            assert!(
                parent_metrics
                    .matching_coordinate(FontBaseline::Central)
                    .is_none()
            );
            assert_eq!(
                parent_metrics.effective_instance().unwrap().face().bytes(),
                parent_bytes
            );
            assert_eq!(
                child_metrics.effective_instance().unwrap().face().bytes(),
                child_bytes
            );
            assert_eq!(
                s.stats(),
                stats,
                "retained prepared views do not resolve fonts"
            );
            let selected = select(&p);
            let selected_stats = s.stats();
            let scope = selected.scopes().next().unwrap();
            assert_eq!(selected.scopes().len(), 1);
            assert_eq!(scope.captured_scope(), child);
            assert_eq!(scope.parent_metrics().unwrap().captured_scope(), parent);
            assert_eq!(
                scope.metrics().effective_instance().unwrap().face().bytes(),
                child_bytes
            );
            assert_eq!(
                s.stats(),
                selected_stats,
                "retained selected metric views do not resolve fonts"
            );
            assert_eq!(
                child.scope_alignment_usage(),
                early,
                "capture accounting is immutable, not a shared ledger"
            );
            let retention = p.scope_alignment_retention().unwrap();
            let usage = p.scope_alignment_usage().unwrap();
            assert!(retention.records() >= 2);
            assert!(retention.payload_bytes() >= parent_bytes.len() + child_bytes.len());
            assert!(retention.records() <= usage.records());
            assert!(retention.payload_bytes() <= usage.peak_payload_bytes());
            assert!(usage.peak_payload_bytes() <= generous().payload_bytes());
            assert!(usage.font_queries() > 0);
            let finalized = selected.finalize(allocations(scope, 0.0)).unwrap();
            assert_eq!(glyphs(&finalized), [(58, 0.0, 0.0, 10.0 * scale)]);
            assert_eq!(
                finalized.feedback().line_height,
                10.0 * m,
                "parent receipt does not donate a root strut"
            );
        }
    }
}

#[test]
fn dormant_context_does_not_create_scope_resources_or_targets_and_font_query_zero_is_real() {
    let mut value = active();
    domain(&mut value, style(10.0, 10.0), Some(reference()));
    inline(&mut value, 1, None, None, Some(style(10.0, 10.0)));
    value.scope_alignment_limits = Some(ScopeAlignmentLimits::new(4096, 0, 1_000_000, 16_777_216));
    let (capture, analysis) = analyze(value);
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    let p = s.prepare(&analysis, PreparationOptions::default()).unwrap();
    assert_eq!(p.scope_alignment_usage().unwrap().font_queries(), 0);
    assert_eq!(
        reason(
            &p.scope_metrics(&capture.captured_scope(0).unwrap())
                .unwrap_err()
        ),
        ScopeAlignmentError::MissingMetrics { scope: 0 }
    );
    assert_eq!(
        reason(
            &p.scope_metrics(&capture.captured_scope(1).unwrap())
                .unwrap_err()
        ),
        ScopeAlignmentError::MissingMetrics { scope: 1 }
    );
    let selected = select(&p);
    assert_eq!(selected.scopes().len(), 0);
    assert!(
        selected
            .unscoped_metric_contributions()
            .any(|c| c.before == 8.0 && c.after == 2.0)
    );
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (8.0, 10.0)
    );
    assert!(line.scope_allocations().is_empty());
    let mut demanded = active();
    demanded.scope_alignment_limits =
        Some(ScopeAlignmentLimits::new(4096, 0, 1_000_000, 16_777_216));
    let (_, analysis) = analyze(demanded);
    let (parent, child) = scoped_fonts();
    let mut s = system(&parent, &child);
    let error = s
        .prepare(&analysis, PreparationOptions::default())
        .unwrap_err();
    assert!(matches!(
        reason(&error),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::FontQueries,
            limit: 0,
            required: 1
        }
    ));
}

#[test]
fn only_actual_demand_requires_each_parent_style_reference_and_inline_boundary_fact() {
    let (parent, child) = scoped_fonts();
    let mut s = system(&parent, &child);
    for missing in 0..3 {
        let mut value = active();
        let expected = match missing {
            0 => {
                let SourceProcessingScope::Block { font_reference, .. } = &mut value.scopes[0]
                else {
                    unreachable!()
                };
                *font_reference = None;
                ScopeAlignmentError::MissingFontReference { scope: 0 }
            }
            1 => {
                let SourceProcessingScope::Inline { font_reference, .. } = &mut value.scopes[1]
                else {
                    unreachable!()
                };
                *font_reference = None;
                ScopeAlignmentError::MissingFontReference { scope: 1 }
            }
            _ => {
                let SourceProcessingScope::Inline { style, .. } = &mut value.scopes[1] else {
                    unreachable!()
                };
                *style = None;
                ScopeAlignmentError::MissingStyle { scope: 1 }
            }
        };
        let (_, analysis) = analyze(value);
        assert_eq!(
            reason(
                &s.prepare(&analysis, PreparationOptions::default())
                    .unwrap_err()
            ),
            expected
        );
    }
    let mut value = captured_input(2);
    raw_source(&mut value, "X");
    value.scope_alignment_limits = Some(generous());
    domain(&mut value, style(10.0, 10.0), Some(reference()));
    inline(
        &mut value,
        1,
        None,
        Some(reference()),
        Some(style(10.0, 10.0)),
    );
    inline(
        &mut value,
        2,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
        Some(style(10.0, 10.0)),
    );
    let (_, analysis) = analyze(value);
    assert_eq!(
        reason(
            &s.prepare(&analysis, PreparationOptions::default())
                .unwrap_err()
        ),
        ScopeAlignmentError::MissingBoundary { scope: 1 }
    );
}

pub(super) fn two_leaves() -> ContextualSourceInput {
    let mut value = captured_input(1);
    raw_source(&mut value, "XX");
    value.scope_alignment_limits = Some(generous());
    domain(&mut value, style(10.0, 10.0), Some(reference()));
    inline(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
        Some(style(10.0, 10.0)),
    );
    value.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    let source = value.source.clone();
    value.origins = (0..2)
        .map(|i| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(i, i + 1),
            aggregate: Range::new(i, i + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    value.boundary_policies = vec![ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    use SourceProcessingEvent as E;
    value.events = vec![
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
    ];
    value
}

#[test]
fn a_real_scoped_glyph_moves_with_its_carets_selection_and_signed_envelope_once() {
    let (capture, analysis) = analyze(two_leaves());
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    let p = s.prepare(&analysis, PreparationOptions::default()).unwrap();
    let selected = select(&p);
    let scope = selected.scopes().next().unwrap();
    assert_eq!(scope.captured_scope(), capture.captured_scope(1).unwrap());
    assert_eq!(
        scope.direct_metric_contributions().collect::<Vec<_>>(),
        [ScopeMetricContribution {
            before: 8.0,
            after: 2.0,
            alignment: LineBoxAlignment::Baseline
        }]
    );
    assert_eq!(
        selected.unscoped_metric_contributions().collect::<Vec<_>>(),
        [ScopeMetricContribution {
            before: 8.0,
            after: 2.0,
            alignment: LineBoxAlignment::Baseline
        }]
    );
    let zero = selected.finalize(allocations(scope.clone(), 0.0)).unwrap();
    assert_eq!(
        (zero.feedback().baseline, zero.feedback().line_height),
        (8.0, 10.0)
    );
    let shifted = selected.finalize(allocations(scope.clone(), 3.0)).unwrap();
    assert_eq!(
        (
            shifted.feedback().advance,
            shifted.feedback().baseline,
            shifted.feedback().line_height
        ),
        (20.0, 11.0, 13.0)
    );
    assert_eq!(
        glyphs(&shifted),
        [(58, 0.0, 0.0, 10.0), (58, 0.0, 0.0, 10.0)]
    );
    let frames = shifted.scope_allocations();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].scope, scope.captured_scope());
    assert_eq!(frames[0].displacement.get(), 3.0);
    assert_eq!(frames[0].cumulative_displacement.get(), 3.0);
    let stats = s.stats();
    let replay = shifted.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.scope_allocations(), shifted.scope_allocations());
    assert_eq!(s.stats(), stats);
    drop(s);
    let layout = place(&p, vec![(shifted, 100.0)]);
    assert_eq!(
        layout
            .run_views()
            .map(|r| (r.origin(), r.range()))
            .collect::<Vec<_>>(),
        [
            (Point::new(0.0, 100.0), Range::new(0, 1)),
            (Point::new(10.0, 97.0), Range::new(1, 2))
        ]
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(0.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(1, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(10.0, 92.0, 1.0, 10.0)
    );
    let scoped_start = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(1, Affinity::After))
        .unwrap();
    assert_eq!(
        layout.position_geometry(&scoped_start).unwrap().rect(),
        Rect::new(10.0, 89.0, 1.0, 10.0)
    );
    assert_eq!(layout.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    assert_eq!(
        layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(2, Affinity::Before)
            ))
            .unwrap()
            .rects()
            .iter()
            .map(|r| r.rect())
            .collect::<Vec<_>>(),
        [
            Rect::new(0.0, 92.0, 10.0, 10.0),
            Rect::new(10.0, 89.0, 10.0, 10.0)
        ]
    );
    assert_eq!(layout.source().text().code_units(), [88, 88]);
    assert_eq!(place(&p, vec![(replay, 100.0)]).bounds(), layout.bounds());
}

#[test]
fn selected_scope_allocations_require_complete_unique_current_handles_and_preserve_prior_results() {
    let (_, analysis) = analyze(two_leaves());
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    let p = s.prepare(&analysis, PreparationOptions::default()).unwrap();
    let first = select(&p);
    let handle = first.scopes().next().unwrap();
    let old = first.finalize(allocations(handle.clone(), 3.0)).unwrap();
    for scopes in [ScopeAllocations::None, ScopeAllocations::Complete(vec![])] {
        assert_eq!(
            reason(
                &first
                    .finalize(LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes
                    })
                    .unwrap_err()
            ),
            ScopeAlignmentError::MissingAllocation { scope: 1 }
        );
    }
    let row = ScopeAllocation {
        scope: handle.clone(),
        displacement: BaselineShift::try_new(0.0).unwrap(),
    };
    assert_eq!(
        reason(
            &first
                .finalize(LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: ScopeAllocations::Complete(vec![row.clone(), row])
                })
                .unwrap_err()
        ),
        ScopeAlignmentError::DuplicateAllocation { scope: 1 }
    );
    let retry = select(&p);
    let retry_handle = retry.scopes().next().unwrap();
    assert_ne!(handle, retry_handle);
    assert_eq!(handle.captured_scope(), retry_handle.captured_scope());
    assert_eq!(
        reason(
            &retry
                .finalize(allocations(handle.clone(), 0.0))
                .unwrap_err()
        ),
        ScopeAlignmentError::ForeignSelectedScope
    );
    assert_eq!(
        reason(&handle.shares_operands_with(&retry_handle).unwrap_err()),
        ScopeAlignmentError::ForeignSelectedScope
    );
    let foreign = process_contextual_source(two_leaves())
        .unwrap()
        .captured_scope(1)
        .unwrap();
    assert_eq!(
        reason(&p.scope_metrics(&foreign).unwrap_err()),
        ScopeAlignmentError::ForeignCapture
    );
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (11.0, 13.0)
    );
    assert_eq!(old.scope_allocations()[0].displacement.get(), 3.0);
    let selected_usage = first.scope_alignment_usage().unwrap();
    let selected_retention = first.scope_alignment_retention().unwrap();
    assert!(selected_retention.records() <= selected_usage.records());
    assert!(selected_retention.payload_bytes() <= selected_usage.peak_payload_bytes());
    let final_usage = old.scope_alignment_usage().unwrap();
    let final_retention = old.scope_alignment_retention().unwrap();
    assert!(final_retention.records() <= final_usage.records());
    assert!(final_retention.payload_bytes() <= final_usage.peak_payload_bytes());
    assert!(final_usage.peak_payload_bytes() <= generous().payload_bytes());
}

#[test]
fn a_genuinely_empty_dormant_inline_does_not_demand_missing_reference_or_unavailable_font() {
    let mut value = active();
    domain(&mut value, style(10.0, 10.0), Some(reference()));
    let mut unavailable = style(80.0, 90.0);
    unavailable.font = Font::new().family("Missing dormant family");
    inline(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::NonInitial),
        None,
        Some(unavailable),
    );
    use SourceProcessingEvent as E;
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::ExitScope(1),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    value.scope_alignment_limits = Some(ScopeAlignmentLimits::new(4096, 0, 1_000_000, 16_777_216));
    let (capture, analysis) = analyze(value);
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    let p = s.prepare(&analysis, PreparationOptions::default()).unwrap();
    assert_eq!(p.scope_alignment_usage().unwrap().font_queries(), 0);
    assert_eq!(
        reason(
            &p.scope_metrics(&capture.captured_scope(1).unwrap())
                .unwrap_err()
        ),
        ScopeAlignmentError::MissingMetrics { scope: 1 }
    );
    let selected = select(&p);
    assert_eq!(selected.scopes().len(), 0);
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (8.0, 10.0)
    );
    assert_eq!(glyphs(&line), [(58, 0.0, 0.0, 10.0)]);
}

#[test]
fn real_indication_and_incoming_fragments_issue_distinct_handles_for_the_same_captured_scope() {
    let mut value = active();
    raw_source(&mut value, "AB\u{ad}CCC");
    let mut child = style(10.0, 10.0);
    child.font = Font::new().family("Surgeist Kern Ahem");
    child.locale = Some("und-Latn".into());
    child.hyphens = Hyphens::Manual;
    child.native_hyphenation_convention = Some(
        NativeHyphenationConvention::language_script(
            "und-Latn",
            "Latn",
            Text::from("X"),
            HyphenationPlacement::BothSides,
        )
        .unwrap(),
    );
    inline(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
        Some(child),
    );
    author_child_body(&mut value);
    let (capture, analysis) = analyze(value);
    let (parent, child) = scoped_fonts();
    let mut s = system(&parent, &child);
    let p = s.prepare(&analysis, PreparationOptions::default()).unwrap();
    let offer = p
        .hyphenation_cut_views()
        .find(|cut| cut.boundary().view().source_before().get() == 3)
        .unwrap()
        .opportunity()
        .unwrap();
    let alternative = offer.view().alternatives().next().unwrap().handle();
    assert_eq!(alternative.view().source_range(), Some(Range::new(2, 3)));
    let start = p.boundary_views().next().unwrap().handle();
    let mut r = selected_request(&p, start, offer.clone());
    r.alternative = Some(alternative.clone());
    let first = p.select(r).unwrap();
    let first_scope = first.scopes().next().unwrap();
    let first_line = first
        .finalize(allocations(first_scope.clone(), 3.0))
        .unwrap();
    let end = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let mut r = selected_request(&p, offer.view().boundary(), end);
    r.incoming = first_line.hyphenation_continuation();
    let second = p.select(r).unwrap();
    let second_scope = second.scopes().next().unwrap();
    assert_ne!(first_scope, second_scope);
    assert_eq!(
        first_scope.captured_scope(),
        capture.captured_scope(1).unwrap()
    );
    assert_eq!(first_scope.captured_scope(), second_scope.captured_scope());
    assert_eq!(
        reason(&second.finalize(allocations(first_scope, 3.0)).unwrap_err()),
        ScopeAlignmentError::ForeignSelectedScope
    );
    let second_line = second.finalize(allocations(second_scope, 3.0)).unwrap();
    for line in [&first_line, &second_line] {
        assert_eq!(line.scope_allocations().len(), 1);
        assert_eq!(
            line.scope_allocations()[0].scope,
            capture.captured_scope(1).unwrap()
        );
        assert_eq!(
            line.scope_allocations()[0].cumulative_displacement.get(),
            3.0
        );
    }
    assert_eq!(
        (
            first_line.feedback().advance,
            second_line.feedback().advance
        ),
        (30.0, 40.0)
    );
    assert_eq!(glyphs(&first_line).iter().filter(|g| g.0 == 58).count(), 1);
    assert_eq!(glyphs(&second_line).iter().filter(|g| g.0 == 58).count(), 1);
    assert_eq!(
        first_line.selected_line().request().alternative,
        Some(alternative)
    );
    let stats = s.stats();
    let replay = first_line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.scope_allocations(), first_line.scope_allocations());
    assert_eq!(s.stats(), stats);
    let layout = place(&p, vec![(first_line, 100.0), (second_line, 130.0)]);
    assert_eq!(
        layout.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 6)]
    );
    assert_eq!(
        layout.source().text().code_units(),
        [65, 66, 173, 67, 67, 67]
    );
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), child);
    }
}

#[test]
fn glyph_free_parent_scales_only_consumed_font_metrics_not_unused_spacing_or_decoration() {
    let (parent_bytes, child_bytes) = scoped_fonts();
    let mut system = system(&parent_bytes, &child_bytes);
    for unused_extremes in [false, true] {
        let mut input = active();
        if unused_extremes {
            let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
                unreachable!()
            };
            // Every authored value is finite and independently legal. Scaling
            // these UNUSED fields by2 would overflow, unlike the consumed
            // parent20px font/24px own line height. Only the child owns body X.
            base_style.letter_spacing = f64::MAX;
            base_style.word_spacing = -f64::MAX;
            base_style.underline = Decoration::solid()
                .with_offset(DecorationOffset::try_absolute(f64::MAX).unwrap())
                .with_thickness(DecorationThickness::try_absolute(f64::MAX).unwrap());
            base_style.strikethrough = Decoration::solid()
                .with_offset(DecorationOffset::try_absolute(-f64::MAX).unwrap())
                .with_thickness(DecorationThickness::try_absolute(f64::MAX).unwrap());
        }
        author_child_body(&mut input);
        let (capture, analysis) = analyze(input);
        let parent = capture.captured_scope(0).unwrap();
        let child = capture.captured_scope(1).unwrap();
        let options =
            PreparationOptions::try_new(2.0, PreparationOptions::default().tabs()).unwrap();
        let prepared = system.prepare(&analysis, options).unwrap();
        let parent_metrics = prepared.scope_metrics(&parent).unwrap();
        let child_metrics = prepared.scope_metrics(&child).unwrap();
        // Real Ahem hhea800/-200 and Hani BASE250 at40px, with raw native
        // zero: over32/under8, matching romn10. Parent own line height is48.
        assert_eq!(parent_metrics.font_size(), 40.0);
        assert_eq!(parent_metrics.line_height(), 48.0);
        assert_eq!((parent_metrics.over(), parent_metrics.under()), (32.0, 8.0));
        assert_eq!(parent_metrics.native_reference_coordinate(), 0.0);
        assert_eq!(
            parent_metrics.matching_coordinate(FontBaseline::Alphabetic),
            Some(10.0)
        );
        available(
            parent_metrics.baseline_outcome(FontBaseline::Alphabetic),
            10.0,
            *b"hani",
        );
        assert_eq!(
            parent_metrics.effective_instance().unwrap().face().bytes(),
            parent_bytes
        );
        // The actual ordinary Kern Ahem child remains20px with its own20px
        // line height and Latn BASE-100 => -2px, independent of parent extras.
        assert_eq!(child_metrics.font_size(), 20.0);
        assert_eq!(child_metrics.line_height(), 20.0);
        assert_eq!((child_metrics.over(), child_metrics.under()), (16.0, 4.0));
        assert_eq!(
            child_metrics.matching_coordinate(FontBaseline::Alphabetic),
            Some(-2.0)
        );
        assert_eq!(
            child_metrics.effective_instance().unwrap().face().bytes(),
            child_bytes
        );
        let selected = select(&prepared);
        assert_eq!(selected.scopes().len(), 1);
        let scope = selected.scopes().next().unwrap();
        assert_eq!(scope.captured_scope(), child);
        assert_eq!(scope.parent_metrics().unwrap().captured_scope(), parent);
        assert!(selected.unscoped_metric_contributions().next().is_none());
        let stats = system.stats();
        let finalized = selected.finalize(allocations(scope, 0.0)).unwrap();
        assert_eq!(glyphs(&finalized), [(58, 0.0, 0.0, 20.0)]);
        assert_eq!(
            (
                finalized.feedback().advance,
                finalized.feedback().baseline,
                finalized.feedback().line_height
            ),
            (20.0, 16.0, 20.0),
            "the glyph-free parent receipt donates no48px paragraph strut"
        );
        assert_eq!(system.stats(), stats);
    }
}

#[path = "prepared/parent_geometry_admission.rs"]
mod parent_geometry_admission;

#[path = "prepared/parent_geometry_resources.rs"]
mod parent_geometry_resources;

#[path = "prepared/parent_atomic_measure.rs"]
mod parent_atomic_measure;

#[path = "prepared/fragment_body_custody.rs"]
mod fragment_body_custody;
