use super::*;
use crate::{
    ContextEffect, ContextView, EventFilter, PaintChange, PaintEvent, PaintLimits, PaintPhase,
    Position, ProfileMode, ZIndex,
};
use surgeist_layout::{ItemOrder, SourceIndex};
use surgeist_ledger::RefusalReason;

const STACKING_WORK_KINDS: [WorkKind; 8] = [
    WorkKind::BoxValidation,
    WorkKind::ParentEdge,
    WorkKind::TopologyVisit,
    WorkKind::SiblingComparison,
    WorkKind::MembershipVisit,
    WorkKind::ContextComparison,
    WorkKind::OrderMove,
    WorkKind::OutputRecord,
];

fn policy(work: [u64; 8], resources: [u64; 2], profile: ProfileMode) -> PaintOptions {
    let mut limits = PaintLimits::unlimited();
    for (kind, limit) in STACKING_WORK_KINDS.into_iter().zip(work) {
        limits = limits.with_work_limit(kind, limit);
    }
    for (kind, limit) in ResourceKind::ALL.iter().copied().zip(resources) {
        limits = limits.with_resource_limit(kind, limit);
    }
    PaintOptions::new(limits, profile)
}
fn options() -> PaintOptions {
    policy(
        [u64::MAX; 8],
        [u64::MAX; 2],
        ProfileMode::Summary { timing: false },
    )
}
fn facts(parent: Option<usize>, source: usize, position: Position, z: ZIndex) -> BoxFacts {
    BoxFacts::new(
        parent.map(BoxId::new),
        SourceIndex::new(source),
        ChildOrder::Source,
        Participation::Ordinary,
        position,
        z,
        ContextEffect::None,
    )
}
fn root() -> BoxFacts {
    facts(None, 0, Position::Static, ZIndex::Auto)
}
fn child(parent: usize, source: usize, level: i64) -> BoxFacts {
    facts(
        Some(parent),
        source,
        Position::Relative,
        ZIndex::Integer(level),
    )
}
fn children(context: ContextView<'_>) -> Vec<usize> {
    context
        .children()
        .map(|child| child.box_id().get())
        .collect()
}
fn membership_ids(plan: &StackingPlan<'_>) -> Vec<usize> {
    (0..plan.facts().len())
        .map(|i| plan.context_for_box(BoxId::new(i)).unwrap().box_id().get())
        .collect()
}
fn assert_failure(boxes: &[BoxFacts], root: usize, expected: PaintError) {
    let outcome = plan_stacking(boxes, BoxId::new(root), options());
    assert_eq!(outcome.result.unwrap_err(), expected);
    let report = outcome.report.unwrap();
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
}

#[test]
fn refused_traversal_consume_keeps_pending_root_until_cleanup_and_retry() {
    let boxes = [root()];
    // Topology admits node/merge initialization, the outer visit and parent
    // visit (4) before draining. Its drain and three indexing passes plus the
    // sibling visit bring the total to 9; membership's initial push makes 10.
    for (limit, phase, color) in [
        (4, PaintPhase::Topology, 1),
        (10, PaintPhase::Membership, 2),
    ] {
        let options = options()
            .with_limits(PaintLimits::unlimited().with_work_limit(WorkKind::TopologyVisit, limit));
        let mut meter = options.operation(OperationKind::Stacking).unwrap();
        let root = meter.root();
        let scopes = setup_scopes(&mut meter).unwrap();
        let mut state = State::default();
        let error =
            prepare(&mut meter, root, &scopes, &mut state, &boxes, BoxId::new(0)).unwrap_err();
        let PaintError::Budget(refusal) = error else {
            panic!("expected traversal admission refusal");
        };
        assert_eq!(refusal.origin, phase);
        assert_eq!(refusal.current, limit);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::TopologyVisit,
                amount: 1
            }
        );
        assert_eq!(state.stack, [0]);
        assert!(state.stack.capacity() >= 1);
        assert_eq!(state.nodes[0].color, color);
        assert_eq!(state.nodes[0].rank, 0);
        assert!(state.contexts.is_empty());
        assert!(state.children.is_empty());
        if phase == PaintPhase::Topology {
            assert!(state.membership.is_empty());
        } else {
            assert_eq!(state.membership, [0]);
            assert!(state.contexts.capacity() >= 1);
        }
        meter
            .with_phase(scopes[6], |p| {
                state.scratch_cleanup(p, root).unwrap();
                state.result_cleanup(p, root).unwrap();
            })
            .unwrap();
        let report = meter.finish().unwrap().unwrap();
        assert_eq!(report.work(WorkKind::TopologyVisit), limit);
        assert_eq!(report.work(WorkKind::MembershipVisit), 0);
        assert!(report.peak(ResourceKind::ScratchBytes) > 0);
        for &resource in ResourceKind::ALL {
            assert_eq!(report.live(resource), 0);
        }
        let retry = plan_stacking(&boxes, BoxId::new(0), self::options());
        let plan = retry.result.unwrap();
        assert_eq!(membership_ids(&plan), [0]);
        assert_eq!(plan.root_context().box_id(), BoxId::new(0));
        assert!(children(plan.root_context()).is_empty());
    }
}

