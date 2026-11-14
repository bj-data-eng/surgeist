use super::*;
use crate::block::entry::{EntryObservation, EntryQuery};
use crate::block::{BlockChildRequest, begin_block_computation};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use std::cell::{Cell, RefCell};
use std::ops::ControlFlow;
use std::sync::Arc;

thread_local! {
    static FAIL: Cell<bool> = const { Cell::new(false) };
    static PUBLISHING: Cell<bool> = const { Cell::new(false) };
    static KNOWN: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static CALLS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}
fn assert_request<S: LayoutScalar>(input: LeafMeasureInputOf<S>, width: f64) {
    // The measurement owner forwards known content dimensions for ComputeSize,
    // but deliberately requests a full sample for actual performed layout.
    // Both retain the same finite Normal content offer and automatic block.
    let known = input.known_content_size();
    let sized = known != Size::NONE;
    if sized {
        assert_eq!(known, Size::new(Some(S::from_f64(width)), None));
    } else {
        assert!(
            PUBLISHING.get(),
            "qualification must retain its known inline size"
        );
    }
    assert_eq!(
        input.content_sizing_request().axes(),
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(width)).unwrap()),
                SizingOperation::Normal
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal)
        )
    );
    KNOWN.with(|requests| requests.borrow_mut().push(sized));
}
fn measured<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
    assert_request(input, 60.0);
    CALLS.with(|calls| calls.borrow_mut().push(3));
    Size::new(
        S::from_f64(60.0),
        if FAIL.get() { S::NAN } else { S::from_f64(7.0) },
    )
}
fn tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [3])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .measure_with(3, measured::<S>)
}
fn input<S: LayoutScalar>(width: f64) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(width)), None),
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(width)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}
fn with_session<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    run: impl FnOnce(&mut ComputeSession<'_, PublicLayoutTreeOf<S>>),
) {
    let (graph, invalidated) =
        crate::engine::validation::validate_layout_request_with_text_graph(tree, 0, &[], &[])
            .unwrap();
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        &graph,
        &WrappingInput::default(),
        &[],
        |_| false,
    )
    .unwrap();
    let mut session = ComputeSession::new(tree, invalidated);
    session.install_wrapping(Arc::new(wrapping), &graph);
    run(&mut session);
}
// Ask the actual Block preparer for its authenticated entry demand. No test
// invents a frame, resident cut, source position or child sizing request.
fn query<S: LayoutScalar>(
    session: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
    width: f64,
) -> EntryQuery<u32, S> {
    let ControlFlow::Continue(mut block) =
        crate::engine::block_scope::ordinary(begin_block_computation(session, 0, input(width)))
            .unwrap()
    else {
        panic!("wrapper needs its body")
    };
    let ControlFlow::Continue(child) =
        crate::engine::block_scope::ordinary(block.advance(session)).unwrap()
    else {
        panic!("wrapper needs entry facts")
    };
    let (BlockChildRequest::Entry(query), _) = child.into_request() else {
        panic!("shared wrapper qualifies before its body")
    };
    *query
}
fn qualify<S: LayoutScalar>(
    session: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
    width: f64,
) -> SessionResult<PublicLayoutTreeOf<S>, EntryObservation<u32, S>> {
    let request = query(session, width);
    session.qualify_entry(request)
}
fn discarded<S: LayoutScalar>(session: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>) {
    assert!(session.outputs.unrounded.is_empty());
    assert!(session.outputs.unrounded_fragments.is_empty());
    assert!(session.cache.take_stores().is_empty());
    assert!(session.cache.clears.is_empty());
}
fn reused<S: LayoutScalar>() {
    FAIL.set(false);
    PUBLISHING.set(false);
    KNOWN.with(|requests| requests.borrow_mut().clear());
    CALLS.with(|calls| calls.borrow_mut().clear());
    let tree = tree::<S>();
    with_session(&tree, |session| {
        tree.reset_measurement_queries_for(3);
        assert!(matches!(
            qualify(session, 100.0).unwrap(),
            EntryObservation::Summary(_)
        ));
        let first = tree.measurement_queries();
        assert!(first > 0);
        discarded(session);
        assert!(matches!(
            qualify(session, 100.0).unwrap(),
            EntryObservation::Summary(_)
        ));
        assert_eq!(
            tree.measurement_queries(),
            first,
            "same observation skips its producer"
        );
        assert!(matches!(
            qualify(session, 80.0).unwrap(),
            EntryObservation::Summary(_)
        ));
        assert!(
            tree.measurement_queries() > first,
            "changed complete input runs its producer"
        );
        discarded(session);
        CALLS.with(|calls| {
            assert_eq!(
                &*calls.borrow(),
                &[3],
                "lowered native sample remains operation-local"
            )
        });
        PUBLISHING.set(true);
        let root = session.compute_child(0, input(100.0)).unwrap();
        assert_eq!(root.size, Size::new(S::from_f64(100.0), S::from_f64(7.0)));
        PUBLISHING.set(false);
        let terminal = session.staged_unrounded(3).unwrap();
        assert_eq!(
            terminal.output().size,
            Size::new(S::from_f64(60.0), S::from_f64(7.0))
        );
        assert_eq!(terminal.output().location, Point::ZERO);
        // This profile withholds host caches from resident shared bodies and
        // already-sampled native leaves. Source observation stores must not
        // appear merely because the actual descendant geometry is published.
        assert!(session.cache.take_stores().is_empty());
    });
    with_session(&tree, |session| {
        qualify(session, 100.0).unwrap();
    });
    CALLS.with(|calls| assert_eq!(&*calls.borrow(), &[3, 3, 3], "fresh operation samples anew"));
    KNOWN.with(|requests| {
        assert_eq!(
            &*requests.borrow(),
            &[true, false, true],
            "sized, actual performed, then fresh sized samples"
        )
    });
}
fn failed<S: LayoutScalar>() {
    FAIL.set(true);
    PUBLISHING.set(false);
    KNOWN.with(|requests| requests.borrow_mut().clear());
    CALLS.with(|calls| calls.borrow_mut().clear());
    let tree = tree::<S>();
    with_session(&tree, |session| {
        let request = query(session, 100.0);
        let body = request.input.with_run_mode(RunMode::PerformLayout);
        let error = session
            .qualify_entry(request)
            .err()
            .expect("invalid producer refuses");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(3));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(
            matches!(error.kind(),LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(output)) if output.axis()==PhysicalAxis::Vertical && matches!(output.error(),NonNegativeFiniteScalarErrorOf::NonFinite{value} if value.to_f64().is_nan()))
        );
        assert!(
            !session.block_floats.blocks_cache(1, &body),
            "qualification failure acquired no body custody"
        );
        discarded(session);
        FAIL.set(false);
        assert!(matches!(
            qualify(session, 100.0).unwrap(),
            EntryObservation::Summary(_)
        ));
        discarded(session);
        CALLS.with(|calls| {
            assert_eq!(
                &*calls.borrow(),
                &[3, 3],
                "failed producer must run on retry"
            )
        });
    });
}
#[test]
fn entry_source_composed_reuse_and_actual_publication_f32() {
    reused::<f32>();
}
#[test]
fn entry_source_composed_reuse_and_actual_publication_f64() {
    reused::<f64>();
}
#[test]
fn entry_source_failed_producer_retries_before_body_custody_f32() {
    failed::<f32>();
}
#[test]
fn entry_source_failed_producer_retries_before_body_custody_f64() {
    failed::<f64>();
}

