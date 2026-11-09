use super::*;
use crate::engine::ComputeSession;
use crate::geometry::{LogicalEdgesOf, LogicalPointOf};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use std::cell::Cell;

#[path = "funding_tests.rs"]
mod funding;

#[path = "receipt_binding_tests.rs"]
mod receipt_binding;

#[path = "progress_tests.rs"]
mod progress;

thread_local! {
    static CALLS: Cell<usize> = const { Cell::new(0) };
}

fn leaf<S: LayoutScalar>(height: f64) -> Size<S> {
    CALLS.with(|calls| calls.set(calls.get() + 1));
    Size::new(S::from_f64(100.0), S::from_f64(height))
}

fn zero<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    leaf(0.0)
}

fn positive<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    leaf(2.0)
}

fn block<S: LayoutScalar>(top: f64, bottom: f64, height: Option<f64>) -> crate::NodeInputOf<S> {
    crate::NodeInputOf {
        display: crate::Display::Block,
        size: Size::new(
            crate::PreferredSizeOf::px(S::from_f64(100.0)),
            height.map_or(crate::PreferredSizeOf::AUTO, |height| {
                crate::PreferredSizeOf::px(S::from_f64(height))
            }),
        ),
        margin: Edges::new(
            crate::LengthAutoOf::px(S::from_f64(top)),
            crate::LengthAutoOf::ZERO,
            crate::LengthAutoOf::px(S::from_f64(bottom)),
            crate::LengthAutoOf::ZERO,
        ),
        ..crate::NodeInputOf::default()
    }
}

fn query<S: LayoutScalar>(purpose: EntryPurpose) -> EntryQuery<u32, S> {
    let axes = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr);
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        crate::SizingMode::InherentSize,
        crate::RequestedAxis::Both,
        Size::new(Some(S::from_f64(100.0)), None),
        Size::splat(Some(S::from_f64(100.0))),
        crate::ContainingLayoutContext::new(axes, crate::ParentFormattingContext::BlockFlow),
        crate::SizingRequestOf::new(
            Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
            ),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    let incoming = (purpose == EntryPurpose::FramedEntry).then(|| {
        let ledger = FloatExclusions::new(
            axes,
            Size::splat(S::from_f64(100.0)),
            S::from_f64(100.0),
            LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
        );
        float_context::Incoming {
            cut: None,
            node: 0,
            request: input,
            frame: std::sync::Arc::new(
                ledger.for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO)),
            ),
        }
    });
    EntryQuery {
        node: 0,
        input,
        reporting_side: PhysicalSide::Top,
        purpose,
        incoming,
        position: None,
    }
}

fn opaque<S: LayoutScalar>() {
    for (measure, expected) in [
        (
            zero::<S> as fn(crate::LeafMeasureInputOf<S>) -> Size<S>,
            29.0,
        ),
        (positive::<S>, 5.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(0, block::<S>(0.0, 0.0, None))
            .style(1, block(5.0, 7.0, None))
            .style(2, block(29.0, 0.0, Some(20.0)))
            .measure_with(1, measure);
        let mut owner = ComputeSession::new(&tree, Vec::new());
        CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            owner.qualify_entry(query(EntryPurpose::StartOnly)).unwrap(),
            EntryObservation::Demand(EntryDemand {
                kind: EntryFrontier::AmbiguousStart,
                ..
            })
        ));
        CALLS.with(|calls| assert_eq!(calls.get(), 0));
        let EntryObservation::Summary(summary) = owner
            .qualify_entry(query(EntryPurpose::FramedEntry))
            .unwrap()
        else {
            panic!("actual opaque leaf custody completes this entry component");
        };
        assert_eq!(summary.query.node, 0);
        assert_eq!(summary.start_fact().margin.resolve(), S::from_f64(expected));
        assert!(matches!(summary.through, Through::Blocked));
        CALLS.with(|calls| assert_eq!(calls.get(), 1));
        assert!(owner.staged_unrounded(1).is_err());
        assert!(owner.staged_unrounded(2).is_err());
    }
}

#[test]
fn framed_entry_resolves_real_opaque_leaf_without_completing_ancestor_f32() {
    opaque::<f32>();
}

#[test]
fn framed_entry_resolves_real_opaque_leaf_without_completing_ancestor_f64() {
    opaque::<f64>();
}