#[test]
fn root_and_nested_signed_contexts_keep_descendants_inside_actual_contexts() {
    let boxes = [
        root(),
        child(0, 2, 3),
        child(0, 0, -2),
        child(2, 0, i64::MAX),
        facts(Some(0), 1, Position::Relative, ZIndex::Auto),
        child(4, 0, i64::MIN),
        child(0, 3, 0),
        child(0, 4, 3),
    ];
    let outcome = plan_stacking(&boxes, BoxId::new(0), options());
    let plan = outcome.result.unwrap();
    assert_eq!(children(plan.root_context()), [5, 2, 6, 1, 7]);
    assert_eq!(children(plan.context_for_box(BoxId::new(2)).unwrap()), [3]);
    assert_eq!(membership_ids(&plan), [0, 1, 2, 3, 0, 5, 6, 7]);
    assert_eq!(
        plan.context_for_box(BoxId::new(3))
            .unwrap()
            .parent()
            .unwrap()
            .box_id(),
        BoxId::new(2)
    );
    assert!(plan.root_context().parent().is_none());
    assert!(plan.context_for_box(BoxId::new(99)).is_none());
    assert_eq!(
        plan.context_for_box(BoxId::new(5)).unwrap().stack_level(),
        i64::MIN
    );
}

#[test]
fn positioned_auto_static_inapplicability_and_effect_contexts_have_distinct_membership() {
    let mut effect = facts(Some(0), 4, Position::Static, ZIndex::Integer(-999));
    effect.effects = ContextEffect::EstablishesContext;
    let boxes = [
        root(),
        facts(Some(0), 0, Position::Relative, ZIndex::Auto),
        facts(Some(0), 1, Position::Absolute, ZIndex::Auto),
        facts(Some(0), 2, Position::Fixed, ZIndex::Auto),
        facts(Some(0), 3, Position::Sticky, ZIndex::Auto),
        effect,
        facts(Some(0), 5, Position::Static, ZIndex::Integer(7)),
        child(1, 0, 0),
        child(2, 0, 0),
    ];
    let plan = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap();
    assert_eq!(membership_ids(&plan), [0, 0, 0, 3, 4, 5, 0, 7, 8]);
    assert_eq!(children(plan.root_context()), [7, 8, 3, 4, 5]);
    assert_eq!(
        plan.context_for_box(BoxId::new(5)).unwrap().stack_level(),
        0
    );
}

#[test]
fn flex_and_grid_traversal_reorders_descendants_and_out_of_flow_children_use_order_zero() {
    for mode in [ChildOrder::Flex, ChildOrder::Grid] {
        let item = |order| match mode {
            ChildOrder::Flex => Participation::FlexItem(ItemOrder::new(order)),
            _ => Participation::GridItem(ItemOrder::new(order)),
        };
        let mut parent = root();
        parent.child_order = mode;
        // Slice order differs from source order and effective item order.
        let mut early = facts(Some(0), 3, Position::Static, ZIndex::Auto);
        early.participation = item(-1);
        let mut late = facts(Some(0), 0, Position::Static, ZIndex::Integer(0));
        late.participation = item(1);
        let absolute = facts(Some(0), 2, Position::Absolute, ZIndex::Integer(0));
        let fixed = facts(Some(0), 1, Position::Fixed, ZIndex::Auto);
        let boxes = [
            parent,
            late,
            absolute,
            fixed,
            early,
            child(4, 0, 0),
            child(1, 0, -1),
        ];
        let plan = plan_stacking(&boxes, BoxId::new(0), options())
            .result
            .unwrap();
        assert_eq!(children(plan.root_context()), [5, 3, 2, 1]);
        assert_eq!(membership_ids(&plan), [0, 1, 2, 3, 0, 5, 6]);
        assert_eq!(children(plan.context_for_box(BoxId::new(1)).unwrap()), [6]);
    }
}

