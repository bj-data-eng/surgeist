//! The stops a ledger refusal maps to. The expected figures come from the
//! meaning of the parser's stop fields: the cap, the value before the refused
//! charge and the change the charge asked for.

use super::*;
use crate::HtmlParseLimitsBuilder;

fn builder() -> HtmlParseLimitsBuilder {
    HtmlParseLimits::builder()
        .source_units(1_000)
        .buffered_utf16_units(1_000)
        .token_attributes(1_000)
        .auxiliary_entries(1_000)
        .active_formatting_entries(1_000)
        .managed_nodes(1_000)
        .managed_edges(1_000)
        .managed_attributes(1_000)
        .managed_utf16_units(1_000)
        .source_map_entries(1_000)
        .source_map_payload_units(1_000)
        .diagnostics(1_000)
        .diagnostic_payload_units(1_000)
        .context_effects(1_000)
        .open_elements(1_000)
        .tree_depth(1_000)
        .managed_capacity_bytes(16_000)
        .peak_transient_bytes(32_000)
        .atomic_layout_bytes(16_000)
        .work_credits(100_000)
}

fn control_with(limits: HtmlParseLimitsBuilder) -> ParseControl {
    ParseControl::new(limits.finish().unwrap(), HtmlParseCancellation::new())
}

fn limit(
    resource: HtmlParseResource,
    limit: u64,
    current: u64,
    requested_delta: u64,
) -> HtmlParseStop {
    HtmlParseStop::LimitExceeded {
        resource,
        limit,
        current,
        requested_delta,
    }
}

#[test]
fn a_work_batch_over_the_credit_cap_reports_the_credits_before_it_and_the_sum_of_its_items() {
    let mut c = control_with(builder().work_credits(20));
    c.charge(HtmlParseWorkCategory::StateStep, 6).unwrap();
    // The first item fits (6 + 5 <= 20), and the batch as a whole does not
    // (6 + 5 + 7 + 4 > 20): the stop names the whole batch.
    let refused = c.charge_changes(&[
        HtmlParseWorkCategory::CopyByte.change(5),
        HtmlParseWorkCategory::InspectByte.change(7),
        HtmlParseWorkCategory::RecordWrite.change(4),
    ]);
    assert_eq!(
        refused,
        Err(limit(HtmlParseResource::WorkCredits, 20, 6, 16))
    );
    assert_eq!(c.usage().work(HtmlParseWorkCategory::CopyByte), 0);
}

#[test]
fn an_item_that_overflows_is_reported_before_a_cap_an_earlier_item_reaches() {
    let mut c = control_with(builder().work_credits(10));
    c.charge(HtmlParseWorkCategory::StateStep, 1).unwrap();
    // The first item alone is over the cap; the second overflows the credit
    // total. The parser reports every overflow of a batch before its cap.
    assert_eq!(
        c.charge_changes(&[
            HtmlParseWorkCategory::CopyByte.change(100),
            HtmlParseWorkCategory::InspectByte.change(u64::MAX),
        ]),
        Err(HtmlParseStop::RepresentationExhausted {
            resource: HtmlParseResource::WorkCredits
        })
    );
}

#[test]
fn a_gauge_over_its_cap_reports_the_previous_value_and_the_change_the_gauge_describes() {
    let mut c = control_with(builder().tree_depth(5).atomic_layout_bytes(200));
    c.observe_tree_depth(3).unwrap();
    // The depth gauge asks for the rise over the previous value.
    assert_eq!(
        c.observe_tree_depth(7),
        Err(limit(HtmlParseResource::TreeDepth, 5, 3, 4))
    );
    let mut c = control_with(builder().tree_depth(5).atomic_layout_bytes(200));
    c.admit_atomic_layout(100).unwrap();
    // The layout gauge asks for the layout itself.
    assert_eq!(
        c.admit_atomic_layout(300),
        Err(limit(HtmlParseResource::AtomicLayoutBytes, 200, 100, 300))
    );
    // The refused value was not recorded.
    assert_eq!(
        c.first_stop_usage()
            .unwrap()
            .resource(HtmlParseResource::AtomicLayoutBytes),
        HtmlParseResourceUsage {
            current: 100,
            high_water: 100
        }
    );
}

#[test]
fn coupled_storage_reports_each_resource_with_its_own_sum_and_value() {
    let mut c = control_with(builder().managed_nodes(10).open_elements(10));
    c.acquire(HtmlParseResource::ManagedNodes, 4).unwrap();
    // Two requests for one resource are one change of their sum.
    assert_eq!(
        c.acquire_many(&[
            (HtmlParseResource::ManagedNodes, 4),
            (HtmlParseResource::ManagedNodes, 3),
        ]),
        Err(limit(HtmlParseResource::ManagedNodes, 10, 4, 7))
    );
}

#[test]
fn an_overflow_of_a_later_resource_is_reported_before_a_cap_an_earlier_one_reaches() {
    let mut c = control_with(builder().managed_nodes(10));
    c.acquire(HtmlParseResource::OpenElements, 1).unwrap();
    assert_eq!(
        c.acquire_many(&[
            (HtmlParseResource::ManagedNodes, 11),
            (HtmlParseResource::OpenElements, u64::MAX),
        ]),
        Err(HtmlParseStop::RepresentationExhausted {
            resource: HtmlParseResource::OpenElements
        })
    );
}