#[test]
fn framed_entry_suspends_before_unqualified_clear_top_and_later_body() {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, block::<f64>(0.0, 0.0, None))
        .style(1, block(8.0, 12.0, None))
        .style(
            2,
            crate::NodeInputOf {
                clear: crate::Clear::Left,
                ..block(20.0, 0.0, None)
            },
        )
        .measure_with(2, positive::<f64>);
    let mut owner = ComputeSession::new(&tree, Vec::new());
    CALLS.with(|calls| calls.set(0));
    let EntryObservation::Demand(demand) = owner
        .qualify_entry(query(EntryPurpose::FramedEntry))
        .unwrap()
    else {
        panic!("an authored offer requires its canonical clearance demand");
    };
    assert_eq!(demand.kind, EntryFrontier::Clearance);
    assert_eq!(demand.node, 0);
    let frame = demand.driver.frames.last().unwrap();
    assert_eq!(frame.children[frame.next - 1], 2);
    assert_eq!(frame.start.resolve(), 12.0);
    CALLS.with(|calls| assert_eq!(calls.get(), 0));
    assert!(owner.staged_unrounded(1).is_err());
}

#[test]
fn framed_entry_requires_the_exact_target_request_before_sampling() {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .style(0, block::<f64>(0.0, 0.0, None))
        .style(1, block(5.0, 7.0, None))
        .measure_with(1, zero::<f64>);
    let mut owner = ComputeSession::new(&tree, Vec::new());
    for absent in [true, false] {
        let mut request = query(EntryPurpose::FramedEntry);
        if absent {
            request.incoming = None;
        } else {
            request.incoming.as_mut().unwrap().request =
                request.input.with_run_mode(RunMode::PerformLayout);
        }
        CALLS.with(|calls| calls.set(0));
        let error = owner.qualify_entry(request).err().unwrap();
        assert!(matches!(
            error.kind(),
            crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis)
        ));
        CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }
}

#[test]
fn numeric_frontier_retains_the_actual_source_range_without_placing_it() {
    let atomic = crate::NodeInputOf {
        display: crate::Display::InlineBlock,
        ..block::<f64>(0.0, 0.0, Some(0.0))
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 4])
        .style(0, block(0.0, 0.0, None))
        .style(1, atomic.clone())
        .style(
            2,
            crate::NodeInputOf {
                display: crate::Display::None,
                ..block(0.0, 0.0, None)
            },
        )
        .style(3, atomic)
        .style(4, block(20.0, 0.0, Some(10.0)));
    let mut owner = ComputeSession::new(&tree, Vec::new());
    let EntryObservation::Demand(demand) = owner
        .qualify_entry(query(EntryPurpose::FramedEntry))
        .unwrap()
    else {
        panic!("numeric content requires its actual participation frontier");
    };
    assert_eq!(demand.kind, EntryFrontier::NativeRun);
    let window = demand.window.as_ref().unwrap();
    assert_eq!(window.nodes, 0..3);
    assert!(window.segment.is_none());
    for node in 1..=4 {
        assert!(owner.staged_unrounded(node).is_err());
    }
}

#[test]
fn hidden_break_does_not_create_a_native_frontier_in_framed_entry() {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .style(0, block::<f64>(0.0, 0.0, None))
        .input(
            1,
            crate::LayoutInputOf::line_break(crate::LineBreakInputOf::new().hidden()),
        )
        .style(2, block(29.0, 0.0, Some(20.0)));
    let mut owner = ComputeSession::new(&tree, Vec::new());
    let EntryObservation::Summary(summary) = owner
        .qualify_entry(query(EntryPurpose::FramedEntry))
        .unwrap()
    else {
        panic!("a hidden break supplies no participating frontier");
    };
    assert_eq!(summary.start_fact().margin.resolve(), 29.0);
    assert!(owner.staged_unrounded(1).is_err());
    assert!(owner.staged_unrounded(2).is_err());
}

