//! Layout consumes actual early Text boundaries; these tests do not execute the
//! six-value CSS box-edge declaration/applicability/propagation operation.

use super::*;
use std::{cell::Cell, collections::HashMap, sync::Arc};
use surgeist_text as text;
use text::{ComputedLineBoundaryInput as Cause, SourceBoxSide as Side, SourceProcessingEvent as E};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct Fixture {
    processed: Arc<text::ProcessedSource>,
    binding: TextParagraphBinding<u32>,
    originals: Vec<Source>,
}

fn style() -> text::Style {
    text::Style {
        wrap: Wrap::None,
        ..ahem_style(10.0, 10.0)
    }
}

fn direction(rtl: bool) -> ParagraphBaseDirection {
    if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    }
}

fn base_input(rtl: bool, strut: text::ParagraphStrut) -> text::ContextualSourceInput {
    let originals = [
        Source::identified("A", text::Id::from_u64(650_751), 12),
        Source::identified("A", text::Id::from_u64(650_752), 12),
    ];
    text::ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: Source::identified("AA", text::Id::from_u64(650_750), 12),
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: originals
            .into_iter()
            .enumerate()
            .map(|(i, source)| text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(i, i + 1),
                kind: text::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![text::SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: text::ParagraphConnectionKind::Continuous,
        }],
        controls: vec![],
        boundary_policies: vec![text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::None,
            following_wrap: Wrap::None,
            ancestor_wrap: Wrap::None,
        }],
        replacements: vec![],
        scopes: vec![text::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style(),
            used_direction: direction(rtl),
        }],
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: direction(rtl),
            paragraph_strut: strut,
        }],
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            origin_event(0),
            E::Connection(0),
            origin_event(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    }
}

fn origin_event(origin: usize) -> E {
    E::Origin {
        origin,
        affinity: text::Affinity::After,
    }
}
fn control_event(control: usize) -> E {
    E::Control {
        control,
        affinity: text::Affinity::After,
    }
}

fn scope_input(rtl: bool, side: Side, forced: bool) -> text::ContextualSourceInput {
    let mut input = base_input(rtl, text::ParagraphStrut::DefaultFont);
    input.scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    });
    input.events = match side {
        Side::Before => vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            origin_event(0),
            E::Connection(0),
            E::EnterScope(1),
            origin_event(1),
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
        Side::After => vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::EnterScope(1),
            origin_event(0),
            E::ExitScope(1),
            E::Connection(0),
            origin_event(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    };
    if forced {
        input
            .computed_line_boundaries
            .push(Cause::Scope { scope: 1, side });
    }
    input
}

fn paired_input(rtl: bool, side: Side) -> text::ContextualSourceInput {
    let mut input = base_input(rtl, text::ParagraphStrut::DefaultFont);
    let at = match side {
        Side::Before => (1, 2),
        Side::After => (0, 1),
    };
    input.controls = vec![
        text::ParagraphControl {
            at: at.0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: at.1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    input.scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
    });
    input.events = match side {
        Side::Before => vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            origin_event(0),
            E::Connection(0),
            E::EnterScope(1),
            control_event(0),
            origin_event(1),
            control_event(1),
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
        Side::After => vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::EnterScope(1),
            control_event(0),
            origin_event(0),
            control_event(1),
            E::ExitScope(1),
            E::Connection(0),
            origin_event(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    };
    input.computed_line_boundaries = vec![Cause::Endpoint {
        control: if side == Side::Before { 0 } else { 1 },
    }];
    input
}

fn atomic_input(rtl: bool, side: Side) -> text::ContextualSourceInput {
    let mut input = base_input(rtl, text::ParagraphStrut::DefaultFont);
    input.source.inline_box(text::InlineBox::new(
        text::Id::from_u64(650_753),
        text::InlineBoxKind::InFlow,
        1,
        text::Size::new(10.0, 10.0),
    ));
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin_event(0),
        E::Atomic {
            atomic: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        origin_event(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    input.boundary_policies.extend([
        text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::AtomicBefore(0),
            preceding_wrap: Wrap::None,
            following_wrap: Wrap::None,
            ancestor_wrap: Wrap::None,
        },
        text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::AtomicAfter(0),
            preceding_wrap: Wrap::None,
            following_wrap: Wrap::None,
            ancestor_wrap: Wrap::None,
        },
    ]);
    input.computed_line_boundaries = vec![Cause::Atomic { atomic: 0, side }];
    input
}

fn empty_input(anchors: bool, strut: text::ParagraphStrut) -> text::ContextualSourceInput {
    let mut input = base_input(false, strut);
    input.source = Source::identified("", text::Id::from_u64(650_754), 12);
    input.origins.clear();
    input.connections.clear();
    input.boundary_policies.clear();
    input.scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: anchors.then_some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
    });
    input.events = vec![E::EnterScope(0), E::EnterParagraph(0), E::EnterScope(1)];
    if anchors {
        input.controls = vec![
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ];
        input.events.extend([control_event(0), control_event(1)]);
    }
    input
        .events
        .extend([E::ExitScope(1), E::ExitParagraph(0), E::ExitScope(0)]);
    input.computed_line_boundaries = vec![Cause::Scope {
        scope: 1,
        side: Side::Before,
    }];
    input
}

