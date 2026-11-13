use super::*;
use crate::engine::block_scope::{Dispatch, ReceiptId, Requests, SourceViewId};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

fn request<S: LayoutScalar>(width: f64, mode: RunMode) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(width)), None),
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::Flex,
        ),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::Normal,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
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
                display: Display::Flex,
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(S::from_f64(7.0))),
                flex_basis: FlexBasisOf::px(S::from_f64(60.0)),
                flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
}

fn with_session<S: LayoutScalar, T>(
    tree: &PublicLayoutTreeOf<S>,
    owners: &[u32],
    paths: &[u32],
    run: impl FnOnce(&mut ComputeSession<'_, PublicLayoutTreeOf<S>>) -> T,
) -> T {
    let (graph, invalidated) =
        crate::engine::validation::validate_layout_request_with_text_graph(tree, 0, &[], &[])
            .unwrap();
    assert!(graph.path_counts().iter().all(|&count| count == 1));
    let input = request::<S>(60.0, RunMode::PerformLayout);
    let target = LayoutBoundRequestOf::continuous(
        LayoutSourceBasis::fresh(),
        LayoutRootRequestOf::standalone(
            input.sizing_request(),
            input.containing_flow_axes(),
            Size::NONE,
        ),
    )
    .target_basis();
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        &graph,
        &WrappingInput::default(),
        &[],
        |_| false,
    )
    .unwrap();
    let mut session = ComputeSession::new(tree, invalidated);
    session.bind_fragment_frame(target.clone(), Vec::new());
    session
        .install_block_scopes(
            0,
            Dispatch {
                scope: ReceiptId(0),
                view: SourceViewId(0),
                target,
                owners: owners.into(),
                paths: paths.into(),
                ready: Default::default(),
                requests: std::rc::Rc::new(std::cell::RefCell::new(Requests::new())),
            },
        )
        .unwrap();
    session.retain_block_bodies();
    session.install_wrapping(Arc::new(wrapping), &graph);
    session.enable_request_memo();
    run(&mut session)
}

fn assert_empty_margins<S: LayoutScalar>(session: &ComputeSession<'_, PublicLayoutTreeOf<S>>) {
    let margins = &session.fragmentation.as_ref().unwrap().margins;
    assert!(margins.entry(2).unwrap().is_empty());
    for start in [true, false] {
        assert!(margins.boundary(2, start).unwrap().is_empty());
    }
    for side in [PhysicalSide::Top, PhysicalSide::Bottom] {
        assert!(margins.reported(2, side).unwrap().is_empty());
    }
}

fn interleaved<S: LayoutScalar>() {
    let tree = tree::<S>();
    with_session(&tree, &[], &[0], |session| {
        let first = request(60.0, RunMode::PerformLayout);
        let original = session.compute_child(1, first).unwrap();
        let geometry = session.staged_unrounded(2).unwrap();
        assert_eq!(
            geometry.output().size,
            Size::new(S::from_f64(60.0), S::from_f64(7.0))
        );
        assert_empty_margins(session);
        session
            .compute_child(1, request(80.0, RunMode::ComputeSize))
            .unwrap();
        session
            .compute_child(1, request(80.0, RunMode::PerformLayout))
            .unwrap();
        assert_eq!(
            session.staged_unrounded(2).unwrap().output().size.width,
            S::from_f64(80.0)
        );
        tree.reset_measurement_queries();
        assert_eq!(session.compute_child(1, first).unwrap(), original);
        assert_eq!(
            tree.measurement_queries(),
            0,
            "completed parent skips descendant admission"
        );
        assert_eq!(session.staged_unrounded(2).unwrap(), geometry);
        assert_empty_margins(session);
        let actual = session
            .block_bodies
            .as_ref()
            .unwrap()
            .iter()
            .map(|body| (body.node, body.input, body.output))
            .collect::<Vec<_>>();
        let expected = with_session(&tree, &[], &[0], |reference| {
            reference.release_request_memo();
            for (width, mode) in [
                (60.0, RunMode::PerformLayout),
                (80.0, RunMode::ComputeSize),
                (80.0, RunMode::PerformLayout),
                (60.0, RunMode::PerformLayout),
            ] {
                reference.compute_child(1, request(width, mode)).unwrap();
            }
            assert_eq!(reference.outputs.unrounded, session.outputs.unrounded);
            assert_empty_margins(reference);
            reference
                .block_bodies
                .as_ref()
                .unwrap()
                .iter()
                .map(|body| (body.node, body.input, body.output))
                .collect::<Vec<_>>()
        });
        assert_eq!(actual.len(), expected.len());
        for row in actual {
            assert!(
                expected.contains(&row),
                "body rows are consumed by exact node/input identity"
            );
        }
        assert!(session.cache.stores.is_empty());
        assert!(session.block_scopes().unwrap().receipts.is_empty());
    });
}

