use super::*;
use crate::bounded::values::HtmlStringPool;
use surgeist_ledger::{Dimension, Error, Operation, RefusalReason};

type Op = Operation<HtmlWork, HtmlResource>;

/// A cap scope with the given caps and one charging scope below it.
fn charging(limits: HtmlLimits) -> (Op, surgeist_ledger::Scope, surgeist_ledger::Scope) {
    let mut operation = Operation::new(HtmlCapScope::Parse.name(), limits, 2).unwrap();
    let cap = operation.root();
    let charging = operation
        .child(
            cap,
            "html-parse-work",
            HtmlLimits::unlimited(),
            charging_tariffs(),
        )
        .unwrap();
    (operation, cap, charging)
}

#[test]
fn parser_categories_convert_to_credits_at_rate_one() {
    let (mut operation, cap, charging) = charging(HtmlLimits::unlimited());
    let categories = [
        (HtmlWork::InputUnitRead, 3),
        (HtmlWork::InspectByte, 5),
        (HtmlWork::CopyByte, 7),
        (HtmlWork::StateStep, 11),
        (HtmlWork::RecordVisit, 13),
        (HtmlWork::RecordWrite, 17),
        (HtmlWork::ReserveByte, 19),
        (HtmlWork::ProviderCall, 23),
    ];
    let batch: Vec<HtmlChange> = categories
        .iter()
        .map(|&(dimension, amount)| Change::Work(dimension, amount))
        .collect();
    operation.apply_batch(charging, &batch).unwrap();
    let expected_total: u64 = categories.iter().map(|&(_, amount)| amount).sum();
    assert_eq!(expected_total, 98);
    assert_eq!(
        operation.usage(cap).unwrap().work(HtmlWork::Credits),
        expected_total
    );
    for (dimension, amount) in categories {
        assert_eq!(operation.usage(charging).unwrap().work(dimension), amount);
        assert_eq!(operation.usage(cap).unwrap().work(dimension), 0);
    }
}

#[test]
fn semantic_steps_do_not_convert_to_credits() {
    let (mut operation, cap, charging) = charging(HtmlLimits::unlimited());
    operation
        .apply(charging, Change::Work(HtmlWork::SemanticStep, 9))
        .unwrap();
    assert_eq!(
        operation.usage(cap).unwrap().work(HtmlWork::SemanticStep),
        9
    );
    assert_eq!(operation.usage(cap).unwrap().work(HtmlWork::Credits), 0);
}

#[test]
fn the_credit_cap_refuses_the_charge_that_exceeds_it() {
    let limits = Limits::from_fn(
        |work| {
            if work == HtmlWork::Credits {
                10
            } else {
                u64::MAX
            }
        },
        |_| u64::MAX,
    );
    let (mut operation, cap, charging) = charging(limits);
    operation
        .apply_batch(
            charging,
            &[
                Change::Work(HtmlWork::RecordVisit, 4),
                Change::Work(HtmlWork::CopyByte, 5),
            ],
        )
        .unwrap();
    let refusal = operation
        .apply_batch(
            charging,
            &[
                Change::Work(HtmlWork::InspectByte, 1),
                Change::Work(HtmlWork::ProviderCall, 1),
            ],
        )
        .unwrap_err();
    let Error::Refused(refusal) = refusal else {
        panic!("expected a refusal, got {refusal:?}");
    };
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 10 });
    assert_eq!(refusal.item, 1);
    assert_eq!(operation.usage(cap).unwrap().work(HtmlWork::Credits), 9);
}

#[test]
fn value_pool_work_charges_the_sum_of_its_five_fields_as_credits() {
    for work in [
        HtmlStringPool::root_work(),
        HtmlStringPool::mark_work(),
        HtmlStringPool::seal_work(),
        HtmlStringPool::disposal_work(),
    ] {
        let expected = work.record_visits() as u64
            + work.inspected_bytes() as u64
            + work.copied_bytes() as u64
            + work.provider_calls() as u64
            + work.reserve_bytes() as u64;
        let (mut operation, cap, charging) = charging(HtmlLimits::unlimited());
        work.charge(&mut operation.charge_at(charging)).unwrap();
        assert_eq!(
            operation.usage(cap).unwrap().work(HtmlWork::Credits),
            expected
        );
    }
}