fn prepare(input: text::ContextualSourceInput) -> Fixture {
    let nodes = input
        .events
        .iter()
        .filter_map(|event| match event {
            E::Origin { origin, .. } => Some(100 + *origin as u32),
            E::Control { control, .. } => Some(200 + *control as u32),
            E::Atomic { atomic, .. } => Some(300 + *atomic as u32),
            _ => None,
        })
        .collect();
    let originals = input
        .origins
        .iter()
        .map(|origin| origin.source.clone())
        .collect();
    let processed = Arc::new(text::process_contextual_source(input).unwrap());
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::None);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, view)| TextOriginBinding {
            origin: view.handle(),
            node: 100 + i as u32,
        })
        .collect();
    let controls = prepared
        .control_views()
        .enumerate()
        .map(|(i, view)| TextControlBinding {
            occurrence: view.handle(),
            node: 200 + i as u32,
        })
        .collect();
    let atomics = prepared
        .atomic_views()
        .enumerate()
        .map(|(i, view)| TextAtomicBinding {
            occurrence: view.handle(),
            node: 300 + i as u32,
        })
        .collect();
    Fixture {
        processed,
        originals,
        binding: TextParagraphBinding {
            prepared,
            container: 0,
            nodes,
            origins,
            controls,
            atomics,
            line_strut: None,
        },
    }
}

fn tree<S: LayoutScalar>(fixture: &Fixture, rtl: bool) -> PublicLayoutTreeOf<S> {
    let direction = if rtl { Direction::Rtl } else { Direction::Ltr };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, fixture.binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction,
                ..NodeInputOf::default()
            },
        );
    for row in &fixture.binding.origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &fixture.binding.controls {
        let kind = match row.occurrence.view().kind() {
            ParagraphControlKind::InlineBoundaryStart => InlineBoundaryKind::Start,
            ParagraphControlKind::InlineBoundaryEnd => InlineBoundaryKind::End,
            _ => panic!("fixture has only genuine endpoint controls"),
        };
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    kind,
                    InlineMetricsOf::try_new(
                        crate::InlineBaselineContext::Physical,
                        S::from_f64(8.0),
                        S::from_f64(10.0),
                    )
                    .unwrap(),
                    InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap()
                .with_direction(direction),
            ),
        );
    }
    for row in &fixture.binding.atomics {
        let size = row.occurrence.view().original_box().size();
        tree = tree.children(row.node, []).style(
            row.node,
            NodeInputOf {
                display: Display::InlineBlock,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(size.width)),
                    PreferredSizeOf::px(S::from_f64(size.height)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn request_at<S: LayoutScalar>(width: AvailableOf<S>) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap()
}

fn compute<S: LayoutScalar>(
    fixture: &Fixture,
    rtl: bool,
    width: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = tree::<S>(fixture, rtl);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&flow, 0, request_at(width)).unwrap()
}

fn finalized<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    lines
        .into_iter()
        .map(|line| line.expect("nonempty original source line retains its real fragment"))
        .collect()
}

fn computed_group(fixture: &Fixture, causes: usize) -> text::PreparedComputedLineBoundary {
    let prepared = &fixture.binding.prepared;
    let groups = prepared
        .opportunity_views()
        .filter_map(|offer| offer.computed_line_boundary())
        .collect::<Vec<_>>();
    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.prepared_text(), fixture.binding.prepared);
    assert_eq!(group.view().causes().len(), causes);
    assert_eq!(
        group.view().paragraph(),
        fixture.processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(
        group.view().domain(),
        fixture.processed.captured_scope(0).unwrap()
    );
    assert!(group.view().preceding_event().is_some());
    assert!(group.view().following_event().is_some());
    for cause in group.view().causes() {
        assert_eq!(cause.paragraph(), group.view().paragraph());
        assert_eq!(cause.domain(), group.view().domain());
    }
    groups[0].clone()
}

