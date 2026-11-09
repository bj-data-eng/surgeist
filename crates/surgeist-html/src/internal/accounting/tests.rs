use super::*;
use crate::bounded::values::HtmlStringPool;
use surgeist_ledger::{Dimension, Error, Operation, RefusalReason};

type Op = Operation<HtmlWork, HtmlResource>;

fn work_of(changes: &[HtmlChange]) -> Vec<(HtmlWork, u64)> {
    changes
        .iter()
        .filter_map(|change| match *change {
            Change::Work(dimension, amount) => Some((dimension, amount)),
            _ => None,
        })
        .collect()
}

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
fn value_pool_actions_convert_to_the_sum_of_their_five_fields() {
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
        operation
            .apply_batch(charging, &value_pool_work(work))
            .unwrap();
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
fn only_categories_and_measured_storage_lack_a_direct_cap() {
    // The eight parser categories are capped through their conversion into
    // credits, and the retained storage dimensions are measured only.
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
    assert_eq!(
        uncapped,
        [
            HtmlResource::RetainedBytes,
            HtmlResource::RetainedUtf16Units
        ]
    );
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

#[test]
fn csp_work_sums_its_own_and_closing_credits() {
    use crate::security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage};
    let storage = HtmlCspStorage::try_new(64, 8, 2).unwrap();
    let closing = HtmlCspDisposalWork::try_new(5, 16).unwrap();
    let layout = HtmlCspLayout::processing(37, storage, closing, 32).unwrap();
    assert_eq!(
        work_of(&[csp_work(&layout).unwrap()]),
        [(HtmlWork::ProviderCall, 42)]
    );
    assert_eq!(
        csp_storage(storage),
        [
            (HtmlResource::ManagedCapacityBytes, 64),
            (HtmlResource::PeakTransientBytes, 64),
            (HtmlResource::ManagedUtf16Units, 8),
            (HtmlResource::AuxiliaryEntries, 2),
        ]
    );
}
