//! Actual draft/bound closing plans retain live custody and cumulative charges.
use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::text_layout::scope_alignment::{
    EntryFunding, ScopeAlignmentFacts, ScopeAlignmentResources, ScopeAlignmentSnapshot,
};
use crate::*;
use std::cell::Cell;
use surgeist_text as text;

const LIMITS: text::ScopeAlignmentLimits =
    text::ScopeAlignmentLimits::new(100_000, 100, 1_000_000, 16 << 20);

thread_local! {
    static MEASUREMENTS: Cell<usize> = const { Cell::new(0) };
}

fn measure(_: LeafMeasureInputOf<f64>) -> Size<f64> {
    MEASUREMENTS.with(|calls| calls.set(calls.get() + 1));
    Size::splat(10.0)
}

fn owner(binding: &TextParagraphBinding<u32>) -> ScopeAlignmentResources<f64> {
    ScopeAlignmentResources::new(
        ScopeAlignmentSnapshot::admit(vec![], std::slice::from_ref(binding), false)
            .unwrap()
            .freeze(),
        Default::default(),
    )
}

fn with_scene(
    run: impl FnOnce(
        &LayoutFlowInput<'_, PublicLayoutTreeOf<f64>>,
        &Arc<crate::engine::FloatChildPacket<u32, f64>>,
        ComputeInputOf<f64>,
        &[text::ControlAllocation],
        &text::FinalizedLine,
    ),
) {
    let fixture = Fixture::with_scope_limits("X", LIMITS);
    let mut binding = fixture.binding.clone();
    binding.nodes.insert(2, 2);
    assert_eq!(binding.nodes, [20, 10, 2, 21]);
    let scope = binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .captured_control()
        .unwrap()
        .scope();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = fixture
        .tree::<f64>(
            InlineBoundaryEdgeOf::ZERO,
            InlineBoundaryEdgeOf::ZERO,
            false,
            Some(40.0),
            false,
        )
        .children(0, binding.nodes.iter().copied())
        .children(2, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                min_size: Size::new(MinSizeOf::px(0.0), MinSizeOf::px(0.0)),
                ..Default::default()
            },
        )
        .measure_with(2, measure);
    for (node, kind, padding) in [
        (20, InlineBoundaryKind::Start, 0.0),
        (21, InlineBoundaryKind::End, 3.0),
    ] {
        tree = tree.input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, 8.0, 10.0).unwrap(),
                InlineBoundaryEdgeOf::try_new(0.0, 0.0, padding).unwrap(),
                axes,
            )),
        );
    }
    let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope.clone()));
    let child = |box_, participation| GeneratedChildBoxInput {
        box_,
        participation,
    };
    let flow = |box_| child(box_, GeneratedBoxParticipation::InFlow);
    let order = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let wrapping = WrappingInput {
        scopes: vec![WrappingScopeInput::Captured {
            scope,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            order(GeneratedBoxRef::Node(0), vec![flow(inline.clone())]),
            order(
                inline,
                vec![
                    flow(GeneratedBoxRef::Node(10)),
                    child(
                        GeneratedBoxRef::Node(2),
                        GeneratedBoxParticipation::OutOfFlow,
                    ),
                ],
            ),
        ],
        ..Default::default()
    };
    let capture = LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(40.0).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    MEASUREMENTS.with(|calls| calls.set(0));
    // The selected receipt comes from actual public layout. No line, replay
    // frame, output or band is supplied to the producer by this resource test.
    let batch = compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::standalone(sizing, axes, Size::NONE),
    )
    .unwrap();
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.selected_line().request().controls.iter().any(|row| {
                matches!(
                    row.occurrence.view().kind(),
                    text::ParagraphControlKind::InlineBoundaryEnd
                )
            })
        })
        .expect("actual End3 line receipt");
    assert_eq!(line.feedback().advance, 13.0);
    let mut computation = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    let request = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(40.0), None),
        ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
        sizing,
    );
    let packet = computation
        .probe_float_child(2, request, crate::engine::FloatChildTrial::Detached)
        .unwrap();
    assert_eq!(packet.output.size, Size::splat(10.0));
    MEASUREMENTS.with(|calls| assert!(calls.get() > 0));
    let allocation_resources = owner(&capture.paragraphs[0]);
    let allocator = super::super::inline_run::BoundaryAllocationContext {
        container: 0,
        prepared: &capture.paragraphs[0].prepared,
        associations: &capture.associations,
        resources: &allocation_resources,
        orientation: capture.paragraphs[0].prepared.orientation(),
        axes,
    };
    let controls = capture.paragraphs[0]
        .controls
        .iter()
        .map(|row| {
            let LayoutInputViewOf::InlineBoundary(input) = computation.layout_input(row.node)
            else {
                panic!("actual paired endpoint input");
            };
            let boundary = allocator
                .allocate(
                    &computation,
                    row.node,
                    &row.occurrence,
                    crate::inline::InlineEndpointOf::from_input(input),
                )
                .unwrap();
            assert!(boundary.box_context().is_some());
            assert_eq!(boundary.advance(), if row.node == 21 { 3.0 } else { 0.0 });
            text::ControlAllocation {
                occurrence: row.occurrence.clone(),
                boundary: Some(boundary),
                strut: Some(text::ControlStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                    alignment: text::LineBoxAlignment::Baseline,
                }),
            }
        })
        .collect::<Vec<_>>();
    // All producer sizing, public line selection and physical endpoint
    // allocation have completed before the tested plan's baseline is taken.
    run(&capture, &packet, request, &controls, &line);
}