#[test]
fn arbitrary_forward_parent_references_and_nonzero_root_preserve_source_ties() {
    let boxes = [child(3, 9, 0), child(3, 2, 0), child(0, 0, 0), root()];
    let plan = plan_stacking(&boxes, BoxId::new(3), options())
        .result
        .unwrap();
    assert_eq!(children(plan.root_context()), [1, 0]);
    assert_eq!(children(plan.context_for_box(BoxId::new(0)).unwrap()), [2]);
}

#[test]
fn malformed_topology_and_participation_refuse_without_partial_custody() {
    assert_failure(&[], 0, PaintError::MissingRoot);
    assert_failure(&[root()], 1, PaintError::MissingRoot);
    assert_failure(
        &[root(), root()],
        0,
        PaintError::MultipleRoots {
            first: BoxId::new(0),
            other: BoxId::new(1),
        },
    );
    assert_failure(
        &[root(), child(9, 0, 0)],
        0,
        PaintError::MissingParent {
            child: BoxId::new(1),
            parent: BoxId::new(9),
        },
    );
    assert_failure(
        &[root(), child(1, 0, 0)],
        0,
        PaintError::SelfParent {
            box_id: BoxId::new(1),
        },
    );
    assert_failure(
        &[root(), child(2, 0, 0), child(1, 0, 0)],
        0,
        PaintError::Cycle {
            box_id: BoxId::new(1),
        },
    );
    assert_failure(&[root(), child(0, 0, 0)], 1, PaintError::MissingRoot);
    assert_failure(
        &[root(), child(0, 2, 0), child(0, 2, 1)],
        0,
        PaintError::DuplicateSourceIndex {
            parent: BoxId::new(0),
            source_index: SourceIndex::new(2),
        },
    );
    let mut parent = root();
    parent.child_order = ChildOrder::Flex;
    let mut item = child(0, 0, 0);
    item.participation = Participation::GridItem(ItemOrder::ZERO);
    assert_failure(
        &[parent, item],
        0,
        PaintError::IncompatibleParticipation {
            box_id: BoxId::new(1),
        },
    );
    item.participation = Participation::FlexItem(ItemOrder::ZERO);
    item.position = Position::Absolute;
    assert_failure(
        &[parent, item],
        0,
        PaintError::IncompatibleParticipation {
            box_id: BoxId::new(1),
        },
    );
    item.position = Position::Relative;
    assert_failure(
        &[root(), item],
        0,
        PaintError::IncompatibleParticipation {
            box_id: BoxId::new(1),
        },
    );
    let mut another = item;
    another.source_index = SourceIndex::ZERO;
    another.participation = Participation::FlexItem(ItemOrder::new(-9));
    assert_failure(
        &[parent, item, another],
        0,
        PaintError::DuplicateSourceIndex {
            parent: BoxId::new(0),
            source_index: SourceIndex::ZERO,
        },
    );
}