fn source_request<S: LayoutScalar>(
    session: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
) -> (u32, ComputeInputOf<S>) {
    let request = query(session, 100.0);
    let mut trial = session.prepare_entry_trial(&request).unwrap();
    let state = crate::block::entry::FramedEntry::begin(&mut trial, request).unwrap();
    let ControlFlow::Continue(source) = state.advance(&mut trial).unwrap() else {
        panic!("real float demand")
    };
    source.request()
}

#[test]
fn entry_source_excluded_profiles_keep_their_producer_path() {
    let ordinary = tree::<f64>();
    with_session(&ordinary, |session| {
        let (node, input) = source_request(session);
        assert!(matches!(
            session.source_float_observation(node, input).unwrap(),
            Some(crate::engine::entry_sources::FloatObservation::Miss(_))
        ));
        assert!(
            session
                .source_float_observation(node, input.with_run_mode(RunMode::PerformLayout))
                .unwrap()
                .is_none()
        );
        assert!(
            session
                .source_float_observation(1, input)
                .unwrap()
                .is_none()
        );
        let target = LayoutBoundRequestOf::continuous(
            LayoutSourceBasis::fresh(),
            LayoutRootRequestOf::standalone(
                input.sizing_request(),
                input.containing_flow_axes(),
                Size::NONE,
            ),
        )
        .target_basis();
        session.bind_fragment_frame(target, Vec::new());
        assert!(
            session
                .source_float_observation(node, input)
                .unwrap()
                .is_none()
        );
    });
    let shape = ordinary.clone().style(
        2,
        NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(PreferredSizeOf::px(60.0), PreferredSizeOf::AUTO),
            ..NodeInputOf::default()
        },
    );
    let controls = ordinary.clone().children(2, [3, 4]).input(
        4,
        LayoutInputOf::line_break(LineBreakInputOf::new().hidden()),
    );
    let grouped = ordinary.clone().style(
        0,
        NodeInputOf {
            display: Display::Block,
            text_group_align: TextGroupAlign::Center,
            ..NodeInputOf::default()
        },
    );
    for tree in [&shape, &controls, &grouped] {
        with_session(tree, |session| {
            let (node, input) = source_request(session);
            assert!(
                session
                    .source_float_observation(node, input)
                    .unwrap()
                    .is_none()
            );
        });
    }
}