#[test]
fn a_reserve_over_the_capacity_cap_reports_the_net_growth_and_the_capacity_held() {
    let mut c = control_with(builder().managed_capacity_bytes(150));
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    // Replacing 40 bytes by 100 leaves 160 held, over the cap of 150, and grows
    // the capacity by 60.
    assert_eq!(
        c.preflight_capacity(40, 100, true),
        Err(limit(HtmlParseResource::ManagedCapacityBytes, 150, 100, 60))
    );
}

#[test]
fn a_reserve_whose_overlap_passes_the_transient_cap_records_the_overlap_as_a_peak_only() {
    let mut c = control_with(builder().peak_transient_bytes(200));
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    // Old and new layouts together are 100 + 100 = 200, the cap.
    c.preflight_capacity(40, 100, true).unwrap();
    let usage = c.usage();
    assert_eq!(
        usage.resource(HtmlParseResource::PeakTransientBytes),
        HtmlParseResourceUsage {
            current: 100,
            high_water: 200
        }
    );
    // One byte over the cap is refused with the layout asked for.
    let mut c = control_with(builder().peak_transient_bytes(199));
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    assert_eq!(
        c.preflight_capacity(40, 100, true),
        Err(limit(HtmlParseResource::PeakTransientBytes, 199, 100, 100))
    );
}

#[test]
fn a_release_beyond_the_held_custody_is_a_release_underflow_and_credits_nothing() {
    let mut c = control_with(builder());
    c.acquire(HtmlParseResource::ManagedEdges, 2).unwrap();
    assert_eq!(
        c.commit_prepared_release(&[
            (HtmlParseResource::ManagedEdges, 1),
            (HtmlParseResource::ManagedEdges, 2),
        ]),
        Err(HtmlParseStop::InternalInvariant(
            HtmlParseInvariant::ReleaseUnderflow
        ))
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        2
    );
}

#[test]
fn an_overshoot_the_cap_cannot_hold_stops_with_the_planned_layout_and_keeps_the_capacity() {
    let mut c = control_with(builder().managed_capacity_bytes(150));
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    // A 200 byte allocation replaced 40 bytes where 100 were planned: the
    // capacity it would hold, 260, is over the cap.
    assert_eq!(
        c.record_capacity(40, 200, 100),
        Err(limit(HtmlParseResource::ManagedCapacityBytes, 100, 40, 160))
    );
    assert_eq!(
        c.first_stop_usage()
            .unwrap()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        100
    );
}

#[test]
fn an_overshoot_the_cap_holds_is_recorded_for_the_caller_to_stop_on() {
    let mut c = control_with(builder().managed_capacity_bytes(200));
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    c.record_capacity(40, 120, 100).unwrap();
    let usage = c.usage();
    assert_eq!(
        usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        180
    );
    assert_eq!(usage.actual_capacity_bytes, 180);
    assert_eq!(usage.actual_capacity_high_water, 180);
    // The old and new layouts overlapped at 220 before the old one went.
    assert_eq!(
        usage
            .resource(HtmlParseResource::PeakTransientBytes)
            .high_water,
        220
    );
    assert_eq!(c.stop(), None);
}

#[test]
fn a_refused_record_after_a_reserve_leaves_the_ticket_releasable_for_what_was_admitted() {
    let mut c = control_with(builder().managed_capacity_bytes(200));
    let (mut vector, mut ticket) = c.new_vector::<u64>().unwrap();
    // The allocation is planned at 32 bytes, which fits, and observed far larger.
    c.overshoot_next_capacity_record(1_000);
    let stop = c.reserve_vector(&mut vector, &mut ticket, 4).unwrap_err();
    assert!(matches!(
        stop,
        HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }
    ));
    // The first stop shows the custody the ledger admitted, without the overshoot.
    assert_eq!(
        c.first_stop_usage()
            .unwrap()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0
    );
    c.release_vector(vector, ticket).unwrap();
    crate::parser::tree_builder::tests::assert_released(&c);
}

/// A control that charges `scope` of a host operation with the given caps.
fn hosted(
    host_limits: surgeist_html::accounting::HtmlLimits,
    parent_tariffs: Option<surgeist_html::accounting::HtmlTariffs>,
    parse_limits: HtmlParseLimitsBuilder,
) -> (ParseControl, crate::accounting::RequestAccounting) {
    let ledger = surgeist_ledger::Operation::new("host", host_limits, 8)
        .unwrap()
        .share();
    let mut scope = ledger.root();
    if let Some(tariffs) = parent_tariffs {
        scope = ledger
            .child(
                scope,
                "tenant",
                surgeist_html::accounting::HtmlLimits::unlimited(),
                tariffs,
            )
            .unwrap();
    }
    let request = crate::HtmlParseAccounting::host(ledger, scope).into_request();
    let control = ParseControl::with_accounting(
        parse_limits.finish().unwrap(),
        HtmlParseCancellation::new(),
        request.lease(),
    );
    (control, request)
}