// Hand-traced three-box algorithm: initialization visits 8; parent walks 11;
// parent-count visits 3; adjacency prefix/fill 6; sibling visits 3;
// membership stack actions 6; context grouping 5 => 42.
// Each two-entry merge uses one compare/four moves;
// source duplicate check adds one compare. Output writes: 3 initialization,
// 3 membership, 3 context creation, 2 child references, 3 group records => 14.
const THREE_BOX_WORK: [u64; 8] = [3, 2, 42, 2, 3, 1, 8, 14];
fn three_box_bytes() -> [u64; 2] {
    // Exact independent inventory: three node-state slots, eight scratch
    // usize slots; three context slots, five retained usize slots, one plan.
    [
        3 * size_of::<NodeScratch>() as u64 + 8 * size_of::<usize>() as u64,
        3 * size_of::<ContextRecord>() as u64
            + 5 * size_of::<usize>() as u64
            + size_of::<StackingPlan<'_>>() as u64,
    ]
}
#[test]
fn exact_work_and_storage_caps_succeed_and_every_one_under_cap_refuses_cleanly() {
    let boxes = [root(), child(0, 0, -1), child(0, 1, 1)];
    let caps = three_box_bytes();
    let exact = policy(THREE_BOX_WORK, caps, ProfileMode::Summary { timing: false });
    let outcome = plan_stacking(&boxes, BoxId::new(0), exact);
    assert_eq!(children(outcome.result.unwrap().root_context()), [1, 2]);
    let report = outcome.report.unwrap();
    for (kind, expected) in STACKING_WORK_KINDS.into_iter().zip(THREE_BOX_WORK) {
        assert_eq!(report.work(kind), expected, "{kind:?}");
    }
    for &kind in WorkKind::ALL {
        if !STACKING_WORK_KINDS.contains(&kind) {
            assert_eq!(report.work(kind), 0, "{kind:?}");
        }
    }
    assert_eq!(report.peak(ResourceKind::ScratchBytes), caps[0]);
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.peak(ResourceKind::RetainedResultBytes), caps[1]);
    assert_eq!(report.live(ResourceKind::RetainedResultBytes), caps[1]);
    for (kind, expected) in STACKING_WORK_KINDS.into_iter().zip(THREE_BOX_WORK) {
        let outcome = plan_stacking(
            &boxes,
            BoxId::new(0),
            exact.with_limits(exact.limits().with_work_limit(kind, expected - 1)),
        );
        let PaintError::Budget(refusal) = outcome.result.unwrap_err() else {
            panic!("expected work refusal");
        };
        assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
        assert!(matches!(refusal.reason, RefusalReason::Limit { .. }));
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        let expected_origin = match kind {
            WorkKind::BoxValidation => PaintPhase::Validate,
            WorkKind::ParentEdge => PaintPhase::Topology,
            WorkKind::SiblingComparison => PaintPhase::SiblingOrder,
            WorkKind::MembershipVisit => PaintPhase::Membership,
            _ => PaintPhase::ContextOrder,
        };
        assert_eq!(refusal.origin, expected_origin);
        let report = outcome.report.unwrap();
        assert!(report.work(WorkKind::BoxValidation) > 0);
        for &resource in ResourceKind::ALL {
            assert_eq!(report.live(resource), 0);
        }
        assert!(report.refusals().next().is_some());
    }
    for &kind in ResourceKind::ALL {
        let outcome = plan_stacking(
            &boxes,
            BoxId::new(0),
            exact.with_limits(
                exact
                    .limits()
                    .with_resource_limit(kind, caps[kind as usize] - 1),
            ),
        );
        assert!(matches!(outcome.result, Err(PaintError::Budget(_))));
        let report = outcome.report.unwrap();
        for &resource in ResourceKind::ALL {
            assert_eq!(report.live(resource), 0);
        }
        assert!(report.peak(kind) < caps[kind as usize]);
    }
}