fn two_sources<S: LayoutScalar>() {
    fn left<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
        assert_request(input, 20.0);
        CALLS.with(|calls| calls.borrow_mut().push(3));
        Size::new(S::from_f64(20.0), S::from_f64(7.0))
    }
    fn right<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
        assert_request(input, 20.0);
        CALLS.with(|calls| calls.borrow_mut().push(5));
        Size::new(S::from_f64(20.0), S::from_f64(9.0))
    }
    let ordinary = || NodeInputOf {
        display: Display::Block,
        ..NodeInputOf::default()
    };
    let float = |side| NodeInputOf {
        float: side,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(20.0)),
            PreferredSizeOf::AUTO,
        ),
        ..ordinary()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2, 4, 6])
        .children(2, [3])
        .children(4, [5])
        .style(0, ordinary())
        .style(1, ordinary())
        .style(2, float(Float::Left))
        .style(3, ordinary())
        .style(4, float(Float::Right))
        .style(5, ordinary())
        .style(
            6,
            NodeInputOf {
                clear: Clear::Left,
                size: Size::new(
                    PreferredSizeOf::AUTO,
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..ordinary()
            },
        )
        .measure_with(3, left::<S>)
        .measure_with(5, right::<S>);
    PUBLISHING.set(false);
    KNOWN.with(|requests| requests.borrow_mut().clear());
    CALLS.with(|calls| calls.borrow_mut().clear());
    with_session(&tree, |session| {
        let request = query(session, 100.0);
        let first_frame = request.incoming.as_ref().unwrap().frame.clone();
        let EntryObservation::Summary(summary) = session.qualify_entry(request).unwrap() else {
            panic!("two real source observations qualify their clear sibling")
        };
        let cut = summary.separation.unwrap();
        assert_eq!((cut.container, cut.node, cut.index), (1, 6, 2));
        CALLS.with(|calls| assert_eq!(&*calls.borrow(), &[3, 5]));
        discarded(session);
        for terminal in [3, 5] {
            let request = query(session, 100.0);
            let current_frame = request.incoming.as_ref().unwrap().frame.clone();
            assert!(!Arc::ptr_eq(&first_frame, &current_frame));
            tree.reset_measurement_queries_for(terminal);
            let EntryObservation::Summary(summary) = session.qualify_entry(request).unwrap() else {
                panic!("each current demand is reconstructed")
            };
            assert!(Arc::ptr_eq(
                &current_frame,
                &summary.query.incoming.as_ref().unwrap().frame
            ));
            let cut = summary.separation.unwrap();
            assert_eq!((cut.container, cut.node, cut.index), (1, 6, 2));
            assert_eq!(
                tree.measurement_queries(),
                0,
                "each distinct source node hits its own result"
            );
            discarded(session);
        }
        CALLS.with(|calls| assert_eq!(&*calls.borrow(), &[3, 5]));
        PUBLISHING.set(true);
        let root = session.compute_child(0, input(100.0)).unwrap();
        // The internal owner returns its own box; its caller publishes it.
        assert_eq!(root.size, Size::new(S::from_f64(100.0), S::from_f64(17.0)));
        PUBLISHING.set(false);
        // Left7 and right9 share a zero top. Clearing only the left float puts
        // the ten-unit following Block at7, so the shared wrapper/root end17.
        for (node, width, height, x, y, source) in [
            (1, 100.0, 17.0, 0.0, 0.0, 0),
            (2, 20.0, 7.0, 0.0, 0.0, 0),
            (3, 20.0, 7.0, 0.0, 0.0, 0),
            (4, 20.0, 9.0, 80.0, 0.0, 1),
            (5, 20.0, 9.0, 0.0, 0.0, 0),
            (6, 100.0, 10.0, 0.0, 7.0, 2),
        ] {
            let output = session.staged_unrounded(node).unwrap().output();
            assert_eq!(
                output.size,
                Size::new(S::from_f64(width), S::from_f64(height)),
                "node {node}"
            );
            assert_eq!(
                output.location,
                Point::new(S::from_f64(x), S::from_f64(y)),
                "node {node}"
            );
            assert_eq!(output.source_index, SourceIndex::new(source), "node {node}");
        }
        CALLS.with(|calls| {
            assert_eq!(
                &*calls.borrow(),
                &[3, 5, 3, 5],
                "actual bodies keep their distinct provider calls"
            )
        });
        KNOWN.with(|requests| assert_eq!(&*requests.borrow(), &[true, true, false, false]));
    });
}
#[test]
fn entry_source_distinct_numeric_siblings_resume_current_frames_f32() {
    two_sources::<f32>();
}
#[test]
fn entry_source_distinct_numeric_siblings_resume_current_frames_f64() {
    two_sources::<f64>();
}