fn assert_originals<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, fixture: &Fixture) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from("AA"));
    for (row, original) in fixture.binding.origins.iter().zip(&fixture.originals) {
        assert_eq!(row.origin.view().source(), original);
        assert_eq!(original.revision(), 12);
    }
    assert_eq!(fixture.originals[0].id(), Some(text::Id::from_u64(650_751)));
    assert_eq!(fixture.originals[1].id(), Some(text::Id::from_u64(650_752)));
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let selected = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(2, text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selected.source_text(), &text::Text::from("AA"));
    assert_eq!(selected.plaintext(), &text::Text::from("AA"));
    let mut ranges = Vec::new();
    for line in finalized(batch) {
        assert_eq!(line.prepared_text(), fixture.binding.prepared);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
            assert_eq!(metrics.units_per_em(), 1000);
            assert_eq!(
                (metrics.ascent(), metrics.descent(), metrics.leading()),
                (8.0, -2.0, 0.0)
            );
            for glyph in run.glyphs() {
                assert_eq!((glyph.id(), glyph.advance()), (35, 10.0));
                ranges.push(glyph.range());
            }
        }
    }
    ranges.sort_by_key(|range| range.start);
    assert_eq!(ranges, [text::Range::new(0, 1), text::Range::new(1, 2)]);
}

fn assert_two_line_content<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Fixture,
    advances: [f64; 2],
) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    let rows = finalized(batch);
    assert_eq!(
        rows.iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        advances
    );
    assert_eq!(
        rows[0].selected_line().request().context.end,
        text::LineEnd::Forced
    );
    assert_eq!(
        rows[1].selected_line().request().context.end,
        text::LineEnd::Paragraph
    );
    let selected = rows[0].selected_line();
    let text::PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("issued mandatory ordinary termination")
    };
    assert_eq!(
        offer.view().computed_line_boundary(),
        Some(computed_group(fixture, 1))
    );
    assert_originals(batch, fixture);
}

fn assert_two_lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Fixture,
    advances: [f64; 2],
) {
    assert_two_line_content(batch, fixture, advances);
    for (i, line) in batch.placed_text()[0].layout().lines().iter().enumerate() {
        assert_eq!(line.baseline(), 8.0 + 10.0 * i as f64);
        assert_eq!(line.bounds().origin.y, 10.0 * i as f64);
        assert_eq!(line.bounds().size.height, 10.0);
    }
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.height, S::from_f64(20.0));
}