#[test]
fn a_gauge_records_the_largest_value_and_checks_the_cap_against_the_new_value() {
    let limits = Limits::from_fn(
        |_| u64::MAX,
        |resource| {
            if resource == HtmlResource::TreeDepth {
                8
            } else {
                u64::MAX
            }
        },
    );
    let (mut operation, cap, charging) = charging(limits);
    let mut held = 0;
    for value in [5, 3, 8] {
        operation
            .apply_batch(
                charging,
                &record_largest(HtmlResource::TreeDepth, held, value),
            )
            .unwrap();
        held = value;
    }
    let usage = *operation.usage(cap).unwrap();
    assert_eq!(usage.live(HtmlResource::TreeDepth), 8);
    assert_eq!(usage.peak(HtmlResource::TreeDepth), 8);
    operation
        .apply_batch(charging, &record_largest(HtmlResource::TreeDepth, 8, 2))
        .unwrap();
    let usage = *operation.usage(cap).unwrap();
    assert_eq!(usage.live(HtmlResource::TreeDepth), 2);
    assert_eq!(usage.peak(HtmlResource::TreeDepth), 8);
    let refusal = operation
        .apply_batch(charging, &record_largest(HtmlResource::TreeDepth, 2, 9))
        .unwrap_err();
    let Error::Refused(refusal) = refusal else {
        panic!("expected a refusal, got {refusal:?}");
    };
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 8 });
    assert_eq!(
        operation.usage(cap).unwrap().live(HtmlResource::TreeDepth),
        2
    );
}

#[test]
fn gauges_stay_in_their_cap_scope_when_a_host_aggregates() {
    let mut host = Op::new("host", HtmlLimits::unlimited(), 3).unwrap();
    let host_root = host.root();
    let cap = host
        .child(
            host_root,
            "html-parse",
            HtmlLimits::unlimited(),
            aggregate_tariffs(),
        )
        .unwrap();
    let charging = host
        .child(
            cap,
            "html-parse-work",
            HtmlLimits::unlimited(),
            charging_tariffs(),
        )
        .unwrap();
    host.apply_batch(
        charging,
        &[
            Change::Acquire(HtmlResource::ManagedNodes, 4),
            Change::Acquire(HtmlResource::Diagnostics, 2),
            Change::Work(HtmlWork::RecordVisit, 6),
        ],
    )
    .unwrap();
    host.apply_batch(charging, &record_largest(HtmlResource::TreeDepth, 0, 7))
        .unwrap();
    host.apply_batch(
        charging,
        &record_largest(HtmlResource::AtomicLayoutBytes, 0, 64),
    )
    .unwrap();
    let at_cap = *host.usage(cap).unwrap();
    assert_eq!(at_cap.peak(HtmlResource::TreeDepth), 7);
    assert_eq!(at_cap.peak(HtmlResource::AtomicLayoutBytes), 64);
    let at_host = *host.usage(host_root).unwrap();
    assert_eq!(at_host.live(HtmlResource::ManagedNodes), 4);
    assert_eq!(at_host.live(HtmlResource::Diagnostics), 2);
    assert_eq!(at_host.work(HtmlWork::Credits), 6);
    for gauge in HtmlResource::ALL
        .iter()
        .copied()
        .filter(|resource| resource.custody() == HtmlCustody::Gauge)
    {
        assert_eq!(at_host.live(gauge), 0, "{gauge:?}");
        assert_eq!(at_host.peak(gauge), 0, "{gauge:?}");
    }
}

#[test]
fn a_retention_cap_refuses_the_acquisition_that_reaches_it_without_changing_usage() {
    let limits = Limits::from_fn(
        |_| u64::MAX,
        |resource| {
            if resource == HtmlResource::Diagnostics {
                2
            } else {
                u64::MAX
            }
        },
    );
    let (mut operation, cap, charging) = charging(limits);
    for _ in 0..2 {
        operation
            .apply(charging, Change::Acquire(HtmlResource::Diagnostics, 1))
            .unwrap();
    }
    let refusal = operation
        .apply(charging, Change::Acquire(HtmlResource::Diagnostics, 1))
        .unwrap_err();
    assert!(matches!(
        refusal,
        Error::Refused(refusal) if refusal.reason == RefusalReason::Limit { limit: 2 }
    ));
    assert_eq!(
        operation
            .usage(cap)
            .unwrap()
            .live(HtmlResource::Diagnostics),
        2
    );
}

