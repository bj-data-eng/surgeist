//! Canonical direct chains retain global append order and actual signed metrics.
use super::capture::{generous, reference, style};
use crate::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn metric(before: f64, after: f64, alignment: LineBoxAlignment) -> ScopeMetricContribution {
    ScopeMetricContribution {
        before,
        after,
        alignment,
    }
}
fn shift(value: f64) -> LineBoxAlignment {
    LineBoxAlignment::Shift(BaselineShift::try_new(value).unwrap())
}
fn exact(
    values: impl Iterator<Item = ScopeMetricContribution>,
    expected: &[ScopeMetricContribution],
) {
    // One extra next also detects a duplicated/cyclic tail without an unbounded
    // test collection. Equality retains every value, tag, occurrence and order.
    assert_eq!(
        values.take(expected.len() + 1).collect::<Vec<_>>(),
        expected
    );
}
fn selected_scope(selected: &SelectedLine, actual: &CapturedScope) -> SelectedScope {
    selected
        .scopes()
        .find(|scope| scope.captured_scope() == *actual)
        .unwrap()
}
fn chains(
    selected: &SelectedLine,
    ancestor: &CapturedScope,
    outer: &CapturedScope,
    inner: &CapturedScope,
) {
    use LineBoxAlignment::Baseline as B;
    let ancestor = selected_scope(selected, ancestor);
    let outer = selected_scope(selected, outer);
    let inner = selected_scope(selected, inner);
    exact(ancestor.direct_metric_contributions(), &[]);
    exact(
        outer.direct_metric_contributions(),
        &[
            metric(4.0, -2.0, B),
            metric(4.0, 2.0, B),
            metric(2.0, 6.0, shift(-2.0)),
        ],
    );
    exact(
        inner.direct_metric_contributions(),
        &[
            metric(6.0, 0.0, B),
            metric(2.0, 1.0, shift(-1.0)),
            metric(7.0, 2.0, B),
        ],
    );
    exact(
        selected.unscoped_metric_contributions(),
        &[
            metric(5.0, -1.0, B),
            metric(3.0, 4.0, B),
            metric(1.0, 4.0, shift(2.0)),
            metric(9.0, 1.0, B),
        ],
    );
    // Descendants filter the global append sequence: outer font, inner font,
    // then the interleaved allocated rows. Concatenating direct chains by depth
    // would place the two outer numeric rows before the inner font and fail.
    let descendants = [
        metric(4.0, -2.0, B),
        metric(6.0, 0.0, B),
        metric(2.0, 1.0, shift(-1.0)),
        metric(4.0, 2.0, B),
        metric(7.0, 2.0, B),
        metric(2.0, 6.0, shift(-2.0)),
    ];
    exact(ancestor.metric_contributions(), &descendants);
    exact(outer.metric_contributions(), &descendants);
    exact(
        inner.metric_contributions(),
        &[
            metric(6.0, 0.0, B),
            metric(2.0, 1.0, shift(-1.0)),
            metric(7.0, 2.0, B),
        ],
    );
    for scope in [ancestor, outer, inner] {
        assert_eq!(scope.selected_line(), *selected);
        assert_eq!(
            scope.metrics().effective_instance().unwrap().face().bytes(),
            AHEM
        );
    }
}
fn allocations(
    selected: &SelectedLine,
    outer: &CapturedScope,
    inner: &CapturedScope,
    own_outer: f64,
    own_inner: f64,
) -> LineAllocations {
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::Complete(
            selected
                .scopes()
                .map(|scope| {
                    let actual = scope.captured_scope();
                    let amount = if actual == *outer {
                        own_outer
                    } else if actual == *inner {
                        own_inner
                    } else {
                        0.0
                    };
                    ScopeAllocation {
                        scope,
                        displacement: BaselineShift::try_new(amount).unwrap(),
                    }
                })
                .collect(),
        ),
    }
}
fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    super::super::place(prepared.clone(), line, 0.0, 100.0).unwrap()
}
fn glyphs(layout: &Layout, baselines: [f64; 3]) {
    assert_eq!(layout.source().text().code_units(), [88, 88, 88]);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 3));
    let runs = layout.run_views().collect::<Vec<_>>();
    assert_eq!(runs.len(), 3);
    for (index, run) in runs.into_iter().enumerate() {
        assert_eq!(run.range(), Range::new(index, index + 1));
        assert_eq!(
            run.origin(),
            Point::new([0.0, 10.0, 20.0][index], baselines[index])
        );
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.x(), glyph.y(), glyph.advance()))
                .collect::<Vec<_>>(),
            [(58, 0.0, 0.0, 10.0)]
        );
    }
}