fn scope_before<S: LayoutScalar>() {
    for rtl in [false, true] {
        let neutral = prepare(scope_input(rtl, Side::Before, false));
        let old = compute::<S>(&neutral, rtl, AvailableOf::definite(S::from_f64(40.0)));
        assert_eq!(old.placed_text()[0].layout().lines().len(), 1);
        assert_eq!(finalized(&old)[0].feedback().advance, 20.0);
        assert_originals(&old, &neutral);
        let forced = prepare(scope_input(rtl, Side::Before, true));
        assert_eq!(forced.binding.prepared.control_views().count(), 0);
        assert_eq!(forced.binding.nodes, [100, 101]);
        let cause = forced.processed.captured_computed_line_boundary(0).unwrap();
        assert_eq!(
            cause.target(),
            text::CapturedComputedLineBoundaryTarget::Scope {
                scope: forced.processed.captured_scope(1).unwrap(),
                side: Side::Before
            }
        );
        let batch = compute::<S>(&forced, rtl, AvailableOf::definite(S::from_f64(40.0)));
        assert_two_lines(&batch, &forced, [10.0, 10.0]);
        let glyphs = batch.placed_text()[0]
            .layout()
            .run_views()
            .flat_map(|run| {
                let x = run.origin().x;
                run.glyphs()
                    .map(move |glyph| x + glyph.pen() + f64::from(glyph.x()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            glyphs,
            if rtl {
                vec![30.0, 30.0]
            } else {
                vec![0.0, 0.0]
            }
        );
        let max = compute::<S>(&forced, rtl, AvailableOf::MAX_CONTENT);
        assert_eq!(
            max.final_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output()
                .size
                .width,
            S::from_f64(10.0)
        );
    }
}

#[test]
fn computed_scope_before_splits_wrap_none_without_inventing_a_control_node() {
    scope_before::<f32>();
    scope_before::<f64>();
}

fn endpoints<S: LayoutScalar>() {
    for rtl in [false, true] {
        for side in [Side::Before, Side::After] {
            let f = prepare(paired_input(rtl, side));
            assert_eq!(f.binding.prepared.control_views().count(), 2);
            assert_eq!(f.binding.controls.len(), 2);
            let control = if side == Side::Before { 0 } else { 1 };
            assert_eq!(
                f.processed
                    .captured_computed_line_boundary(0)
                    .unwrap()
                    .target(),
                text::CapturedComputedLineBoundaryTarget::Endpoint(
                    f.processed.captured_control(control).unwrap()
                )
            );
            let batch = compute::<S>(&f, rtl, AvailableOf::definite(S::from_f64(40.0)));
            assert_two_lines(&batch, &f, [10.0, 10.0]);
            assert_eq!(batch.placed_text()[0].nodes().len(), 4);
        }
    }
}

#[test]
fn computed_endpoint_before_and_after_keep_only_original_endpoint_bindings() {
    endpoints::<f32>();
    endpoints::<f64>();
}

fn atomic_edges<S: LayoutScalar>() {
    for rtl in [false, true] {
        for side in [Side::Before, Side::After] {
            let f = prepare(atomic_input(rtl, side));
            assert_eq!(f.binding.prepared.control_views().count(), 0);
            assert_eq!(f.binding.nodes, [100, 300, 101]);
            assert_eq!(
                f.processed
                    .captured_computed_line_boundary(0)
                    .unwrap()
                    .target(),
                text::CapturedComputedLineBoundaryTarget::Atomic {
                    position: f.processed.captured_atomic(0).unwrap(),
                    side
                }
            );
            assert_eq!(
                f.binding.atomics[0].occurrence.view().level().get(),
                if rtl { 1 } else { 0 }
            );
            let batch = compute::<S>(&f, rtl, AvailableOf::definite(S::from_f64(40.0)));
            assert_two_line_content(
                &batch,
                &f,
                if side == Side::Before {
                    [10.0, 20.0]
                } else {
                    [20.0, 10.0]
                },
            );
            // CSS2 10.8.1: an empty inline-block has its baseline at the
            // bottom margin edge. Its10/0 plus Ahem's8/2 makes a12-high line.
            let geometry = if side == Side::Before {
                [(8.0, 0.0, 10.0), (20.0, 10.0, 12.0)]
            } else {
                [(10.0, 0.0, 12.0), (20.0, 12.0, 10.0)]
            };
            for (line, (baseline, top, height)) in
                batch.placed_text()[0].layout().lines().iter().zip(geometry)
            {
                assert_eq!(line.baseline(), baseline);
                assert_eq!(line.bounds().origin.y, top);
                assert_eq!(line.bounds().size.height, height);
            }
            assert_eq!(
                batch
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output()
                    .size
                    .height,
                S::from_f64(22.0)
            );
            let atom = batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 300)
                .unwrap()
                .output();
            let x = match (rtl, side) {
                (false, Side::Before) => 0.0,
                (false, Side::After) => 10.0,
                // P1 separates at the computed B. N2 resolves the edge ON to
                // R/level1, while A is level2. L2 gives A,box for Before and
                // box,A for After. Start alignment puts the20-wide group at20.
                (true, Side::Before) => 30.0,
                (true, Side::After) => 20.0,
            };
            assert_eq!(
                atom.location,
                Point::new(
                    S::from_f64(x),
                    S::from_f64(if side == Side::Before { 10.0 } else { 0.0 })
                )
            );
            assert_eq!(atom.size, Size::splat(S::from_f64(10.0)));
        }
    }
}

#[test]
fn computed_atomic_edges_split_original_flow_and_keep_the_actual_atomic_geometry() {
    atomic_edges::<f32>();
    atomic_edges::<f64>();
}

fn whole_scope_input(rtl: bool, side: Side) -> text::ContextualSourceInput {
    let mut input = base_input(rtl, text::ParagraphStrut::DefaultFont);
    input.scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    });
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin_event(0),
        E::Connection(0),
        origin_event(1),
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    input.computed_line_boundaries = vec![Cause::Scope { scope: 1, side }];
    input
}