#[test]
fn only_categories_and_the_formation_backing_lack_a_direct_cap() {
    // The eight parser categories are capped through their conversion into
    // credits, and the formation backing is a balance the formation totals.
    let uncapped_work: Vec<HtmlWork> = HtmlWork::ALL
        .iter()
        .copied()
        .filter(|work| {
            !HtmlCapScope::ALL
                .iter()
                .any(|scope| scope.capped_work().contains(work))
        })
        .collect();
    assert_eq!(
        uncapped_work,
        [
            HtmlWork::InputUnitRead,
            HtmlWork::InspectByte,
            HtmlWork::CopyByte,
            HtmlWork::StateStep,
            HtmlWork::RecordVisit,
            HtmlWork::RecordWrite,
            HtmlWork::ReserveByte,
            HtmlWork::ProviderCall,
        ]
    );
    let uncapped: Vec<HtmlResource> = HtmlResource::ALL
        .iter()
        .copied()
        .filter(|resource| {
            !HtmlCapScope::ALL
                .iter()
                .any(|scope| scope.capped_resources().contains(resource))
        })
        .collect();
    assert_eq!(uncapped, [HtmlResource::FormationBacking]);
}

#[test]
fn a_cap_scope_lists_each_dimension_once_and_has_a_unique_name() {
    for scope in HtmlCapScope::ALL {
        let work = scope.capped_work();
        let resources = scope.capped_resources();
        for (index, dimension) in work.iter().enumerate() {
            assert!(
                !work[index + 1..].contains(dimension),
                "{scope:?} {dimension:?}"
            );
        }
        for (index, dimension) in resources.iter().enumerate() {
            assert!(
                !resources[index + 1..].contains(dimension),
                "{scope:?} {dimension:?}"
            );
        }
        assert_eq!(
            HtmlCapScope::ALL
                .iter()
                .filter(|other| other.name() == scope.name())
                .count(),
            1
        );
    }
}

#[test]
fn the_parse_scope_caps_every_parser_resource_and_credits() {
    assert_eq!(HtmlCapScope::Parse.capped_work(), [HtmlWork::Credits]);
    assert_eq!(
        HtmlCapScope::Parse.capped_resources(),
        &HtmlResource::ALL[..19],
        "the parse resources are the first nineteen, in precedence order"
    );
}

#[test]
fn construction_limits_cap_the_five_arena_dimensions_in_argument_order() {
    let limits = construction_limits(HtmlConstructionLimits::new(1, 2, 3, 4, 5));
    let capped = [
        (HtmlResource::ManagedNodes, 1),
        (HtmlResource::ManagedEdges, 2),
        (HtmlResource::ManagedAttributes, 3),
        (HtmlResource::ManagedUtf16Units, 4),
        (HtmlResource::TreeDepth, 5),
    ];
    for resource in HtmlResource::ALL {
        let expected = capped
            .iter()
            .find(|&&(dimension, _)| dimension == *resource)
            .map_or(u64::MAX, |&(_, cap)| cap);
        assert_eq!(limits.live(*resource), expected, "{resource:?}");
    }
    for work in HtmlWork::ALL {
        assert_eq!(limits.work(*work), u64::MAX, "{work:?}");
    }
}

#[test]
fn autocomplete_limits_cap_bytes_nested_output_and_scratch() {
    let limits = autocomplete_limits(HtmlAutocompleteMaterializationLimits {
        allocated_bytes: 11,
        nested_output_records: 22,
        scratch_records: 33,
    });
    assert_eq!(limits.live(HtmlResource::AllocatedBytes), 11);
    assert_eq!(limits.live(HtmlResource::NestedOutputRecords), 22);
    assert_eq!(limits.live(HtmlResource::ScratchRecords), 33);
    assert_eq!(limits.live(HtmlResource::OutputRecords), u64::MAX);
}