fn event(
    capture: &LayoutFlowInput<'_, PublicLayoutTreeOf<f64>>,
    packet: &Arc<crate::engine::FloatChildPacket<u32, f64>>,
    request: ComputeInputOf<f64>,
    funding: &EntryFunding<f64>,
) -> FloatEvent<u32, f64> {
    let [row] = capture.associations.runs() else {
        panic!("one actual admitted run")
    };
    let admitted = capture.associations.admitted_run(&capture.paragraphs, row);
    let output = packet.output;
    FloatEvent::new(
        PendingFloat {
            placement: super::super::floats::PendingPlacement::Unresolved,
            node: 2,
            source_index: 2,
            side: Float::Left,
            clear: Clear::None,
            block_start: 0.0,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            float_exclusion: FloatExclusion::MarginBox,
            child_compute_geometry: output.scroll_geometry,
        },
        packet.clone(),
        &admitted,
        &capture.associations,
        2,
        request,
        Some(funding.as_ref()),
    )
    .unwrap_or_else(|_| panic!("the funded real float source has an exact seam"))
}

fn bind(
    draft: FloatEvents<u32, f64>,
    capture: &LayoutFlowInput<'_, PublicLayoutTreeOf<f64>>,
    controls: &[text::ControlAllocation],
    funding: &EntryFunding<f64>,
) -> Result<BoundEvents<u32, f64>, FitError> {
    let [row] = capture.associations.runs() else {
        panic!("one actual admitted run")
    };
    let admitted = capture.associations.admitted_run(&capture.paragraphs, row);
    draft.bind(
        &admitted,
        &capture.associations,
        controls.iter(),
        |owner| {
            super::source_interval(
                &admitted,
                &capture.associations,
                &capture.wrapping,
                &capture.graph,
                owner,
                Some(funding.as_ref()),
            )
        },
        Some(funding.as_ref()),
    )
}

fn structural_limit(error: FitError) {
    let FitError::Text(error) = error else {
        panic!("actual captured resource refusal")
    };
    assert!(
        matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
        reason: text::ScopeAlignmentError::Limit {
            kind: text::ScopeAlignmentLimitKind::StructuralWork, required, limit,
        },
    }) if required > limit && *limit == LIMITS.structural_work())
    );
}

fn remaining_work(funding: &EntryFunding<f64>) -> usize {
    for count in 0..=LIMITS.structural_work() {
        match funding.visit(1) {
            Ok(()) => {}
            Err(error) => {
                structural_limit(FitError::Text(error));
                return count;
            }
        }
    }
    panic!("captured finite structural-work ceiling");
}

