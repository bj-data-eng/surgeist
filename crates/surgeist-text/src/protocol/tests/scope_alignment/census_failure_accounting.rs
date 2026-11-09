//! Standalone failure receipts preserve incurred costs without a union receipt.
use super::*;

fn observe(
    input: ScopeAlignmentProduct<'_>,
    related: &[ScopeAlignmentProduct<'_>],
    allowance: ScopeAlignmentAllowance,
) -> std::result::Result<ScopeAlignmentCensus, ScopeAlignmentCensusFailure> {
    scope_alignment_census(
        ScopeAlignmentCensusInput::Product(input),
        ScopeAlignmentProducts::Mixed(related),
        allowance,
    )
}

#[test]
fn complete_union_separates_new_scratch_rows_from_retained_roots() {
    let (system, prepared, selected, _, caps) = fixture();
    let stats = system.stats();
    let first = finalized(&selected, 3.0);
    let second = finalized(&selected, 5.0);
    let related = [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
        ScopeAlignmentProduct::Finalized(&second),
        ScopeAlignmentProduct::Finalized(&second),
    ];
    let census = observe(
        ScopeAlignmentProduct::Finalized(&first),
        &related,
        full(caps),
    )
    .unwrap();
    let accounting = census.accounting();
    assert_eq!(accounting.usage(), census.usage());
    let borrowed = census.input_retention().records() + census.external_retention().records();
    assert_eq!(
        accounting.newly_materialized_records(),
        census.usage().records() - borrowed
    );
    assert!(accounting.newly_materialized_records() > 0);
    assert_eq!(
        census.usage().payload_bytes(),
        census.input_retention().payload_bytes() + census.external_retention().payload_bytes()
    );
    assert!(census.usage().peak_payload_bytes() > census.usage().payload_bytes());
    assert_eq!(census.usage().font_queries(), 0);
    // Keep real source and metric outputs usable; the census is not a metric
    // operation or a replacement for their actual finalized authority.
    geometry(&prepared, &first, 3.0);
    assert_eq!(system.stats(), stats);
}

#[test]
fn pre_budget_refusal_has_no_accounting_and_preserves_original_cause() {
    let (_, prepared, _, _, caps) = fixture();
    let failure = observe(
        ScopeAlignmentProduct::Prepared(&prepared),
        &[],
        ScopeAlignmentAllowance::new(caps.records() + 1, caps.payload_bytes()),
    )
    .unwrap_err();
    assert_eq!(failure.accounting(), None);
    let original = (
        failure.error().code,
        failure.error().message.clone(),
        reason(failure.error()),
    );
    let (error, accounting) = failure.into_parts();
    assert_eq!(accounting, None);
    assert_eq!(
        (error.code, error.message.clone(), reason(&error)),
        original
    );
    assert!(error.source.is_none());
    assert_eq!(
        reason(&error),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::Records,
            required: caps.records() + 1,
            limit: caps.records(),
        }
    );
}

#[test]
fn funded_prepass_failure_reports_work_without_claiming_retained_union() {
    let (_, prepared, _, _, caps) = fixture();
    let aliases = vec![ScopeAlignmentProduct::Prepared(&prepared); 1024];
    let failure = observe(
        ScopeAlignmentProduct::Prepared(&prepared),
        &aliases,
        full(caps),
    )
    .unwrap_err();
    assert!(matches!(
        reason(failure.error()),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            ..
        }
    ));
    let accounting = failure.accounting().unwrap();
    assert!(accounting.usage().structural_work() > 0);
    assert_eq!(accounting.usage().records(), 0);
    assert_eq!(accounting.newly_materialized_records(), 0);
    assert_eq!(accounting.usage().payload_bytes(), 0);
    assert_eq!(accounting.usage().peak_payload_bytes(), 0);
    assert_eq!(accounting.usage().font_queries(), 0);
}

#[test]
fn funded_first_work_refusal_reports_zero_unperformed_work() {
    let (_, prepared, _, _, caps) = fixture();
    assert_eq!(caps.structural_work(), 1_000_000);
    // The complete slot inspection is funded before visiting any slot. These
    // are actual initialized strong-guard slots, with no retained products.
    let slots = vec![None; caps.structural_work()];
    let failure = scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(&prepared)),
        ScopeAlignmentProducts::FinalizedSlots(&slots),
        full(caps),
    )
    .unwrap_err();
    let accounting = failure.accounting().unwrap();
    assert_eq!(accounting.usage().structural_work(), 0);
    assert_eq!(accounting.usage().records(), 0);
    assert_eq!(accounting.usage().payload_bytes(), 0);
    assert_eq!(accounting.usage().peak_payload_bytes(), 0);
    assert_eq!(accounting.newly_materialized_records(), 0);
    assert_eq!(
        reason(failure.error()),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            required: caps.structural_work() + 1,
            limit: caps.structural_work(),
        }
    );
}