/// The schema dimension of each construction limit resource.
fn construction(resource: crate::construction::HtmlConstructionResource) -> HtmlResource {
    use crate::construction::HtmlConstructionResource as Resource;
    match resource {
        Resource::Nodes => HtmlResource::ManagedNodes,
        Resource::Edges => HtmlResource::ManagedEdges,
        Resource::Attributes => HtmlResource::ManagedAttributes,
        Resource::Utf16Units => HtmlResource::ManagedUtf16Units,
        Resource::Depth => HtmlResource::TreeDepth,
    }
}

#[test]
fn the_construction_scope_caps_exactly_the_construction_resources() {
    use crate::construction::HtmlConstructionResource as Resource;
    let capped: Vec<HtmlResource> = [
        Resource::Nodes,
        Resource::Edges,
        Resource::Attributes,
        Resource::Utf16Units,
        Resource::Depth,
    ]
    .into_iter()
    .map(construction)
    .collect();
    assert_eq!(
        HtmlCapScope::Construction.capped_resources(),
        capped.as_slice()
    );
}

/// The field names of a limits value, read from its `Debug` text.
pub(crate) fn field_names(limits: &impl std::fmt::Debug) -> Vec<String> {
    let text = format!("{limits:?}");
    let body = text
        .split_once('{')
        .and_then(|(_, rest)| rest.rsplit_once('}'))
        .map_or("", |(body, _)| body);
    body.split(',')
        .filter_map(|field| field.split_once(':'))
        .map(|(name, _)| name.trim().to_owned())
        .collect()
}

#[test]
fn the_construction_limits_fields_are_the_construction_scope_caps() {
    let limits = crate::construction::HtmlConstructionLimits::new(1, 2, 3, 4, 5);
    let capped: Vec<HtmlResource> = field_names(&limits)
        .iter()
        .map(|name| match name.as_str() {
            "nodes" => HtmlResource::ManagedNodes,
            "edges" => HtmlResource::ManagedEdges,
            "attributes" => HtmlResource::ManagedAttributes,
            "utf16_units" => HtmlResource::ManagedUtf16Units,
            "depth" => HtmlResource::TreeDepth,
            other => panic!("unmapped construction limit {other}"),
        })
        .collect();
    assert_eq!(
        HtmlCapScope::Construction.capped_resources(),
        capped.as_slice()
    );
    assert!(HtmlCapScope::Construction.capped_work().is_empty());
}

#[test]
fn the_autocomplete_limits_fields_are_the_autocomplete_scope_caps() {
    let limits = HtmlAutocompleteMaterializationLimits {
        allocated_bytes: 1,
        nested_output_records: 2,
        scratch_records: 3,
    };
    let mut capped: Vec<HtmlResource> = field_names(&limits)
        .iter()
        .map(|name| match name.as_str() {
            "allocated_bytes" => HtmlResource::AllocatedBytes,
            "nested_output_records" => HtmlResource::NestedOutputRecords,
            "scratch_records" => HtmlResource::ScratchRecords,
            other => panic!("unmapped autocomplete limit {other}"),
        })
        .collect();
    let mut listed = HtmlCapScope::Autocomplete.capped_resources().to_vec();
    capped.sort();
    listed.sort();
    assert_eq!(listed, capped);
    assert!(HtmlCapScope::Autocomplete.capped_work().is_empty());
}

/// Retained storage is refused in the order of the parser's resource precedence,
/// which is the declaration order of the resources.
#[test]
fn retained_storage_conversions_follow_the_resource_precedence() {
    let position = |resource: HtmlResource| {
        HtmlResource::ALL
            .iter()
            .position(|candidate| *candidate == resource)
    };
    let ordered = |amounts: [(HtmlResource, u64); 4]| {
        amounts
            .windows(2)
            .all(|pair| position(pair[0].0) < position(pair[1].0))
    };
    assert!(ordered(retained_provider_storage(1, 2, 3)));
}

