use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

fn tree<S: LayoutScalar>(float: bool) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(
            0,
            if float {
                vec![4, 1, 2, 3]
            } else {
                vec![1, 2, 3]
            },
        )
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(25.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                ..NodeInputOf::default()
            },
        );
    for (node, kind) in [(1, InlineBoundaryKind::Start), (3, InlineBoundaryKind::End)] {
        tree = tree.input(
            node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    kind,
                    InlineMetricsOf::try_new(
                        crate::InlineBaselineContext::Physical,
                        S::from_f64(10.0),
                        S::from_f64(10.0),
                    )
                    .unwrap(),
                    InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
        );
    }
    if float {
        tree = tree.style(
            4,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn scopes(policy: WrapInside) -> Vec<WrappingScopeInput<u32>> {
    vec![WrappingScopeInput::NativePair {
        metric_contribution: None,
        edges: Default::default(),
        start: 1,
        end: 3,
        wrap_inside: policy,
    }]
}

fn input<S: LayoutScalar>(mode: RunMode) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(S::from_f64(25.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        crate::SizingRequestOf::new(
            Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(25.0)).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
            ),
            crate::OrthogonalFallbackOf::new(
                crate::Size::NONE,
                crate::NearestScrollportOf::Unknown,
                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        ),
    )
}

#[test]
fn native_avoid_results_do_not_enter_geometry_only_host_caches() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>(false);
        for policy in [WrapInside::Auto, WrapInside::Avoid] {
            let capture = LayoutFlowInput::try_new(
                &tree,
                0,
                Vec::new(),
                crate::WrappingInput {
                    native_inline_boxes: Vec::new(),
                    gaps: Vec::new(),
                    scopes: scopes(policy),
                    child_order: Vec::new(),
                },
                Vec::new(),
            )
            .unwrap();
            let mut owner =
                ComputeSession::with_flow_unobserved(&capture, capture.invalidated.clone());
            let output = owner.compute_child(0, input(RunMode::ComputeSize)).unwrap();
            assert_eq!(output.size, Size::new(S::from_f64(25.0), S::from_f64(10.0)));
            assert!(owner.text.is_none());
            assert_eq!(
                owner.wrapping_context().is_some(),
                policy == WrapInside::Avoid
            );
            assert_eq!(
                owner.cache.stores.iter().any(|entry| entry.node() == 0),
                policy == WrapInside::Auto
            );
            assert!(owner.cache.stores.iter().any(|entry| entry.node() == 2));
            if policy == WrapInside::Avoid {
                assert!(owner.cache.clears.iter().any(|entry| entry.node() == 0));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn a_float_packet_cannot_cross_an_immutable_wrapping_basis() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>(true);
        let avoid = LayoutFlowInput::try_new(
            &tree,
            0,
            Vec::new(),
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: scopes(WrapInside::Avoid),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let auto = LayoutFlowInput::try_new(
            &tree,
            0,
            Vec::new(),
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: scopes(WrapInside::Auto),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let mut owner = ComputeSession::with_flow_unobserved(&avoid, avoid.invalidated.clone());
        let packet = owner
            .probe_float_child(
                4,
                input(RunMode::PerformLayout),
                crate::engine::FloatChildTrial::Detached,
            )
            .unwrap();
        assert_eq!(packet.output.size, Size::splat(S::from_f64(10.0)));
        assert!(owner.cache.stores.is_empty());
        owner.accept_float_child(&packet).unwrap();
        assert!(owner.cache.stores.iter().any(|entry| entry.node() == 4));

        let mut changed = ComputeSession::with_flow_unobserved(&auto, auto.invalidated.clone());
        let error = changed.accept_float_child(&packet).unwrap_err();
        assert_eq!(error.operation(), LayoutOperation::InlineWrapping);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::InconsistentIncidence
            ))
        ));
        assert!(changed.cache.stores.is_empty());
        assert!(changed.outputs.unrounded.is_empty());
    }
    lane::<f32>();
    lane::<f64>();
}