#[test]
fn nested_direct_and_common_chains_preserve_signed_rows_and_global_descendant_order() {
    use LineBoxAlignment::Baseline as B;
    use SourceProcessingEvent as E;
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        (
            font.hhea().unwrap().ascender().to_i16(),
            font.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
    // Font8/2 at size10 with half-leading -3,-4,-2 gives genuine signed
    // contributions5/-1,4/-2,6/0 for line heights4,2,6 respectively.
    let mut source = Source::identified("XXX", Id::from_u64(1578077), 61);
    for (index, height) in [4.0, 2.0, 6.0].into_iter().enumerate() {
        source.span(Range::new(index, index + 1), style(10.0, height));
    }
    let capture = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            scope_alignment_limits: Some(generous()),
            origins: (0..3)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..2)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..2)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            // Supplied ordinal1 is deepest, ordinal3 is the glyph-free ancestor.
            // Actual source entry order, not these ordinals, determines frames.
            scopes: vec![
                SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: style(10.0, 4.0),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    font_reference: Some(reference()),
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: WhiteSpaceTrim::default(),
                    style: Some(style(10.0, 6.0)),
                    boundaries: None,
                    alignment_boundary: Some(ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference()),
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: WhiteSpaceTrim::default(),
                    style: Some(style(10.0, 2.0)),
                    boundaries: None,
                    alignment_boundary: Some(ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference()),
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: WhiteSpaceTrim::default(),
                    style: Some(style(10.0, 10.0)),
                    boundaries: None,
                    alignment_boundary: Some(ScopeAlignmentBoundary::Initial),
                    font_reference: Some(reference()),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                E::Connection(0),
                E::EnterScope(3),
                E::EnterScope(2),
                E::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                E::Connection(1),
                E::EnterScope(1),
                E::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                E::ExitScope(1),
                E::ExitScope(2),
                E::ExitScope(3),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let ancestor = capture.captured_scope(3).unwrap();
    let outer = capture.captured_scope(2).unwrap();
    let inner = capture.captured_scope(1).unwrap();
    assert_eq!(inner.parent(), Some(outer.clone()));
    assert_eq!(outer.parent(), Some(ancestor.clone()));
    let analyzed = AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared
            .origin_views()
            .map(|origin| (
                origin.source().id(),
                origin.source().revision(),
                origin.local_range(),
                origin.aggregate_range(),
                origin.kind()
            ))
            .collect::<Vec<_>>(),
        (0..3)
            .map(|index| (
                Some(Id::from_u64(1578077)),
                61,
                Range::new(index, index + 1),
                Range::new(index, index + 1),
                ParagraphOriginKind::Authored
            ))
            .collect::<Vec<_>>()
    );
    for (scope, height) in [(&ancestor, 10.0), (&outer, 2.0), (&inner, 6.0)] {
        let receipt = prepared.scope_metrics(scope).unwrap();
        assert_eq!(receipt.captured_scope(), *scope);
        assert_eq!(receipt.line_height(), height);
        assert_eq!((receipt.over(), receipt.under()), (8.0, 2.0));
    }
    let rows = vec![
        LineBoxStrut {
            scope: Some(inner.clone()),
            metrics: LineStrut {
                baseline: 2.0,
                line_extent: 3.0,
            },
            alignment: shift(-1.0),
        },
        LineBoxStrut {
            scope: None,
            metrics: LineStrut {
                baseline: 1.0,
                line_extent: 5.0,
            },
            alignment: shift(2.0),
        },
        LineBoxStrut {
            scope: Some(outer.clone()),
            metrics: LineStrut {
                baseline: 4.0,
                line_extent: 6.0,
            },
            alignment: B,
        },
        LineBoxStrut {
            scope: Some(inner.clone()),
            metrics: LineStrut {
                baseline: 7.0,
                line_extent: 9.0,
            },
            alignment: B,
        },
        LineBoxStrut {
            scope: None,
            metrics: LineStrut {
                baseline: 9.0,
                line_extent: 10.0,
            },
            alignment: B,
        },
        LineBoxStrut {
            scope: Some(outer.clone()),
            metrics: LineStrut {
                baseline: 2.0,
                line_extent: 8.0,
            },
            alignment: shift(-2.0),
        },
    ];
    let mut request = super::super::request(&prepared);
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    request.context.alignment = Alignment::Start;
    request.line_strut = Some(LineStrut {
        baseline: 3.0,
        line_extent: 7.0,
    });
    request.line_box_struts = rows.clone();
    let selected = prepared.select(request).unwrap();
    assert_eq!(
        selected
            .scopes()
            .map(|scope| scope.captured_scope())
            .collect::<Vec<_>>(),
        [ancestor.clone(), outer.clone(), inner.clone()]
    );
    assert_eq!(selected.request().line_box_struts, rows);
    chains(&selected, &ancestor, &outer, &inner);
    let old = selected
        .finalize(allocations(&selected, &outer, &inner, 0.0, 0.0))
        .unwrap();
    // At zero frames, common9/1 fixes before9; outer2/6 with own-2 fixes after8.
    assert_eq!(
        (
            old.feedback().advance,
            old.feedback().baseline,
            old.feedback().line_height
        ),
        (30.0, 9.0, 17.0)
    );
    let old_layout = place(&prepared, old.clone());
    glyphs(&old_layout, [100.0, 100.0, 100.0]);
    assert_eq!(
        old_layout.lines()[0].bounds(),
        Rect::new(0.0, 91.0, 30.0, 17.0)
    );
    let stats = system.stats();
    let line = selected
        .finalize(allocations(&selected, &outer, &inner, 3.0, -1.0))
        .unwrap();
    // Cumulative frames are ancestor0/outer3/inner2. Common9/1 fixes before9;
    // outer2/6 with own-2 plus inherited3 fixes after5, so the union is9/5.
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (30.0, 9.0, 14.0)
    );
    assert_eq!(
        line.scope_allocations()
            .iter()
            .map(|row| (
                row.scope.clone(),
                row.displacement.get(),
                row.cumulative_displacement.get()
            ))
            .collect::<Vec<_>>(),
        [
            (ancestor.clone(), 0.0, 0.0),
            (outer.clone(), 3.0, 3.0),
            (inner.clone(), -1.0, 2.0)
        ]
    );
    assert_eq!(
        (0..6)
            .map(|index| line.line_box_strut_scope_displacement(index).unwrap().get())
            .collect::<Vec<_>>(),
        [2.0, 0.0, 3.0, 2.0, 0.0, 3.0]
    );
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line(), selected);
    assert_eq!(replay.scope_allocations(), line.scope_allocations());
    assert_eq!(replay.selected_line().request().line_box_struts, rows);
    let layout = place(&prepared, line);
    let replay_layout = place(&prepared, replay);
    glyphs(&layout, [100.0, 97.0, 98.0]);
    glyphs(&replay_layout, [100.0, 97.0, 98.0]);
    assert_eq!(layout.lines()[0].bounds(), Rect::new(0.0, 91.0, 30.0, 14.0));
    assert_eq!(
        replay_layout.lines()[0].bounds(),
        layout.lines()[0].bounds()
    );
    chains(&selected, &ancestor, &outer, &inner);
    assert_eq!(system.stats(), stats);
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (9.0, 17.0)
    );
    glyphs(&old_layout, [100.0, 100.0, 100.0]);
}
