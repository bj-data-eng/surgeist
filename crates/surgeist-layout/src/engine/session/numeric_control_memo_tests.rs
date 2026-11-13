use super::*;
use crate::engine::session::FloatChildTrial;
use crate::output::{NamedBaselineDemand, NamedBaselineSet};
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
            ParentFormattingContext::BlockFlow,
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
    session.enable_request_memo();
    run(&mut session);
}

fn tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    let block = NodeInputOf {
        display: Display::Block,
        inline_baseline_context: InlineBaselineContext::Named(BaselineName::Alphabetic),
        ..NodeInputOf::default()
    };
    let metrics = InlineMetricsOf::try_new(
        block.inline_baseline_context,
        S::from_f64(8.0),
        S::from_f64(10.0),
    )
    .unwrap();
    let edge = |margin, border, padding| {
        InlineBoundaryEdgeOf::try_new(
            S::from_f64(margin),
            S::from_f64(border),
            S::from_f64(padding),
        )
        .unwrap()
    };
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2, 3, 4, 5, 6, 7, 8])
        .style(0, block.clone())
        .style(1, block.clone())
        // A normal Block following the ten-unit line supplies an actual child
        // cache store, independently of the controls' geometry-only effects.
        .style(
            8,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(S::from_f64(7.0))),
                ..block
            },
        )
        .input(
            2,
            LayoutInputOf::line_break(LineBreakInputOf::new().hidden()),
        )
        .input(
            3,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::Start,
                    metrics,
                    edge(0.0, 2.0, 1.0),
                )
                .unwrap(),
            ),
        )
        .input(
            4,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::End,
                    metrics,
                    edge(0.0, 1.0, 1.0),
                )
                .unwrap(),
            ),
        )
        .input(
            5,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::Start,
                    metrics,
                    edge(0.0, 3.0, 0.0),
                )
                .unwrap(),
            ),
        )
        .input(
            6,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::End,
                    metrics,
                    InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
        )
        .input(
            7,
            LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
        )
}

fn assert_geometry<S: LayoutScalar>(session: &ComputeSession<'_, PublicLayoutTreeOf<S>>) {
    let output = |node| session.staged_unrounded(node).unwrap().output();
    let hidden = output(2);
    assert_eq!(hidden.source_index, SourceIndex::new(0));
    assert_eq!(hidden.location, Point::ZERO);
    assert_eq!(hidden.size, Size::ZERO);
    // Consecutive three- and two-unit Contribution strips have zero margins.
    // Supplied8/10 metrics put both strips at baseline8.
    for (node, source, x, extent, border, padding, margin) in [
        (
            3,
            1,
            0.0,
            3.0,
            Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(2.0)),
            Edges::new(S::ZERO, S::ZERO, S::ZERO, S::ONE),
            Edges::ZERO,
        ),
        (
            4,
            2,
            3.0,
            2.0,
            Edges::new(S::ZERO, S::ONE, S::ZERO, S::ZERO),
            Edges::new(S::ZERO, S::ONE, S::ZERO, S::ZERO),
            Edges::ZERO,
        ),
    ] {
        let actual = output(node);
        assert_eq!(actual.source_index, SourceIndex::new(source));
        assert_eq!(
            actual.location,
            Point::new(S::from_f64(x), S::from_f64(8.0))
        );
        assert_eq!(actual.size, Size::new(S::from_f64(extent), S::ZERO));
        assert_eq!(actual.border, border);
        assert_eq!(actual.padding, padding);
        assert_eq!(actual.margin, margin);
    }
    for (node, source) in [(5, 3), (6, 4), (7, 5)] {
        assert_eq!(output(node).source_index, SourceIndex::new(source));
    }
    assert_eq!(output(5).border.left, S::from_f64(3.0));
    let footer = output(8);
    assert_eq!(footer.source_index, SourceIndex::new(6));
    assert_eq!(footer.location, Point::new(S::ZERO, S::from_f64(10.0)));
    assert_eq!(footer.size.height, S::from_f64(7.0));
}

