use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    engine::contracts::Compute,
    error::{LayoutErrorKindOf, LayoutInvalidInputOf, LayoutOperation},
    geometry::{FlowAxes, Point, Size},
    inline_wrapping::{
        ChildOrderCompleteness, GeneratedBoxParticipation, GeneratedBoxRef, GeneratedChildBoxInput,
        InlineChildOrderInput, NativeInlineBoxFamilyInput, WrapInside, WrappingInput,
        WrappingScopeError, WrappingScopeInput,
    },
    node_input::{
        AtomicInlineParticipation, Direction, Display, Float, InlineBoundaryEdgeOf,
        InlineBoundaryInputOf, InlineBoundaryKind, InlineBreakOpportunity, InlineMetricsOf,
        LayoutInputOf, NodeInputOf, WritingMode,
    },
    output::{
        ComputeInputOf, ContainingLayoutContext, LayoutOutputEntryOf, NodeOutputOf,
        ParentFormattingContext, RequestedAxis, RunMode, SizingMode,
    },
    scalar::LayoutScalar,
    sizing::PreferredSizeOf,
    text_layout::LayoutFlowInput,
    tree::LayoutTree,
};
use surgeist_text::supplied_line::BidiLevel;

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
                        crate::node_input::InlineBaselineContext::Physical,
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
        crate::value::SizingRequestOf::new(
            Size::new(
                crate::value::AxisSizingRequestOf::new(
                    crate::value::AvailableSpaceOf::Definite(
                        crate::value::NonNegativeFiniteOf::new(S::from_f64(25.0)).unwrap(),
                    ),
                    crate::value::SizingOperation::Normal,
                ),
                crate::value::AxisSizingRequestOf::new(
                    crate::value::AvailableSpaceOf::Indefinite,
                    crate::value::SizingOperation::Normal,
                ),
            ),
            crate::value::OrthogonalFallbackOf::new(
                crate::geometry::Size::NONE,
                crate::value::NearestScrollportOf::Unknown,
                crate::geometry::Size::splat(crate::value::OrthogonalFallbackLimitOf::Unknown),
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
                WrappingInput {
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
            WrappingInput {
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
            WrappingInput {
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

fn float_formatter_wrapping() -> WrappingInput<u32> {
    let child = |node| GeneratedChildBoxInput {
        box_: GeneratedBoxRef::Node(node),
        participation: GeneratedBoxParticipation::InFlow,
    };
    WrappingInput {
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

// Entry sources charge funding on every visit, as entry storage does, so a session that holds
// them repeats every subgrid computation.
#[test]
fn entry_sources_keep_a_session_from_reusing_subgrid_computations() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>(false);
        let flow =
            LayoutFlowInput::try_new(&tree, 0, Vec::new(), WrappingInput::default(), Vec::new())
                .unwrap();
        let mut session = ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        assert!(session.reuses_subgrid_computations());
        session.entry_sources = Some(std::sync::Arc::new(std::sync::Mutex::new(
            crate::engine::entry_sources::SourceMemo::new(None),
        )));
        assert!(!session.reuses_subgrid_computations());
    }
    lane::<f32>();
    lane::<f64>();
}

// A hidden write of a node removes the stores the node staged before it and a store staged after
// it stays, so a compaction that keeps the last store and the last geometry write of the node
// must not let an earlier store outlive the hidden write.
#[test]
fn a_hidden_write_in_a_capture_replays_to_the_same_stores_and_geometry() {
    fn lane<S: LayoutScalar>() {
        let tree = tree::<S>(false);
        let flow =
            LayoutFlowInput::try_new(&tree, 0, Vec::new(), WrappingInput::default(), Vec::new())
                .unwrap();
        let session = || ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        let hidden = LayoutOutputEntryOf::hidden_unrounded(2, crate::output::SourceIndex::ZERO);
        let visible = |width: f64| {
            LayoutOutputEntryOf::ordinary_unrounded(
                2,
                crate::output::NormalGeometryOf::metadata(
                    Point::ZERO,
                    Size::splat(S::from_f64(width)),
                ),
                NodeOutputOf::with_source_index(crate::output::SourceIndex::ZERO),
            )
        };
        for (label, hidden_after_store) in
            [("store then hidden", true), ("hidden then store", false)]
        {
            let mut original = session();
            let mut replayed = session();
            let start = original.begin_effect_capture(crate::engine::CaptureKind::Preparation);
            if hidden_after_store {
                original
                    .compute_child(2, input(RunMode::PerformLayout))
                    .unwrap();
                original.set_unrounded(hidden);
                original.set_unrounded(visible(7.0));
            } else {
                original.set_unrounded(hidden);
                original
                    .compute_child(2, input(RunMode::PerformLayout))
                    .unwrap();
                original.set_unrounded(visible(7.0));
            }
            let effects = original
                .end_effect_capture(start)
                .unwrap_or_else(|| panic!("{label}: the capture is reusable"));
            assert_eq!(
                original.cache.stores.iter().any(|entry| entry.node() == 2),
                !hidden_after_store,
                "{label}: the stores of the node after its hidden write"
            );
            replayed.replay_effects(&effects).unwrap();
            assert_eq!(
                replayed
                    .cache
                    .stores
                    .iter()
                    .map(|entry| (entry.node(), *entry.input(), entry.context()))
                    .collect::<Vec<_>>(),
                original
                    .cache
                    .stores
                    .iter()
                    .map(|entry| (entry.node(), *entry.input(), entry.context()))
                    .collect::<Vec<_>>(),
                "{label}: the replayed stores"
            );
            assert_eq!(
                replayed.outputs.unrounded, original.outputs.unrounded,
                "{label}: the replayed geometry"
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

// Repeated acceptances of a float formatter in one capture compact to the writes of the last,
// and replay to the session state they staged.
#[test]
fn repeated_float_formatter_acceptances_compact_and_replay_to_the_same_state() {
    fn lane<S: LayoutScalar>() {
        let tree = float_formatter_tree::<S>();
        let flow =
            LayoutFlowInput::try_new(&tree, 0, Vec::new(), float_formatter_wrapping(), Vec::new())
                .unwrap();
        let session = || ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        let mut probing = session();
        let mut probe = || {
            probing
                .probe_float_child(
                    4,
                    input(RunMode::PerformLayout),
                    crate::engine::FloatChildTrial::Detached,
                )
                .unwrap()
        };
        let with_runs = probe();
        let mut without_runs = probe();
        std::sync::Arc::get_mut(&mut without_runs)
            .expect("the packet is not shared")
            .native_inline_runs
            .clear();

        let mut original = session();
        let mut replayed = session();
        let start = original.begin_effect_capture(crate::engine::CaptureKind::Preparation);
        for packet in [
            &with_runs,
            &with_runs,
            &without_runs,
            &with_runs,
            &with_runs,
            &with_runs,
        ] {
            original.accept_float_child(packet).unwrap();
        }
        let single = {
            let mut once = session();
            let start = once.begin_effect_capture(crate::engine::CaptureKind::Preparation);
            once.accept_float_child(&with_runs).unwrap();
            once.end_effect_capture(start).expect("recorded").len()
        };
        let effects = original.end_effect_capture(start).expect("recorded");
        assert!(
            effects.len() <= single + 1,
            "{} writes against {single} for one acceptance",
            effects.len()
        );
        replayed.replay_effects(&effects).unwrap();
        assert_eq!(native_runs_of(&replayed), native_runs_of(&original));
        assert!(!native_runs_of(&replayed).is_empty());
        assert_eq!(
            replayed.wrapping.native_completions.len(),
            original.wrapping.native_completions.len()
        );
        assert_eq!(replayed.cache.stores.len(), original.cache.stores.len());
        assert_eq!(replayed.outputs.unrounded, original.outputs.unrounded);
    }
    lane::<f32>();
    lane::<f64>();
}