#[test]
fn scoped_memo_replays_interleaved_bodies_geometry_and_empty_margins() {
    interleaved::<f32>();
    interleaved::<f64>();
}

#[test]
fn scoped_memo_excludes_scope_participants_nonleaf_blocks_and_other_parent_roles() {
    let input = request(60.0, RunMode::PerformLayout);
    let context = CacheKeyContext::new();
    let tree = tree::<f32>();
    for (owners, paths) in [(vec![1], vec![0]), (Vec::new(), vec![0, 2])] {
        with_session(&tree, &owners, &paths, |session| {
            assert!(session.request_key(1, input, context).is_none());
        });
    }
    let block = tree.clone().style(
        1,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    );
    with_session(&block, &[], &[0], |session| {
        assert!(session.request_key(1, input, context).is_none());
        assert!(session.request_key(2, input, context).is_none());
    });
    with_session(&tree, &[], &[0], |session| {
        let other = ComputeInputOf::root_layout(
            input.known(),
            input.parent(),
            ContainingLayoutContext::new(
                input.containing_flow_axes(),
                ParentFormattingContext::BlockFlow,
            ),
            input.sizing_request(),
        );
        assert!(session.request_key(2, other, context).is_none());
    });
}

#[test]
fn scoped_memo_stays_in_its_original_trial_and_binding() {
    let tree = tree::<f32>();
    with_session(&tree, &[], &[0], |session| {
        let input = request(60.0, RunMode::PerformLayout);
        let mut trial = session
            .child_staging(1, super::super::FloatChildTrial::Detached, None)
            .unwrap();
        assert!(trial.memo.is_none());
        trial.compute_child(1, input).unwrap();
        drop(trial);
        tree.reset_measurement_queries();
        session.compute_child(1, input).unwrap();
        assert!(tree.measurement_queries() > 0);
        assert!(session.memo.as_ref().unwrap().fork().is_none());
        let target = session.bound.clone().unwrap();
        session.bind_fragment_frame(target, Vec::new());
        assert!(session.memo.is_none());
    });
}

thread_local! {
    static SAMPLES: Cell<usize> = const { Cell::new(0) };
}

fn sample<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
    SAMPLES.set(SAMPLES.get() + 1);
    let width = input
        .known_content_size()
        .width
        .unwrap_or(S::from_f64(60.0));
    Size::new(
        width,
        if width > S::from_f64(70.0) {
            S::NAN
        } else {
            S::from_f64(7.0)
        },
    )
}

#[test]
fn scoped_memo_preserves_excluded_sampling_and_failed_completion() {
    let tree = tree::<f32>()
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
                flex_basis: FlexBasisOf::px(60.0),
                flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .measure_with(2, sample::<f32>);
    with_session(&tree, &[], &[0], |session| {
        SAMPLES.set(0);
        let input = request(60.0, RunMode::ComputeSize);
        session.compute_child(2, input).unwrap();
        session.compute_child(2, input).unwrap();
        assert_eq!(
            SAMPLES.get(),
            2,
            "excluded requests retain native sample operations"
        );
    });
    let tree = tree.style(
        1,
        NodeInputOf {
            display: Display::Flex,
            ..NodeInputOf::default()
        },
    );
    with_session(&tree, &[], &[0], |session| {
        let input = request(80.0, RunMode::PerformLayout);
        let error = session.compute_child(1, input).unwrap_err();
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(output))
                if output.axis() == PhysicalAxis::Vertical
                    && matches!(output.error(), NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.is_nan())
        ));
        let key = session
            .request_key(1, input, CacheKeyContext::new())
            .unwrap();
        assert!(session.memo.as_ref().unwrap().find(key).is_none());
    });
}