#[test]
fn actual_failed_reservation_records_no_accepted_resource_peak() {
    let mut meter = options().operation(OperationKind::Stacking).unwrap();
    let root = meter.root();
    let result = meter
        .with_phase(root, |p| {
            allocate::<u8>(p, root, isize::MAX as usize + 1, ResourceKind::ScratchBytes)
        })
        .unwrap();
    assert_eq!(
        result.unwrap_err(),
        PaintError::Allocation {
            resource: ResourceKind::ScratchBytes
        }
    );
    let report = meter.finish().unwrap().unwrap();
    assert_eq!(report.peak(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.phases().next().unwrap().accepted_batches(), 0);
}

#[test]
fn paint_order_work_limits_preserve_named_admissions_refusals_and_reports() {
    for (kind, name, phase) in [
        (
            WorkKind::PaintValidation,
            "paint_validate",
            PaintPhase::PaintValidation,
        ),
        (
            WorkKind::FragmentVisit,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::LineVisit,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::InlineMemberVisit,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::PrimitiveVisit,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::PaintScopeVisit,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::PaintOrderComparison,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
        (
            WorkKind::DecorationComparison,
            "paint_sequence",
            PaintPhase::PaintSequence,
        ),
    ] {
        let limits = PaintLimits::unlimited().with_work_limit(kind, 2);
        assert_eq!(limits.work_limit(kind), 2);
        let mut meter = PaintOptions::new(
            limits,
            ProfileMode::Detailed {
                timing: false,
                event_limit: 8,
                filter: EventFilter::All,
            },
        )
        .operation(OperationKind::PaintOrder)
        .unwrap();
        let root = meter.root();
        assert_eq!(meter.scope_name(root).unwrap(), "paint order");
        let scope = meter
            .child(root, phase, Limits::unlimited(), Tariffs::identity())
            .unwrap();
        assert_eq!(meter.scope_name(scope).unwrap(), name);
        meter
            .with_phase(scope, |p| charge(p, scope, kind, 2))
            .unwrap()
            .unwrap();
        let error = meter
            .with_phase(scope, |p| charge(p, scope, kind, 1))
            .unwrap()
            .unwrap_err();
        let PaintError::Budget(refusal) = error else {
            panic!("expected work refusal");
        };
        assert_eq!(refusal.origin, phase);
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
        assert_eq!(refusal.current, 2);
        let report = meter.finish().unwrap().unwrap();
        for &dimension in WorkKind::ALL {
            assert_eq!(
                report.work(dimension),
                if dimension == kind { 2 } else { 0 }
            );
        }
        let phase_report = report
            .phases()
            .find(|entry| entry.phase() == phase)
            .unwrap();
        assert_eq!(phase_report.work(kind), 2);
        assert_eq!(phase_report.refused_batches(), 1);
        assert!(report.refusals().any(|entry| entry == refusal));
        assert!(report.events().any(|event| event
            == PaintEvent::Accepted {
                phase,
                change: PaintChange::Work { kind, amount: 2 },
                elapsed: None,
            }));
        assert!(report.events().any(|event| event
            == PaintEvent::Refused {
                refusal,
                elapsed: None,
            }));
    }
}

#[test]
fn paint_order_reserves_scopes_for_stacking_and_both_paint_phases() {
    let mut meter = options().operation(OperationKind::PaintOrder).unwrap();
    setup_scopes(&mut meter).unwrap();
    for phase in [PaintPhase::PaintValidation, PaintPhase::PaintSequence] {
        let scope = meter
            .child(
                meter.root(),
                phase,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        meter.with_phase(scope, |_| ()).unwrap();
    }
    assert!(matches!(
        meter.child(
            meter.root(),
            PaintPhase::Publication,
            Limits::unlimited(),
            Tariffs::identity()
        ),
        Err(PaintError::Accounting(
            crate::AccountingFailure::ScopeLimit { limit: 10 }
        ))
    ));
    let report = meter.finish().unwrap().unwrap();
    for phase in [PaintPhase::PaintValidation, PaintPhase::PaintSequence] {
        assert_eq!(
            report
                .phases()
                .find(|entry| entry.phase() == phase)
                .unwrap()
                .invocations(),
            1
        );
    }
}

static CLOCK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn clock() -> std::time::Duration {
    std::time::Duration::from_nanos(CLOCK.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}
#[test]
fn fixed_phases_measure_algorithm_execution_and_detail_loss_never_changes_admission() {
    let boxes = [root(), child(0, 0, -1), child(0, 1, 1)];
    let summary = options().with_clock(clock);
    let timed = policy(
        THREE_BOX_WORK,
        three_box_bytes(),
        ProfileMode::Detailed {
            timing: true,
            event_limit: 1,
            filter: EventFilter::All,
        },
    )
    .with_clock(clock);
    let outcome = plan_stacking(&boxes, BoxId::new(0), timed);
    let report = outcome.report.unwrap();
    assert!(report.elapsed().unwrap() > std::time::Duration::ZERO);
    assert_eq!(report.phases().len(), 8);
    assert!(report.dropped_events() > 0);
    for scope in report
        .phases()
        .filter(|scope| scope.phase() != PaintPhase::Operation)
    {
        assert_eq!(scope.invocations(), 1, "{:?}", scope.phase());
        assert!(scope.elapsed().unwrap() > std::time::Duration::ZERO);
    }
    for (kind, expected) in STACKING_WORK_KINDS.into_iter().zip(THREE_BOX_WORK) {
        assert_eq!(report.work(kind), expected);
    }
    assert_eq!(children(outcome.result.unwrap().root_context()), [1, 2]);
    let untimed = plan_stacking(&boxes, BoxId::new(0), summary);
    let off = plan_stacking(
        &boxes,
        BoxId::new(0),
        policy(THREE_BOX_WORK, three_box_bytes(), ProfileMode::Off),
    );
    assert!(off.report.is_none());
    assert_eq!(
        membership_ids(&off.result.unwrap()),
        membership_ids(&untimed.result.unwrap())
    );
}

#[test]
fn deep_and_wide_topologies_use_bounded_iterative_storage() {
    const N: usize = 20_000;
    let mut boxes = Vec::with_capacity(N);
    boxes.push(root());
    for i in 1..N {
        boxes.push(child(i - 1, 0, 0));
    }
    let plan = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap();
    let mut view = plan.root_context();
    let mut count = 1;
    while let Some(child) = view.children().next() {
        count += 1;
        view = child;
    }
    assert_eq!(count, N);
    for (i, item) in boxes.iter_mut().enumerate().skip(1) {
        *item = child(0, N - i, 0);
    }
    let plan = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap();
    assert_eq!(
        children(plan.root_context()),
        (1..N).rev().collect::<Vec<_>>()
    );
}

#[test]
fn root_only_ignores_its_own_stack_level_and_failed_input_can_be_retried() {
    let mut only = root();
    only.z_index = ZIndex::Integer(i64::MIN);
    let boxes = [only];
    let failed = plan_stacking(
        &boxes,
        BoxId::new(0),
        options()
            .with_limits(PaintLimits::unlimited().with_work_limit(WorkKind::MembershipVisit, 0)),
    );
    assert!(matches!(failed.result, Err(PaintError::Budget(_))));
    let plan = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap();
    assert_eq!(plan.root_context().stack_level(), 0);
    assert_eq!(plan.context_count(), 1);
    assert_eq!(plan.root_context().children().len(), 0);
    assert_eq!(plan.facts(), boxes);
}

#[test]
fn detailed_failures_and_phase_work_use_only_named_public_views() {
    let boxes = [root(), child(0, 0, 0)];
    let limits = PaintLimits::unlimited().with_work_limit(WorkKind::MembershipVisit, 0);
    assert_eq!(limits.work_limit(WorkKind::MembershipVisit), 0);
    assert_eq!(limits.resource_limit(ResourceKind::ScratchBytes), u64::MAX);
    let outcome = plan_stacking(
        &boxes,
        BoxId::new(0),
        PaintOptions::new(
            limits,
            ProfileMode::Detailed {
                timing: false,
                event_limit: 1000,
                filter: EventFilter::All,
            },
        ),
    );
    let PaintError::Budget(expected) = outcome.result.unwrap_err() else {
        panic!("budget refusal");
    };
    let report = outcome.report.unwrap();
    assert_eq!(report.refusals().collect::<Vec<_>>(), [expected]);
    assert!(report.events().any(
        |event| matches!(event, crate::PaintEvent::Refused { refusal, .. } if refusal == expected)
    ));
    assert!(report.events().any(|event| matches!(
        event,
        crate::PaintEvent::Accepted {
            phase: PaintPhase::Validate,
            change: PaintChange::Work {
                kind: WorkKind::BoxValidation,
                amount: 1
            },
            ..
        }
    )));
    let validation = report
        .phases()
        .find(|phase| phase.phase() == PaintPhase::Validate)
        .unwrap();
    assert_eq!(validation.work(WorkKind::BoxValidation), 2);
    assert_eq!(validation.elapsed(), None);
    assert!(!report.saturated());
}