#[test]
fn native_frontier_keeps_issued_section_custody_without_placing_source() {
    use crate::test_support::text::{PreparedParagraph, ahem_style};
    let paragraph =
        PreparedParagraph::plain(10, surgeist_text::Source::new("X"), ahem_style(4.0, 4.0));
    let foreign =
        PreparedParagraph::plain(10, surgeist_text::Source::new("X"), ahem_style(4.0, 4.0));
    let tree = PublicLayoutTreeOf::new()
        .children(0, [10])
        .style(0, block::<f64>(0.0, 0.0, None))
        .input(10, paragraph.text_input(10));
    let capture = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut owner = ComputeSession::with_flow(&capture, Vec::new());
    let expected = owner
        .text_computation()
        .unwrap()
        .run(0, &[10])
        .unwrap()
        .segment
        .clone();
    let EntryObservation::Demand(mut demand) = owner
        .qualify_entry(query(EntryPurpose::FramedEntry))
        .unwrap()
    else {
        panic!("the actual issued window must remain a native demand");
    };
    assert_eq!(demand.kind, EntryFrontier::NativeRun);
    let window = demand.window.as_ref().unwrap();
    assert_eq!(window.nodes, 0..1);
    assert!(
        window
            .segment
            .as_ref()
            .is_some_and(|segment| *segment == expected)
    );
    assert!(owner.staged_unrounded(10).is_err());
    // A different issued source window cannot replace this demand's section,
    // including when both preparations spell the same source text.
    let foreign_tree = PublicLayoutTreeOf::new()
        .children(0, [10])
        .style(0, block::<f64>(0.0, 0.0, None))
        .input(10, foreign.text_input(10));
    let foreign_capture = crate::LayoutFlowInput::try_new(
        &foreign_tree,
        0,
        vec![foreign.binding(0)],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut foreign_owner = ComputeSession::with_flow(&foreign_capture, Vec::new());
    demand.window.as_mut().unwrap().segment = Some(
        foreign_owner
            .text_computation()
            .unwrap()
            .run(0, &[10])
            .unwrap()
            .segment
            .clone(),
    );
    let error = demand.validate_window(&mut owner).unwrap_err();
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis)
    ));
}

fn component<S: LayoutScalar>(left: f64, right: f64, own_top: f64) {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 6, 2])
        .children(2, [3, 4])
        .style(0, block(0.0, 0.0, Some(100.0)))
        .style(
            1,
            crate::NodeInputOf {
                float: crate::Float::Left,
                size: Size::new(
                    crate::PreferredSizeOf::px(S::from_f64(20.0)),
                    crate::PreferredSizeOf::px(S::from_f64(left)),
                ),
                ..block(0.0, 0.0, None)
            },
        )
        .style(
            6,
            crate::NodeInputOf {
                float: crate::Float::Right,
                size: Size::new(
                    crate::PreferredSizeOf::px(S::from_f64(20.0)),
                    crate::PreferredSizeOf::px(S::from_f64(right)),
                ),
                ..block(0.0, 0.0, None)
            },
        )
        .style(2, block(own_top, 0.0, None))
        .style(
            3,
            crate::NodeInputOf {
                clear: crate::Clear::Left,
                ..block(8.0, 12.0, None)
            },
        )
        .style(
            4,
            crate::NodeInputOf {
                clear: crate::Clear::Right,
                ..block(20.0, 0.0, Some(10.0))
            },
        );
    let mut owner = ComputeSession::new(&tree, Vec::new());
    let mut request = query(EntryPurpose::FramedEntry);
    let parent = block_container_projection::<_, core::convert::Infallible>(&owner, 0);
    let constants = Constants::from_projection::<_, core::convert::Infallible>(
        &owner,
        0,
        &parent,
        request.input,
        Size::NONE,
    )
    .unwrap();
    let axes = constants.flow_axes;
    let mut ledger = FloatExclusions::new(
        axes,
        Size::splat(S::from_f64(100.0)),
        S::from_f64(100.0),
        LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    for (source_index, node) in [1, 6].into_iter().enumerate() {
        let style = BlockChildProjection::from_node(owner.node_input(node));
        let mut first = true;
        let prepared = in_flow::prepare_child(
            &owner,
            node,
            &style,
            &constants,
            in_flow::ChildSizingContext {
                inner_size: constants.node_inner_size,
                request: constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: RequestedAxis::Both,
                first_in_flow: &mut first,
            },
        )
        .unwrap();
        let output = owner.compute_child(node, prepared.input).unwrap();
        assert_eq!(output.size.width, S::from_f64(20.0));
        assert_eq!(
            output.size.height,
            S::from_f64(if node == 1 { left } else { right })
        );
        let pending = PendingFloat {
            placement: super::super::floats::PendingPlacement::Unresolved,
            node,
            source_index,
            side: style.float,
            clear: style.clear,
            block_start: S::ZERO,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            border: prepared.border,
            padding: prepared.padding,
            margin: Edges::all(S::ZERO),
            float_exclusion: style.float_exclusion,
            child_compute_geometry: output.scroll_geometry,
        };
        let placement = ledger
            .place_float_for_line::<core::convert::Infallible>(0, &pending, None)
            .unwrap();
        assert_eq!(placement.logical.block, S::ZERO);
    }
    request.node = 2;
    request.purpose = EntryPurpose::Component;
    request.incoming = Some(float_context::Incoming {
        cut: None,
        node: 2,
        request: request.input,
        frame: std::sync::Arc::new(
            ledger.for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO)),
        ),
    });
    request.position = Some(super::super::canonical::EntryPosition {
        cursor: S::ZERO,
        active: CollapsibleMarginOf::ZERO,
        start_open: false,
        collapses_start: false,
        child_adjoins_parent: true,
        own_start: CollapsibleMarginOf::ZERO.collapse_with_margin(S::from_f64(own_top)),
        inline_offset: S::ZERO,
    });
    let EntryObservation::Summary(summary) = owner.qualify_entry(request).unwrap() else {
        panic!("actual box-only prefix has a complete source-cut basis");
    };
    if own_top >= left.max(right) {
        // An ancestor's own top belongs to the complete counterfactual group;
        // qualifying the child before that group would introduce a false cut.
        assert!(summary.separation.is_none());
        assert_eq!(summary.start_fact().margin.resolve(), S::from_f64(20.0));
    } else {
        let cut = summary.separation.unwrap();
        assert_eq!(cut.node, 3);
        assert_eq!(cut.container, 2);
        assert_eq!(cut.index, 0);
        assert_eq!(cut.hypothetical.get(), S::from_f64(12.0));
        assert_eq!(cut.relevant_bottom.get(), S::from_f64(left));
        assert_eq!(summary.start_fact().margin.resolve(), S::ZERO);
    }
    assert!(owner.staged_unrounded(2).is_err());
    assert!(owner.staged_unrounded(3).is_err());
    assert!(owner.staged_unrounded(4).is_err());
}