fn host_cap_on(
    work: Option<(HtmlWork, u64)>,
    live: Option<(HtmlResource, u64)>,
) -> surgeist_html::accounting::HtmlLimits {
    surgeist_html::accounting::HtmlLimits::from_fn(
        |dimension| match work {
            Some((capped, cap)) if capped == dimension => cap,
            _ => u64::MAX,
        },
        |dimension| match live {
            Some((capped, cap)) if capped == dimension => cap,
            _ => u64::MAX,
        },
    )
}

#[test]
fn a_host_credit_cap_reached_in_a_batch_reports_the_hosts_total_before_it_and_the_batch_sum() {
    let (mut c, _request) = hosted(
        host_cap_on(Some((HtmlWork::Credits, 20)), None),
        None,
        builder(),
    );
    c.charge(HtmlParseWorkCategory::StateStep, 6).unwrap();
    // The host holds 6 credits, the first two items fit and the third crosses its
    // cap: the stop names the host's total before the batch and the whole batch.
    assert_eq!(
        c.charge_changes(&[
            HtmlParseWorkCategory::CopyByte.change(5),
            HtmlParseWorkCategory::InspectByte.change(7),
            HtmlParseWorkCategory::RecordWrite.change(4),
        ]),
        Err(limit(HtmlParseResource::WorkCredits, 20, 6, 16))
    );
}

#[test]
fn a_cap_above_the_host_scope_reports_its_figures_in_the_units_it_counts() {
    use surgeist_ledger::{Tariff, Tariffs};
    // The root counts the host scope's work twice over.
    let tariffs = Tariffs::from_fn(
        |work| Tariff::new(work, 2),
        |resource| Tariff::new(resource, 1),
    );
    let (mut c, _request) = hosted(
        host_cap_on(Some((HtmlWork::Credits, 40)), None),
        Some(tariffs),
        builder(),
    );
    c.charge(HtmlParseWorkCategory::StateStep, 6).unwrap();
    // The root holds 12 and the batch is 16 in the parse's units, 32 in its own.
    assert_eq!(
        c.charge_changes(&[
            HtmlParseWorkCategory::CopyByte.change(5),
            HtmlParseWorkCategory::InspectByte.change(7),
            HtmlParseWorkCategory::RecordWrite.change(4),
        ]),
        Err(limit(HtmlParseResource::WorkCredits, 40, 12, 32))
    );
}

#[test]
fn a_replacement_refused_by_the_host_reports_the_hosts_value_and_the_net_growth() {
    let (mut c, _request) = hosted(
        host_cap_on(None, Some((HtmlResource::ManagedCapacityBytes, 150))),
        None,
        builder(),
    );
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    c.set(HtmlParseResource::PeakTransientBytes, 100);
    // Replacing 40 bytes by 100 leaves 160 held in the host, over its cap of 150,
    // and grows what it holds by 60.
    assert_eq!(
        c.preflight_capacity(40, 100, true),
        Err(limit(HtmlParseResource::ManagedCapacityBytes, 150, 100, 60))
    );
}

#[test]
fn coalesced_storage_refused_by_the_host_reports_the_hosts_value_and_the_sum() {
    let (mut c, _request) = hosted(
        host_cap_on(None, Some((HtmlResource::ManagedNodes, 10))),
        None,
        builder(),
    );
    c.acquire(HtmlParseResource::ManagedNodes, 4).unwrap();
    assert_eq!(
        c.acquire_many(&[
            (HtmlParseResource::ManagedNodes, 4),
            (HtmlParseResource::ManagedNodes, 3),
        ]),
        Err(limit(HtmlParseResource::ManagedNodes, 10, 4, 7))
    );
}

#[test]
fn a_phase_the_ledger_cannot_observe_stops_the_parse_with_the_fault() {
    use surgeist_html::accounting::HtmlLimits;
    let ledger = surgeist_ledger::Operation::new("host", HtmlLimits::unlimited(), 8)
        .unwrap()
        .share();
    let root = ledger.root();
    let other = surgeist_ledger::Operation::<HtmlWork, HtmlResource>::new(
        "other",
        HtmlLimits::unlimited(),
        1,
    )
    .unwrap();
    let request = crate::HtmlParseAccounting::host(ledger, root)
        .observing_phases()
        .into_request();
    let mut c = ParseControl::with_accounting(
        builder().finish().unwrap(),
        HtmlParseCancellation::new(),
        request.lease(),
    );
    assert_eq!(c.stop(), None);
    // The phase scopes now name a scope of another operation.
    c.accounts
        .as_mut()
        .unwrap()
        .replace_phases_for_test(crate::accounting::PhaseScopes::all_at(other.root()));
    c.enter_phase(ParsePhase::Construction);
    let fault = HtmlParseStop::Accounting(HtmlParseAccountingFault::ForeignScope);
    assert_eq!(c.stop(), Some(fault));
    assert_eq!(c.charge(HtmlParseWorkCategory::StateStep, 1), Err(fault));
}