fn empty_edges<S: LayoutScalar>() {
    for rtl in [false, true] {
        for side in [Side::Before, Side::After] {
            let f = prepare(whole_scope_input(rtl, side));
            let group = computed_group(&f, 1);
            assert_eq!(
                group.view().source_position().get(),
                if side == Side::Before { 0 } else { 2 }
            );
            let batch = compute::<S>(&f, rtl, AvailableOf::definite(S::from_f64(40.0)));
            let placed = batch.placed_text()[0].layout();
            assert_eq!(placed.source().text(), &text::Text::from("AA"));
            // The leading actual Forced ending is real even without text.
            // After the trailing force, only the Paragraph receiver is strut-only.
            let expected = if side == Side::Before {
                vec![text::Range::new(0, 0), text::Range::new(0, 2)]
            } else {
                vec![text::Range::new(0, 2)]
            };
            assert_eq!(
                placed
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                placed
                    .glyph_runs()
                    .iter()
                    .map(|run| run.advance())
                    .sum::<f64>(),
                20.0
            );
            for (i, line) in placed.lines().iter().enumerate() {
                assert_eq!(line.baseline(), 8.0 + 10.0 * i as f64);
                assert_eq!(line.bounds().origin.y, 10.0 * i as f64);
                assert_eq!(line.bounds().size.height, 10.0);
            }
            let completions = placed.nonformatted_lines().collect::<Vec<_>>();
            if side == Side::Before {
                assert!(completions.is_empty());
            } else {
                assert_eq!(completions.len(), 1);
                let completion = completions[0].line.finalized_line();
                assert_eq!(completion.prepared_text(), f.binding.prepared);
                assert_eq!(
                    (
                        completion
                            .selected_line()
                            .request()
                            .start
                            .view()
                            .source_before()
                            .get(),
                        completion
                            .selected_line()
                            .request()
                            .end
                            .view()
                            .source_after()
                            .get()
                    ),
                    (2, 2)
                );
                assert!(!completion.has_in_flow_content());
                assert_eq!(completion.feedback().baseline, 8.0);
                assert_eq!(completion.feedback().line_height, 10.0);
                assert_eq!(completions[0].origin.y, 10.0);
                let selected = completion.selected_line();
                let request = selected.request();
                assert_eq!(request.context.end, text::LineEnd::Paragraph);
                assert!(matches!(&request.termination,
                    text::PreparedTermination::Ordinary(offer)
                        if offer.view().kind() == text::PreparedBreakKind::ParagraphEnd));
                assert_eq!(request.start, request.end);
                assert_eq!(request.end.view().source_after().get(), 2);
                assert!(request.participants.is_empty());
            }
            assert_eq!(
                batch
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output()
                    .size
                    .height,
                S::from_f64(if side == Side::Before { 20.0 } else { 10.0 })
            );
        }
    }
}

#[test]
fn leading_computed_force_is_real_and_trailing_strut_only_receiver_is_nonformatted() {
    empty_edges::<f32>();
    empty_edges::<f64>();
}

fn anchored_empty<S: LayoutScalar>() {
    let f = prepare(empty_input(true, text::ParagraphStrut::DefaultFont));
    assert!(f.binding.origins.is_empty());
    assert_eq!(f.binding.nodes, [200, 201]);
    assert_eq!(f.binding.prepared.control_views().count(), 2);
    assert_eq!(computed_group(&f, 1).view().source_position().get(), 0);
    let batch = compute::<S>(&f, false, AvailableOf::definite(S::from_f64(40.0)));
    let placed = batch.placed_text()[0].layout();
    assert!(placed.source().text().is_empty());
    assert!(
        placed
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty() && run.advance() == 0.0)
    );
    // The computed Forced selection is one real 10px line. Its terminal
    // Paragraph selection contains only zero-decoration endpoints and struts.
    assert_eq!(placed.lines().len(), 1);
    let completions = placed.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let completion = completions[0].line.finalized_line();
    assert_eq!(completion.prepared_text(), f.binding.prepared);
    assert_eq!(
        (
            completion
                .selected_line()
                .request()
                .start
                .view()
                .source_before()
                .get(),
            completion
                .selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get()
        ),
        (0, 0)
    );
    assert!(!completion.has_in_flow_content());
    assert_eq!(completion.feedback().baseline, 8.0);
    assert_eq!(completion.feedback().line_height, 10.0);
    assert_eq!(completions[0].origin.y, 10.0);
    let selected = completion.selected_line();
    let request = selected.request();
    assert_eq!(request.context.end, text::LineEnd::Paragraph);
    assert!(matches!(&request.termination,
        text::PreparedTermination::Ordinary(offer)
            if offer.view().kind() == text::PreparedBreakKind::ParagraphEnd));
    assert_eq!(
        request
            .controls
            .iter()
            .map(|row| row.occurrence.clone())
            .collect::<Vec<_>>(),
        f.binding
            .controls
            .iter()
            .map(|row| row.occurrence.clone())
            .collect::<Vec<_>>()
    );
    for (i, line) in placed.lines().iter().enumerate() {
        assert_eq!(line.range(), text::Range::new(0, 0));
        assert_eq!(line.baseline(), 8.0 + 10.0 * i as f64);
        assert_eq!(line.bounds().size.height, 10.0);
    }
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output()
            .size
            .height,
        S::from_f64(10.0)
    );
    // No extra line-break/control node or painted text was fabricated.
    assert_eq!(batch.placed_text()[0].nodes(), &[200, 201]);
}