fn remaining_records(resources: &ScopeAlignmentResources<f64>) -> usize {
    let mut rows = Vec::<()>::new();
    for _ in 0..=LIMITS.records() {
        // These rows borrow no Text product; the owned snapshot is charged.
        if let Err(error) = resources.materialize(LIMITS, 0, 1) {
            assert!(
                matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::Limit {
                    kind: text::ScopeAlignmentLimitKind::Records, required, limit,
                },
            }) if required > limit && *limit == LIMITS.records())
            );
            return rows.len();
        }
        rows.push(());
    }
    panic!("captured finite materialization ceiling");
}

#[test]
fn draft_and_bound_closing_plans_release_live_backing_without_refunding_usage() {
    with_scene(|capture, packet, request, controls, line| {
        let spent = owner(&capture.paragraphs[0]);
        let fresh = owner(&capture.paragraphs[0]);
        let funding = EntryFunding::new(spent.clone(), capture.paragraphs[0].prepared.clone())
            .unwrap()
            .unwrap();
        let fresh_funding =
            EntryFunding::new(fresh.clone(), capture.paragraphs[0].prepared.clone())
                .unwrap()
                .unwrap();
        let initial = spent.retention();
        let calls = MEASUREMENTS.with(Cell::get);
        let mut draft = FloatEvents::new();
        draft
            .push(
                event(capture, packet, request, &funding),
                Some(funding.as_ref()),
            )
            .unwrap_or_else(|_| panic!("funded draft admission"));
        assert!(spent.retention().payload_bytes() > initial.payload_bytes());
        let bound = bind(draft, capture, controls, &funding)
            .unwrap_or_else(|_| panic!("actual immediate End3 binds"));
        assert_eq!(bound.len(), 1);
        assert!(spent.retention().payload_bytes() > initial.payload_bytes());
        let observation = AcceptedFloatPrefix::Seam(bound[0].source.seam.clone());
        let demand = bound[0]
            .closing_ends
            .demand(line, &observation, Some(funding.as_ref()))
            .unwrap_or_else(|_| panic!("the real selected End supplies demand"));
        assert_eq!(demand.extra, 3.0);
        assert!(demand.participates);
        assert_eq!(
            MEASUREMENTS.with(Cell::get),
            calls,
            "binding does not repeat measured float production"
        );
        drop(bound);
        assert_eq!(spent.retention(), initial);
        assert!(remaining_work(&funding) < remaining_work(&fresh_funding));
        assert!(remaining_records(&spent) < remaining_records(&fresh));
    });
}

#[test]
fn exhausted_closing_plan_construction_refuses_and_releases_draft_custody() {
    with_scene(|capture, packet, request, controls, _| {
        let spent = owner(&capture.paragraphs[0]);
        let funding = EntryFunding::new(spent.clone(), capture.paragraphs[0].prepared.clone())
            .unwrap()
            .unwrap();
        let initial = spent.retention();
        let calls = MEASUREMENTS.with(Cell::get);
        let mut draft = FloatEvents::new();
        draft
            .push(
                event(capture, packet, request, &funding),
                Some(funding.as_ref()),
            )
            .unwrap_or_else(|_| panic!("funded draft admission"));
        let unbound = event(capture, packet, request, &funding);
        remaining_work(&funding);
        let Err(error) = bind(draft, capture, controls, &funding) else {
            panic!("spent work must refuse bound-plan construction");
        };
        structural_limit(error);
        assert_eq!(
            spent.retention(),
            initial,
            "refused bind retains no draft or scratch backing"
        );
        let mut retry = FloatEvents::new();
        let Err(error) = retry.push(unbound, Some(funding.as_ref())) else {
            panic!("spent work must refuse draft storage");
        };
        structural_limit(error);
        drop(retry);
        assert_eq!(spent.retention(), initial);
        assert_eq!(
            MEASUREMENTS.with(Cell::get),
            calls,
            "refusal does not repeat prior producer effects"
        );
    });
}