fn refusal_of(
    operation: &mut Op,
    scope: surgeist_ledger::Scope,
    batch: &[HtmlChange],
) -> Error<HtmlWork, HtmlResource> {
    operation
        .apply_batch(scope, batch)
        .expect_err("the batch is refused")
}

#[test]
fn a_ledger_limit_refusal_classifies_as_a_cap_with_its_figures() {
    let limits = HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            HtmlResource::ManagedNodes => 5,
            _ => u64::MAX,
        },
    );
    let mut operation = Operation::new("cap", limits, 1).unwrap();
    let root = operation.root();
    operation
        .apply(root, Change::Acquire(HtmlResource::ManagedNodes, 3))
        .unwrap();
    let error = refusal_of(
        &mut operation,
        root,
        &[Change::Acquire(HtmlResource::ManagedNodes, 4)],
    );
    let HtmlLedgerOutcome::Cap(refusal) = classify(error) else {
        panic!("a limit refusal is a cap refusal");
    };
    assert_eq!(refusal.scope(), root);
    assert_eq!(refusal.item(), 0);
    assert_eq!(
        refusal.change(),
        Change::Acquire(HtmlResource::ManagedNodes, 4)
    );
    assert_eq!(
        (refusal.current(), refusal.amount(), refusal.limit()),
        (3, 4, 5)
    );
    assert_eq!(refusal.required(), Ok(7));
    assert_eq!(
        refusal.figures(),
        Ok(HtmlRefusalFigures {
            limit: 5,
            required: 7
        })
    );
}

#[test]
fn the_current_value_of_a_cap_refusal_includes_the_batch_prefix() {
    let limits = HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            HtmlResource::ManagedEdges => 10,
            _ => u64::MAX,
        },
    );
    let mut operation = Operation::new("cap", limits, 1).unwrap();
    let root = operation.root();
    let error = refusal_of(
        &mut operation,
        root,
        &[
            Change::Acquire(HtmlResource::ManagedEdges, 6),
            Change::Acquire(HtmlResource::ManagedEdges, 6),
        ],
    );
    let HtmlLedgerOutcome::Cap(refusal) = classify(error) else {
        panic!("a limit refusal is a cap refusal");
    };
    assert_eq!(refusal.item(), 1);
    assert_eq!(
        refusal.figures(),
        Ok(HtmlRefusalFigures {
            limit: 10,
            required: 12
        })
    );
}

#[test]
fn an_arithmetic_overflow_classifies_as_exhaustion_of_the_changed_resource() {
    let mut operation = Operation::new("cap", HtmlLimits::unlimited(), 1).unwrap();
    let root = operation.root();
    operation
        .apply(root, Change::Acquire(HtmlResource::ManagedEdges, u64::MAX))
        .unwrap();
    let change = Change::Acquire(HtmlResource::ManagedEdges, 1);
    let error = refusal_of(&mut operation, root, &[change]);
    assert_eq!(classify(error), HtmlLedgerOutcome::Exhausted(change));
}

#[test]
fn a_release_beyond_the_held_custody_classifies_as_a_release_underflow() {
    let mut operation = Operation::new("cap", HtmlLimits::unlimited(), 1).unwrap();
    let root = operation.root();
    let error = refusal_of(
        &mut operation,
        root,
        &[Change::Release(HtmlResource::ManagedEdges, 1)],
    );
    assert_eq!(
        classify(error),
        HtmlLedgerOutcome::Fault(HtmlAccountingFault::ReleaseUnderflow)
    );
}

#[test]
fn every_other_ledger_error_classifies_as_one_fault() {
    use surgeist_ledger::Defect;
    for (error, fault) in [
        (Error::Allocation, HtmlAccountingFault::Allocation),
        (Error::ForeignScope, HtmlAccountingFault::ForeignScope),
        (
            Error::ScopeLimit { limit: 3 },
            HtmlAccountingFault::ScopeLimit { limit: 3 },
        ),
        (
            Error::IdentityExhausted,
            HtmlAccountingFault::IdentityExhausted,
        ),
        (
            Error::TimingUnavailable,
            HtmlAccountingFault::TimingUnavailable,
        ),
        (Error::ForeignCheckpoint, HtmlAccountingFault::Checkpoint),
        (Error::StaleCheckpoint, HtmlAccountingFault::Checkpoint),
        (
            Error::CheckpointLimit { limit: 1 },
            HtmlAccountingFault::Checkpoint,
        ),
        (
            Error::CheckpointsConfigured,
            HtmlAccountingFault::Checkpoint,
        ),
        (
            Error::Defect(Defect::ScopeEntry),
            HtmlAccountingFault::Defect(Defect::ScopeEntry),
        ),
    ] {
        assert_eq!(
            classify(error),
            HtmlLedgerOutcome::Fault(fault),
            "{error:?}"
        );
    }
}