#[test]
fn computed_only_paragraph_retains_real_endpoints_in_nonformatted_completion() {
    anchored_empty::<f32>();
    anchored_empty::<f64>();
}

#[test]
fn node_free_computed_paragraph_admits_its_real_capture_without_child_nodes() {
    let f = prepare(empty_input(false, text::ParagraphStrut::DefaultFont));
    assert!(f.binding.nodes.is_empty());
    assert!(f.binding.origins.is_empty());
    assert!(f.binding.controls.is_empty());
    let computed = computed_group(&f, 1);
    assert_eq!(computed.view().source_position().get(), 0);
    let tree = tree::<f64>(&f, false);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .expect("actual computed participation admits a semantic paragraph in the unique empty-container gap");
    assert_eq!(flow.paragraphs().len(), 1);
    let actual = &flow.paragraphs()[0];
    assert_eq!(actual.container, f.binding.container);
    assert_eq!(actual.prepared, f.binding.prepared);
    assert_eq!(
        actual
            .prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        vec![f.processed.captured_paragraph(0).unwrap()]
    );
    assert!(actual.nodes.is_empty());
    assert!(actual.origins.is_empty());
    assert!(actual.controls.is_empty());
    assert!(actual.atomics.is_empty());
    // Admission retains the actual issued computed participant, not a native
    // placeholder or a newly prepared source. Final geometry is qualified by
    // the public node-free completion tests with actual box metric witnesses.
    assert!(actual.prepared.participants().any(|participant| {
        matches!(participant, text::PreparedParticipant::ComputedLineBoundary(group) if group == computed)
    }));
}

fn missing_metrics<S: LayoutScalar>() {
    let f = prepare(empty_input(
        true,
        text::ParagraphStrut::AllocatedContributions,
    ));
    let tree = tree::<S>(&f, false);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let error = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .expect_err("a pure computed force supplies no strut or control metrics");
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::Text(error) if matches!(error.detail(), Some(text::ErrorDetail::InvalidTextProtocol { reason: text::ProtocolError::MissingLineMetrics }))),
        "{error:?}"
    );
}

#[test]
fn allocated_metrics_cannot_be_borrowed_from_endpoints_beyond_the_force() {
    missing_metrics::<f32>();
    missing_metrics::<f64>();
}

#[test]
fn foreign_preparation_and_absent_original_target_bindings_refuse_precisely() {
    let first = prepare(scope_input(false, Side::Before, true));
    let next = prepare(scope_input(false, Side::Before, true));
    assert_eq!(first.originals, next.originals);
    assert_ne!(first.binding.prepared, next.binding.prepared);
    let next_tree = tree::<f64>(&next, false);
    let mut foreign = next.binding.clone();
    foreign.origins[0].origin = first.binding.origins[0].origin.clone();
    let error = LayoutFlowInput::try_new(
        &next_tree,
        0,
        vec![foreign],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .err()
    .expect("equal characters/IDs do not authenticate another preparation");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::OriginCoverage
        ))
    ));
    let foreign_tree = next_tree.clone().input(
        100,
        LayoutInputOf::inline_text(TextParticipation::new(
            first.binding.origins[0].origin.clone(),
        )),
    );
    let error = LayoutFlowInput::try_new(
        &foreign_tree,
        0,
        vec![next.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .err()
    .expect("actual host participation must match the issued preparation");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantRole
        ))
    ));
    let endpoint = prepare(paired_input(false, Side::After));
    let endpoint_tree = tree::<f64>(&endpoint, false);
    let mut absent = endpoint.binding.clone();
    absent.controls.clear();
    let error = LayoutFlowInput::try_new(
        &endpoint_tree,
        0,
        vec![absent],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .err()
    .expect("the original target endpoint must retain its native binding");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ControlCoverage
        ))
    ));
    let atomic = prepare(atomic_input(false, Side::Before));
    let atomic_tree = tree::<f64>(&atomic, false);
    let mut absent = atomic.binding.clone();
    absent.atomics.clear();
    let error = LayoutFlowInput::try_new(
        &atomic_tree,
        0,
        vec![absent],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .err()
    .expect("the original target atomic must retain its native binding");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::AtomicCoverage
        ))
    ));
}