#[test]
fn numeric_control_memo_restores_source_edges_hidden_breaks_and_exact_width() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>();
        with_session(&tree, |session| {
            let first = request(60.0, RunMode::PerformLayout);
            let original = session.compute_child(1, first).unwrap();
            assert_eq!(
                original.size,
                Size::new(S::from_f64(60.0), S::from_f64(17.0))
            );
            assert_geometry(session);
            let geometry = std::mem::take(&mut session.outputs.unrounded);
            let stores = std::mem::take(&mut session.cache.stores);
            let changed = session
                .compute_child(1, request(80.0, RunMode::PerformLayout))
                .unwrap();
            assert_eq!(
                changed.size,
                Size::new(S::from_f64(80.0), S::from_f64(17.0))
            );
            assert_eq!(
                session.staged_unrounded(8).unwrap().output().size.width,
                S::from_f64(80.0)
            );
            session.outputs.unrounded.clear();
            session.cache.stores.clear();
            tree.reset_measurement_queries();
            assert_eq!(session.compute_child(1, first).unwrap(), original);
            assert_eq!(tree.measurement_queries(), 0);
            assert_eq!(session.outputs.unrounded, geometry);
            assert_eq!(session.cache.stores, stores);
            assert_geometry(session);
        });
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_control_memo_keeps_named_demand_distinct_from_known_size_observation() {
    fn lane<S: LayoutScalar>() {
        let named = InlineBaselineContext::Named(BaselineName::Alphabetic);
        let tree = tree::<S>()
            .children(1, [7])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(AtomicInlineParticipation::new(
                        surgeist_text::BidiLevel::try_new(0).unwrap(),
                        InlineBreakOpportunity::Allowed,
                    )),
                    inline_baseline_context: named,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(60.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(
                7,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new()
                        .with_metrics(InlineMetricsOf::try_new(named, S::ZERO, S::ZERO).unwrap()),
                ),
            );
        with_session(&tree, |session| {
            let input = request(60.0, RunMode::ComputeSize);
            let unknown = session.compute_child(1, input).unwrap();
            assert_eq!(
                unknown.named_baselines.get(BaselineName::Alphabetic).last(),
                BaselineFactOf::Unknown
            );
            let demanded = input.with_named_baseline_demand(NamedBaselineDemand::Fact {
                name: BaselineName::Alphabetic,
                set: NamedBaselineSet::Last,
            });
            let actual = session.compute_child(1, demanded).unwrap();
            assert_eq!(actual.size, Size::new(S::from_f64(60.0), S::from_f64(20.0)));
            assert_eq!(
                actual.named_baselines.get(BaselineName::Alphabetic).last(),
                BaselineFactOf::Available(
                    BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, S::ZERO).unwrap()
                )
            );
            assert_eq!(session.compute_child(1, input).unwrap(), unknown);
            assert_eq!(session.compute_child(1, demanded).unwrap(), actual);
        });
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_control_memo_failed_measurement_never_completes() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>()
            .children(1, [2, 3, 4, 5, 6, 7, 8])
            .style(
                8,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(AtomicInlineParticipation::new(
                        surgeist_text::BidiLevel::try_new(0).unwrap(),
                        InlineBreakOpportunity::Allowed,
                    )),
                    ..NodeInputOf::default()
                },
            )
            .measure(8, Size::new(S::from_f64(10.0), S::NAN));
        with_session(&tree, |session| {
            let input = request(60.0, RunMode::PerformLayout);
            for _ in 0..2 {
                let error = session.compute_child(1, input).unwrap_err();
                assert_eq!(error.site(), LayoutErrorSiteOf::Node(8));
                assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
                assert!(
                    matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(output))
                    if output.axis() == PhysicalAxis::Vertical && matches!(output.error(), NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan()))
                );
                let key = session
                    .request_key(1, input, CacheKeyContext::new())
                    .unwrap();
                assert!(session.memo.as_ref().unwrap().find(key).is_none());
            }
        });
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_control_memo_trial_discard_and_adoption_keep_local_effect_custody() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>();
        with_session(&tree, |session| {
            let input = request(60.0, RunMode::PerformLayout);
            let rejected = session
                .probe_float_child(1, input, FloatChildTrial::Detached)
                .unwrap();
            assert_eq!(
                rejected.output.size,
                Size::new(S::from_f64(60.0), S::from_f64(17.0))
            );
            drop(rejected);
            assert!(session.outputs.unrounded.is_empty());
            assert!(session.cache.stores.is_empty());
            tree.reset_measurement_queries();
            let accepted = session
                .probe_float_child(1, input, FloatChildTrial::Detached)
                .unwrap();
            assert_eq!(
                tree.measurement_queries(),
                0,
                "a completed trial may be reused without publishing its effects"
            );
            assert!(session.outputs.unrounded.is_empty());
            assert!(session.cache.stores.is_empty());
            session.accept_float_child(&accepted).unwrap();
            assert_geometry(session);
            let geometry = std::mem::take(&mut session.outputs.unrounded);
            let stores = std::mem::take(&mut session.cache.stores);
            assert!(!stores.is_empty());
            assert_eq!(session.compute_child(1, input).unwrap(), accepted.output);
            assert_eq!(session.outputs.unrounded, geometry);
            assert_eq!(session.cache.stores, stores);
            assert_geometry(session);
        });
    }
    lane::<f32>();
    lane::<f64>();
}