// A float Block that is a native inline-box formatter holding two atomic inlines, beside which
// nothing else lays out.
fn float_formatter_tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    let atomic = || NodeInputOf {
        display: Display::InlineBlock,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(10.0)),
            PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        atomic_inline_participation: Some(AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            InlineBreakOpportunity::Allowed,
        )),
        ..NodeInputOf::default()
    };
    PublicLayoutTreeOf::new()
        .children(0, vec![4])
        .children(4, vec![5, 6])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(25.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(5, atomic())
        .style(6, atomic())
}

fn float_formatter_wrapping() -> crate::WrappingInput<u32> {
    let child = |node| GeneratedChildBoxInput {
        box_: GeneratedBoxRef::Node(node),
        participation: GeneratedBoxParticipation::InFlow,
    };
    crate::WrappingInput {
        native_inline_boxes: vec![NativeInlineBoxFamilyInput { formatter: 4 }],
        gaps: Vec::new(),
        scopes: Vec::new(),
        child_order: vec![InlineChildOrderInput {
            parent: GeneratedBoxRef::Node(4),
            children: vec![child(5), child(6)],
            prefix: ChildOrderCompleteness::Complete,
            suffix: ChildOrderCompleteness::Complete,
            formatting_context: 4,
        }],
    }
}

fn native_runs_of<Tree: LayoutTree>(
    session: &ComputeSession<'_, Tree>,
) -> Vec<(usize, std::ops::Range<usize>)> {
    session
        .wrapping
        .native_runs
        .iter()
        .map(|run| (run.family, run.source.clone()))
        .collect()
}

// A float formatter's packet is accepted into the session in place of the formatter's own
// computation. A formatter that completed again with no run removes the runs of its family, and a
// reuse that stands in for the acceptance must remove them as well.
#[test]
fn replaying_a_float_formatter_acceptance_removes_the_runs_the_acceptance_removed() {
    fn lane<S: LayoutScalar>() {
        let tree = float_formatter_tree::<S>();
        let flow =
            LayoutFlowInput::try_new(&tree, 0, Vec::new(), float_formatter_wrapping(), Vec::new())
                .unwrap();
        let session = || ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        let mut probing = session();
        let probe = |probing: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>| {
            probing
                .probe_float_child(
                    4,
                    input(RunMode::PerformLayout),
                    crate::engine::FloatChildTrial::Detached,
                )
                .unwrap()
        };
        let with_runs = probe(&mut probing);
        assert!(
            !with_runs.native_inline_runs.is_empty()
                && !with_runs.native_inline_completions.is_empty(),
            "the formatter completes with runs"
        );
        // The same completion with no run: what a formatter without inline content publishes.
        let mut without_runs = probe(&mut probing);
        std::sync::Arc::get_mut(&mut without_runs)
            .expect("the packet is not shared")
            .native_inline_runs
            .clear();

        let mut original = session();
        let mut replayed = session();
        original.accept_float_child(&with_runs).unwrap();
        replayed.accept_float_child(&with_runs).unwrap();
        assert!(
            !native_runs_of(&original).is_empty(),
            "the session holds the runs"
        );
        assert_eq!(native_runs_of(&original), native_runs_of(&replayed));

        let start = original.begin_effect_capture(crate::engine::CaptureKind::Preparation);
        original.accept_float_child(&without_runs).unwrap();
        let effects = original
            .end_effect_capture(start)
            .expect("the acceptance is recorded");
        assert!(
            native_runs_of(&original).is_empty(),
            "the acceptance removes the runs of the family"
        );

        replayed.replay_effects(&effects).unwrap();
        assert_eq!(
            native_runs_of(&replayed),
            native_runs_of(&original),
            "the replay removes the runs the acceptance removed"
        );
        assert_eq!(
            replayed.wrapping.native_completions.len(),
            original.wrapping.native_completions.len()
        );
    }
    lane::<f32>();
    lane::<f64>();
}