#[test]
fn a_scope_of_another_operation_classifies_as_a_foreign_scope() {
    let mut operation = Operation::new("one", HtmlLimits::unlimited(), 1).unwrap();
    let other =
        Operation::<HtmlWork, HtmlResource>::new("two", HtmlLimits::unlimited(), 1).unwrap();
    let error = operation
        .apply(other.root(), Change::Acquire(HtmlResource::ManagedEdges, 1))
        .expect_err("a foreign scope is refused");
    assert_eq!(
        classify(error),
        HtmlLedgerOutcome::Fault(HtmlAccountingFault::ForeignScope)
    );
}

#[test]
fn an_operation_without_room_for_a_scope_classifies_as_a_scope_limit() {
    let mut operation = Operation::new("one", HtmlLimits::unlimited(), 1).unwrap();
    let root = operation.root();
    let error = operation
        .child(
            root,
            "child",
            HtmlLimits::unlimited(),
            HtmlTariffs::identity(),
        )
        .expect_err("the bound is spent");
    assert_eq!(
        classify(error),
        HtmlLedgerOutcome::Fault(HtmlAccountingFault::ScopeLimit { limit: 1 })
    );
}

#[test]
fn an_auxiliary_scope_name_is_unique_prefixed_and_not_a_cap_scope_name() {
    for (index, scope) in HtmlAuxiliaryScope::ALL.iter().enumerate() {
        let name = scope.name();
        assert!(name.starts_with("html-"), "{scope:?}");
        for other in &HtmlAuxiliaryScope::ALL[index + 1..] {
            assert_ne!(name, other.name(), "{scope:?} {other:?}");
        }
        assert!(
            HtmlCapScope::ALL.iter().all(|cap| cap.name() != name),
            "{scope:?} repeats a cap scope name"
        );
    }
}

/// The exact names a host profile reader matches on. Each auxiliary scope has one fixed name, and
/// a table of every variant keeps a new entry from going unpinned.
#[test]
fn each_auxiliary_scope_has_its_exact_name() {
    let table = [
        (HtmlAuxiliaryScope::ParseHost, "html-parse-host"),
        (HtmlAuxiliaryScope::ParseCharging, "html-parse-charging"),
        (HtmlAuxiliaryScope::ParseAdmission, "html-parse-admission"),
        (
            HtmlAuxiliaryScope::ParseConstruction,
            "html-parse-construction",
        ),
        (
            HtmlAuxiliaryScope::ParseFinalization,
            "html-parse-finalization",
        ),
        (
            HtmlAuxiliaryScope::ParsePublication,
            "html-parse-publication",
        ),
        (HtmlAuxiliaryScope::QueryObserved, "html-query-observed"),
        (
            HtmlAuxiliaryScope::AutocompleteObserved,
            "html-autocomplete-observed",
        ),
        (HtmlAuxiliaryScope::FactGrown, "html-fact-grown"),
        (
            HtmlAuxiliaryScope::OptionValueRetained,
            "html-option-value-retained",
        ),
        (
            HtmlAuxiliaryScope::OptionValueTemporary,
            "html-option-value-temporary",
        ),
        (
            HtmlAuxiliaryScope::OptionValueObserved,
            "html-option-value-observed",
        ),
    ];
    assert_eq!(table.len(), HtmlAuxiliaryScope::ALL.len());
    for (scope, name) in table {
        assert_eq!(scope.name(), name, "{scope:?}");
        assert!(HtmlAuxiliaryScope::ALL.contains(&scope), "{scope:?}");
    }
}