fn insert_float<S: LayoutScalar>(fixture: &mut Fixture, paired: bool) -> PublicLayoutTreeOf<S> {
    let at = if paired { 3 } else { 1 };
    fixture.binding.nodes.insert(at, 400);
    tree::<S>(fixture, false).children(400, []).style(
        400,
        NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            ..NodeInputOf::default()
        },
    )
}

fn after_float<S: LayoutScalar>() {
    let mut neutral_input = paired_input(false, Side::After);
    neutral_input.computed_line_boundaries.clear();
    let mut neutral = prepare(neutral_input);
    let neutral_tree = insert_float::<S>(&mut neutral, true);
    let flow = LayoutFlowInput::try_new(
        &neutral_tree,
        0,
        vec![neutral.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let old = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(old.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(finalized(&old)[0].feedback().advance, 20.0);
    assert_eq!(
        old.final_entries()
            .iter()
            .find(|entry| entry.node() == 400)
            .unwrap()
            .output()
            .location,
        Point::ZERO
    );
    assert_originals(&old, &neutral);

    for targets in [
        vec![Cause::Endpoint { control: 1 }],
        vec![Cause::Scope {
            scope: 1,
            side: Side::After,
        }],
        vec![
            Cause::Scope {
                scope: 1,
                side: Side::After,
            },
            Cause::Endpoint { control: 1 },
        ],
    ] {
        let count = targets.len();
        let mut input = paired_input(false, Side::After);
        input.computed_line_boundaries = targets;
        let mut f = prepare(input);
        let group = computed_group(&f, count);
        assert_eq!(group.view().source_position().get(), 1);
        assert_eq!(f.binding.prepared.control_views().count(), 2);
        let tree = insert_float::<S>(&mut f, true);
        let flow = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![f.binding.clone()],
            crate::WrappingInput {
                scopes: vec![],
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &flow,
            0,
            request_at(AvailableOf::definite(S::from_f64(40.0))),
        )
        .unwrap();
        assert_eq!(
            batch.placed_text()[0]
                .layout()
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [text::Range::new(0, 1), text::Range::new(1, 2)]
        );
        let rows = finalized(&batch);
        assert_eq!(
            rows[0].selected_line().request().context.end,
            text::LineEnd::Forced
        );
        let selected = rows[0].selected_line();
        assert_eq!(
            selected.request().participants.last(),
            Some(&text::PreparedParticipant::ComputedLineBoundary(
                group.clone()
            ))
        );
        assert!(
            selected
                .request()
                .participants
                .contains(&text::PreparedParticipant::Control(
                    f.binding.controls[1].occurrence.clone()
                ))
        );
        let float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 400)
            .unwrap()
            .output();
        assert_eq!(float.location, Point::new(S::ZERO, S::from_f64(10.0)));
        assert_eq!(float.size, Size::splat(S::from_f64(10.0)));
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.lines()[0].baseline(), 8.0);
        assert_eq!(placed.lines()[1].baseline(), 18.0);
        let glyphs = placed
            .run_views()
            .flat_map(|run| {
                let origin = run.origin();
                run.glyphs()
                    .map(move |glyph| (origin.x + glyph.pen() + f64::from(glyph.x()), origin.y))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [(0.0, 8.0), (10.0, 18.0)]);
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output()
                .size
                .height,
            S::from_f64(20.0)
        );
        assert_originals(&batch, &f);
    }
}

#[test]
fn float_after_proved_endpoint_or_paired_scope_after_uses_the_computed_seam() {
    after_float::<f32>();
    after_float::<f64>();
}

#[derive(Clone)]
struct Retained<S: LayoutScalar> {
    caches: HashMap<u32, CacheOf<S>>,
    outputs: HashMap<u32, NodeOutputOf<S>>,
    fragments: Vec<InlineFragmentOutputEntryOf<u32, S>>,
}

struct Cached<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    retained: Retained<S>,
    hits: Cell<usize>,
}

impl<S: LayoutScalar> Cached<S> {
    fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            retained: Retained {
                caches: HashMap::new(),
                outputs: HashMap::new(),
                fragments: vec![],
            },
            hits: Cell::new(0),
        }
    }
}

