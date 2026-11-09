use super::*;
use crate::engine::FloatChildTrial;
use crate::engine::contracts::Compute;
use crate::geometry::{LogicalEdgesOf, LogicalPointOf};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

fn normal_request<S: LayoutScalar>(width: f64, height: f64, parent: Size<S>) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(width)), Some(S::from_f64(height))),
        parent.map(Some),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(parent.width).unwrap()),
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

fn framed<S: LayoutScalar>() {
    let sized = |width: f64, height: f64| NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [4])
        .children(4, [])
        .style(0, sized(100.0, 100.0))
        .style(
            1,
            NodeInputOf {
                float: Float::Left,
                ..sized(10.0, 30.0)
            },
        )
        .style(2, sized(40.0, 10.0))
        .style(4, sized(10.0, 5.0));
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let bound = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    let root_request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        Size::splat(Some(bound)),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(bound),
        SizingOperation::Normal,
    )));
    let mut owner = crate::engine::ComputeSession::new(&tree, Vec::new());
    crate::engine::compute_root(&mut owner, 0, root_request).unwrap();
    let prior = owner.staged_unrounded(4).unwrap();
    let root = owner
        .block_float_state()
        .unwrap()
        .completed
        .as_ref()
        .unwrap();
    assert_eq!(root.node, 0);
    assert_eq!(root.ledger.len(), 1);
    let resident_request = normal_request(40.0, 10.0, root.output.size);
    let resident_frame = root
        .ledger
        .for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO));
    let mut resident_ledger = FloatExclusions::new(
        axes,
        Size::new(S::from_f64(40.0), S::from_f64(10.0)),
        S::from_f64(40.0),
        LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    resident_ledger.inherit_into_child(&resident_frame).unwrap();
    owner
        .block_float_state()
        .unwrap()
        .push(2, resident_request, resident_frame, None)
        .unwrap();
    let resident = owner
        .block_float_state()
        .unwrap()
        .incoming(2, resident_request)
        .unwrap();
    let target_request = normal_request(10.0, 5.0, Size::new(S::from_f64(40.0), S::from_f64(10.0)));
    let target = Incoming {
        cut: None,
        node: 4,
        request: target_request,
        frame: std::sync::Arc::new(
            resident_ledger.for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO)),
        ),
    };
    let mut trial = owner
        .child_staging(4, FloatChildTrial::InFlow, Some(target.clone()))
        .unwrap();
    let actual = trial
        .block_float_state()
        .unwrap()
        .incoming(4, target_request)
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&actual.frame, &target.frame));
    assert!(
        trial
            .block_float_state()
            .unwrap()
            .incoming(4, target_request.with_run_mode(RunMode::PerformLayout))
            .is_none()
    );
    let output = trial.compute_child(4, target_request).unwrap();
    assert_eq!(output.size, Size::new(S::from_f64(10.0), S::from_f64(5.0)));
    let completed = trial
        .block_float_state()
        .unwrap()
        .pop(4, target_request)
        .unwrap();
    assert_eq!(completed.output, output);
    assert_eq!(completed.inherited, 1);
    assert!(std::sync::Arc::ptr_eq(
        completed.frame.as_ref().unwrap(),
        &target.frame
    ));
    let restored = trial
        .block_float_state()
        .unwrap()
        .incoming(2, resident_request)
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&restored.frame, &resident.frame));
    drop(trial);
    assert_eq!(owner.staged_unrounded(4).unwrap(), prior);
    let retained = owner
        .block_float_state()
        .unwrap()
        .incoming(2, resident_request)
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&retained.frame, &resident.frame));

    // Existing unframed trials keep their independent float-state policy.
    let mut detached = owner
        .child_staging(4, FloatChildTrial::Detached, None)
        .unwrap();
    assert!(
        detached
            .block_float_state()
            .unwrap()
            .incoming(2, resident_request)
            .is_none()
    );
    assert!(matches!(
        owner
            .child_staging(4, FloatChildTrial::Detached, Some(target))
            .err()
            .unwrap()
            .kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
    assert!(matches!(
        owner
            .child_staging(4, FloatChildTrial::InFlow, Some(resident))
            .err()
            .unwrap()
            .kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
}

#[test]
fn framed_child_staging_keeps_exact_resident_custody_without_publishing_f32() {
    framed::<f32>();
}

#[test]
fn framed_child_staging_keeps_exact_resident_custody_without_publishing_f64() {
    framed::<f64>();
}
