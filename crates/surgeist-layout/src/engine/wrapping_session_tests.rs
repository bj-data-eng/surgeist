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
        Size::new(
            AvailableOf::definite(S::from_f64(25.0)),
            AvailableOf::MAX_CONTENT,
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
            let mut owner = ComputeSession::with_flow(&capture, capture.invalidated.clone());
            let output = owner.compute_child(0, input(RunMode::ComputeSize)).unwrap();
            assert_eq!(output.size, Size::new(S::from_f64(25.0), S::from_f64(10.0)));
            assert!(owner.text.is_none());
            assert_eq!(
                owner.wrapping_context().is_some(),
                policy == WrapInside::Avoid
            );
            assert_eq!(
                owner
                    .cache_store_entries
                    .iter()
                    .any(|entry| entry.node() == 0),
                policy == WrapInside::Auto
            );
            assert!(
                owner
                    .cache_store_entries
                    .iter()
                    .any(|entry| entry.node() == 2)
            );
            if policy == WrapInside::Avoid {
                assert!(
                    owner
                        .cache_clear_entries
                        .iter()
                        .any(|entry| entry.node() == 0)
                );
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
        let mut owner = ComputeSession::with_flow(&avoid, avoid.invalidated.clone());
        let packet = owner
            .probe_float_child(4, input(RunMode::PerformLayout))
            .unwrap();
        assert_eq!(packet.output.size, Size::splat(S::from_f64(10.0)));
        assert!(owner.cache_store_entries.is_empty());
        owner.accept_float_child(&packet).unwrap();
        assert!(
            owner
                .cache_store_entries
                .iter()
                .any(|entry| entry.node() == 4)
        );

        let mut changed = ComputeSession::with_flow(&auto, auto.invalidated.clone());
        let error = changed.accept_float_child(&packet).unwrap_err();
        assert_eq!(error.operation(), LayoutOperation::InlineWrapping);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::InconsistentIncidence
            ))
        ));
        assert!(changed.cache_store_entries.is_empty());
        assert!(changed.unrounded_entries.is_empty());
    }
    lane::<f32>();
    lane::<f64>();
}