#[test]
fn component_cuts_retain_the_original_ancestor_basis_f32() {
    for right in [25.0, 27.0, 30.0] {
        component::<f32>(15.0, right, 0.0);
    }
    component::<f32>(30.0, 30.0, 40.0);
}

#[test]
fn component_cuts_retain_the_original_ancestor_basis_f64() {
    for right in [25.0, 27.0, 30.0] {
        component::<f64>(15.0, right, 0.0);
    }
    component::<f64>(30.0, 30.0, 40.0);
}

fn native_samples<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .style(0, block::<S>(0.0, 0.0, None))
        .style(1, block(5.0, 7.0, None))
        .style(2, block(29.0, 0.0, Some(20.0)))
        .measure_with(1, zero::<S>);
    CALLS.with(|calls| calls.set(0));
    let mut owner = ComputeSession::new(&tree, Vec::new());
    for _ in 0..2 {
        let EntryObservation::Summary(summary) = owner
            .qualify_entry(query(EntryPurpose::FramedEntry))
            .unwrap()
        else {
            panic!("exact native sample resumes its component");
        };
        assert_eq!(summary.start_fact().margin.resolve(), S::from_f64(29.0));
        CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }
    // The independent publication operation samples anew: the entry memo is
    // neither host-cache geometry nor a persistent provider result cache.
    let mut next = ComputeSession::new(&tree, Vec::new());
    let EntryObservation::Summary(summary) = next
        .qualify_entry(query(EntryPurpose::FramedEntry))
        .unwrap()
    else {
        panic!("new operation owns its own native observation");
    };
    assert_eq!(summary.start_fact().margin.resolve(), S::from_f64(29.0));
    CALLS.with(|calls| assert_eq!(calls.get(), 2));
}

#[test]
fn exact_native_entry_sample_is_operation_local_f32() {
    native_samples::<f32>();
}
#[test]
fn exact_native_entry_sample_is_operation_local_f64() {
    native_samples::<f64>();
}
