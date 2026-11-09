use super::*;
use crate::engine::ComputeSession;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

#[path = "fragment_margin_boundary_tests.rs"]
mod margin_boundary;

fn suspension<S: LayoutScalar>() {
    let block = |height: Option<f64>| NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::AUTO,
            height.map_or(PreferredSizeOf::AUTO, |h| {
                PreferredSizeOf::px(S::from_f64(h))
            }),
        ),
        ..Default::default()
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .style(0, block(None))
        .style(1, block(Some(10.0)))
        .style(2, block(None))
        .style(3, block(Some(10.0)))
        .children(0, [1, 2])
        .children(2, [3]);
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(100.0)), None),
        Size::new(Some(S::from_f64(100.0)), None),
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
    );
    let requests = std::rc::Rc::new(std::cell::RefCell::new(Requests::new()));
    let target = crate::LayoutBoundRequestOf::continuous(
        crate::LayoutSourceBasis::fresh(),
        crate::LayoutRootRequestOf::standalone(
            input.sizing_request(),
            input.containing_flow_axes(),
            input
                .parent()
                .map(|value| value.map(|v| crate::NonNegativeFiniteOf::new(v).unwrap())),
        ),
    )
    .target_basis();
    let id = {
        let mut trial = ComputeSession::new(&tree, Vec::new());
        trial.bind_fragment_frame(target.clone(), Vec::new());
        trial
            .install_block_scopes(
                0,
                Dispatch {
                    scope: ReceiptId(0),
                    view: super::SourceViewId(0),
                    target: target.clone(),
                    owners: std::sync::Arc::from([2]),
                    paths: std::sync::Arc::from([0]),
                    ready: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
                    requests: requests.clone(),
                },
            )
            .unwrap();
        let stop = crate::block::compute_block_scoped(&mut trial, 0, input).unwrap_err();
        assert!(trial.staged_unrounded(1).is_ok());
        assert!(trial.staged_unrounded(2).is_err());
        let ComputeStop::NeedScope(id) = stop else {
            panic!("scope demand is internal control, not a layout error")
        };
        id
    };
    let suspended = requests
        .borrow_mut()
        .take(id)
        .expect("driver owns the emitted request");
    let request = suspended.entry;
    assert_eq!(request.node, 2);
    assert_eq!(
        request.input.parent(),
        Size::new(Some(S::from_f64(100.0)), None)
    );
    assert!(requests.borrow_mut().take(id).is_none());
    // A separately produced child body retains its own staging. The parent
    // consumes only the exact principal result, not a copied descendant tree.
    let mut body = ComputeSession::new(&tree, Vec::new());
    let principal =
        crate::block::compute_block_scoped(&mut body, request.node, request.input).unwrap();
    assert_eq!(principal.size.height, S::from_f64(10.0));
    assert!(body.staged_unrounded(3).is_ok());
    let ready = Ready {
        id: ReceiptId(7),
        view: super::SourceViewId(0),
        projection: Some(super::ProductionId(11)),
        entry: request,
        output: principal,
    };
    let mut accepted = ComputeSession::new(&tree, Vec::new());
    accepted.bind_fragment_frame(target.clone(), Vec::new());
    accepted
        .install_block_scopes(
            0,
            Dispatch {
                scope: ReceiptId(0),
                view: super::SourceViewId(0),
                target: target.clone(),
                owners: std::sync::Arc::from([2]),
                paths: std::sync::Arc::from([0]),
                ready: std::rc::Rc::new(std::cell::RefCell::new(vec![ready])),
                requests: requests.clone(),
            },
        )
        .unwrap();
    let output = crate::block::compute_block_scoped(&mut accepted, 0, input).unwrap();
    assert_eq!(output.size.height, S::from_f64(20.0));
    assert!(accepted.staged_unrounded(2).is_ok());
    assert!(accepted.staged_unrounded(3).is_err());
    let adopted = accepted.block_packet().unwrap();
    assert_eq!(adopted.receipts, [ReceiptId(7)]);
    assert_eq!(adopted.projections, [(2, super::ProductionId(11))]);

    let mut stale = ready;
    stale.entry.cursor = stale.entry.cursor + S::ONE;
    let mut other_view = ready;
    other_view.view = super::SourceViewId(1);
    for stale in [stale, other_view] {
        let mut rejected = ComputeSession::new(&tree, Vec::new());
        rejected.bind_fragment_frame(target.clone(), Vec::new());
        rejected
            .install_block_scopes(
                0,
                Dispatch {
                    scope: ReceiptId(0),
                    view: super::SourceViewId(0),
                    target: target.clone(),
                    owners: std::sync::Arc::from([2]),
                    paths: std::sync::Arc::from([0]),
                    ready: std::rc::Rc::new(std::cell::RefCell::new(vec![stale])),
                    requests: requests.clone(),
                },
            )
            .unwrap();
        assert!(matches!(
            crate::block::compute_block_scoped(&mut rejected, 0, input),
            Err(ComputeStop::NeedScope(_))
        ));
    }

    // The rejected trial is gone. Replay uses a new staging owner, while the
    // owned request still retains the original child context.
    let mut replay = ComputeSession::new(&tree, Vec::new());
    let output = crate::block::compute_block_scoped(&mut replay, 0, input).unwrap();
    assert_eq!(
        output.size,
        Size::new(S::from_f64(100.0), S::from_f64(20.0))
    );
    assert_eq!(
        replay.staged_unrounded(2).unwrap().output().location.y,
        S::from_f64(10.0)
    );
}
#[test]
fn block_scope_stop_retains_request_and_discards_trial_f32() {
    suspension::<f32>();
}
#[test]
fn block_scope_stop_retains_request_and_discards_trial_f64() {
    suspension::<f64>();
}

#[test]
fn scope_request_arena_releases_consumed_work_without_reusing_identity() {
    let mut arena = Requests::new();
    let first = arena.insert(11).unwrap();
    assert_eq!(arena.take(first), Some(11));
    let capacity = arena.entries.capacity();
    for value in 0..1000 {
        let current = arena.insert(value).unwrap();
        assert_ne!(current, first);
        assert_eq!(arena.take(first), None);
        assert_eq!(arena.take(current), Some(value));
        assert!(arena.entries.is_empty());
    }
    assert_eq!(arena.entries.capacity(), capacity);
}