impl<S: LayoutScalar> Traverse for Cached<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        S: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Cached<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let result = self
            .retained
            .caches
            .get(&node)?
            .get_with_context(input, context);
        if result.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.retained.outputs.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Cached<S> {
    type Error = core::convert::Infallible;
    type Prepared = Retained<S>;
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Retained<S>, Self::Error> {
        let mut next = self.retained.clone();
        for entry in batch.cache_clear_entries() {
            next.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            next.caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        for entry in batch.unrounded_entries() {
            next.outputs.insert(entry.node(), entry.output());
        }
        next.fragments = batch.final_inline_fragments().to_vec();
        Ok(next)
    }
    fn commit_layout_batch(&mut self, next: Retained<S>) {
        self.retained = next;
    }
}

fn cached_tree<S: LayoutScalar>(f: &Fixture) -> PublicLayoutTreeOf<S> {
    tree::<S>(f, false)
        .children(1000, [0, 900])
        .children(900, [])
        .style(
            1000,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            900,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(1.0)),
                    PreferredSizeOf::px(S::from_f64(1.0)),
                ),
                ..NodeInputOf::default()
            },
        )
}

fn cached_compute<S: LayoutScalar>(
    tree: &Cached<S>,
    f: &Fixture,
) -> CompletedLayoutBatchOf<u32, S> {
    let flow = LayoutFlowInput::try_new(
        tree,
        1000,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &flow,
        1000,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap()
}

fn changed_cause<S: LayoutScalar>() {
    let old = prepare(scope_input(false, Side::Before, false));
    let current = prepare(scope_input(false, Side::Before, true));
    assert_eq!(old.originals, current.originals);
    let mut warm_tree = Cached::new(cached_tree::<S>(&old));
    let prior = cached_compute(&warm_tree, &old);
    assert_eq!(prior.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(finalized(&prior)[0].feedback().advance, 20.0);
    prior.apply_to(&mut warm_tree).unwrap();
    assert_eq!(warm_tree.retained.fragments, prior.final_inline_fragments());
    warm_tree.tree = cached_tree(&current);
    let warm = cached_compute(&warm_tree, &current);
    let cold_tree = Cached::new(cached_tree::<S>(&current));
    let cold = cached_compute(&cold_tree, &current);
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_two_lines(&warm, &current, [10.0, 10.0]);
    assert_two_lines(&cold, &current, [10.0, 10.0]);
    assert!(warm_tree.hits.get() > 0);
    assert_eq!(cold_tree.hits.get(), 0);
    assert_eq!(prior.placed_text()[0].layout().lines().len(), 1);
    assert_originals(&prior, &old);
}

#[test]
fn changed_computed_cause_rebuilds_warm_output_and_preserves_old_result() {
    changed_cause::<f32>();
    changed_cause::<f64>();
}

fn unproved_float<S: LayoutScalar>() {
    let mut neutral = prepare(scope_input(false, Side::After, false));
    let neutral_tree = insert_float::<S>(&mut neutral, false);
    let mut host = Cached::new(neutral_tree);
    let flow = LayoutFlowInput::try_new(
        &host,
        0,
        vec![neutral.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let old = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(old.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(
        old.final_entries()
            .iter()
            .find(|entry| entry.node() == 400)
            .unwrap()
            .output()
            .location
            .y,
        S::ZERO
    );
    old.apply_to(&mut host).unwrap();
    let outputs = host.retained.outputs.clone();
    let fragments = host.retained.fragments.clone();

    let mut requested = prepare(scope_input(false, Side::After, true));
    let scope = requested.processed.captured_scope(1).unwrap();
    assert!(scope.entry_control().is_none());
    assert!(scope.exit_control().is_none());
    assert!(requested.binding.controls.is_empty());
    assert!(requested.binding.atomics.is_empty());
    let group = computed_group(&requested, 1);
    assert_eq!(group.view().source_position().get(), 1);
    host.tree = insert_float::<S>(&mut requested, false);
    // The native float is genuinely between the two original text nodes, but
    // has no captured event or paired endpoint proving its side of ScopeAfter.
    // Equal source1 cannot choose the preceding physical seam for this force.
    let error = match LayoutFlowInput::try_new(
        &host,
        0,
        vec![requested.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    ) {
        Ok(flow) => compute_layout_with_flow(
            &flow,
            0,
            request_at(AvailableOf::definite(S::from_f64(40.0))),
        )
        .expect_err("unproved semantic/native side cannot publish a batch"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::MissingContext(LayoutMissingContext::ComputedLineBoundaryIncidence)
        ),
        "{error:?}"
    );
    assert_eq!(host.retained.outputs, outputs);
    assert_eq!(host.retained.fragments, fragments);
    assert_eq!(old.placed_text()[0].layout().lines().len(), 1);
    assert_originals(&old, &neutral);
}

#[test]
fn unrepresented_float_at_semantic_only_scope_after_requires_real_side_incidence() {
    unproved_float::<f32>();
    unproved_float::<f64>();
}