#[test]
fn later_scratch_refusal_keeps_peak_and_rows_after_releasing_live_scratch() {
    let (_, prepared, _, _, caps) = fixture();
    let root = ScopeAlignmentProduct::Prepared(&prepared);
    let complete = observe(root, &[], full(caps)).unwrap();
    assert!(complete.accounting().newly_materialized_records() > 1);
    let limit = complete.usage().records() - 1;
    let failure = observe(
        root,
        &[],
        ScopeAlignmentAllowance::new(limit, caps.payload_bytes()),
    )
    .unwrap_err();
    let accounting = failure.accounting().unwrap();
    assert_eq!(accounting.usage().records(), limit);
    assert_eq!(
        accounting.newly_materialized_records(),
        complete.accounting().newly_materialized_records() - 1
    );
    assert!(accounting.usage().structural_work() > 0);
    assert_eq!(
        accounting.usage().payload_bytes(),
        complete.input_retention().payload_bytes()
    );
    assert!(accounting.usage().peak_payload_bytes() > accounting.usage().payload_bytes());
    assert_eq!(accounting.usage().font_queries(), 0);
}

#[test]
fn seeding_each_dimension_retains_only_its_successful_credit() {
    let (_, prepared, _, _, caps) = fixture();
    let root = ScopeAlignmentProduct::Prepared(&prepared);
    let complete = observe(root, &[], full(caps)).unwrap();
    let retained = complete.input_retention();
    assert!(retained.records() > 0 && retained.payload_bytes() > 0);
    for records_seeded in [false, true] {
        let allowance = if records_seeded {
            ScopeAlignmentAllowance::new(caps.records(), 0)
        } else {
            ScopeAlignmentAllowance::new(0, caps.payload_bytes())
        };
        let failure = observe(root, &[], allowance).unwrap_err();
        let accounting = failure.accounting().unwrap();
        assert!(accounting.usage().structural_work() > 0);
        assert_eq!(
            accounting.usage().records(),
            if records_seeded {
                retained.records()
            } else {
                0
            }
        );
        assert_eq!(accounting.newly_materialized_records(), 0);
        assert_eq!(accounting.usage().payload_bytes(), 0);
        assert_eq!(accounting.usage().peak_payload_bytes(), 0);
        assert!(
            matches!(reason(failure.error()), ScopeAlignmentError::Limit {kind, ..}
            if kind == if records_seeded {ScopeAlignmentLimitKind::PayloadBytes} else {ScopeAlignmentLimitKind::Records})
        );
    }
}

#[test]
fn scratch_failure_retains_new_rows_and_work_but_discards_only_scratch_bytes() {
    let (_, prepared, _, _, caps) = fixture();
    let root = ScopeAlignmentProduct::Prepared(&prepared);
    let complete = observe(root, &[], full(caps)).unwrap();
    let retained = complete.input_retention();
    for byte_failure in [false, true] {
        let allowance = if byte_failure {
            ScopeAlignmentAllowance::new(caps.records(), retained.payload_bytes())
        } else {
            ScopeAlignmentAllowance::new(retained.records(), caps.payload_bytes())
        };
        let failure = observe(root, &[], allowance).unwrap_err();
        let accounting = failure.accounting().unwrap();
        assert!(accounting.usage().structural_work() > 0);
        assert_eq!(
            accounting.newly_materialized_records(),
            usize::from(byte_failure)
        );
        assert_eq!(
            accounting.usage().records(),
            retained.records() + usize::from(byte_failure)
        );
        assert_eq!(accounting.usage().payload_bytes(), retained.payload_bytes());
        assert_eq!(
            accounting.usage().peak_payload_bytes(),
            retained.payload_bytes()
        );
        assert_eq!(accounting.usage().font_queries(), 0);
        let original = (
            failure.error().code,
            failure.error().message.clone(),
            reason(failure.error()),
        );
        let (cause, observed) = failure.into_parts();
        assert_eq!(
            (cause.code, cause.message.clone(), reason(&cause)),
            original
        );
        assert!(cause.source.is_none());
        assert_eq!(observed, Some(accounting));
    }
}
