//! Funded standalone census failure must remain charged to its Layout owner.
use super::*;
use crate::text_layout::scope_alignment::{
    ScopeAlignmentFacts, ScopeAlignmentResources, ScopeLease, allocation_buffer,
};

const RECORDS: usize = 1_000_000;
const WORK: usize = 1_000_000;
const BYTES: usize = 16 << 20;
const PAGE: usize = 1024;

struct Descriptors {
    _pages: Vec<[u8; PAGE]>,
    _page_lease: ScopeLease,
    _tail: Vec<u8>,
    _tail_lease: ScopeLease,
    initialized_rows: usize,
}

fn fund_descriptors<S: LayoutScalar>(
    facts: &ScopeAlignmentResources<S>,
    prepared: &text::PreparedText,
    retained: text::ScopeAlignmentRetention,
) -> Descriptors {
    let operation = facts
        .begin(
            text::ScopeAlignmentCensusInput::Product(text::ScopeAlignmentProduct::Prepared(
                prepared,
            )),
            0,
        )
        .unwrap()
        .unwrap();
    assert!(retained.records() > 0 && retained.payload_bytes() > 0);
    let available = operation.allowance().unwrap().payload_bytes();
    let bytes = available.checked_sub(retained.payload_bytes()).unwrap();
    assert!(bytes > PAGE);
    // These are actual funded backing allocations, not synthetic live-byte
    // declarations. Page rows keep their record cost below the record ceiling;
    // the tail makes the next byte allowance exactly the retained input size.
    let pages = bytes / PAGE;
    let (mut page_values, page_lease) =
        allocation_buffer::<[u8; PAGE]>(facts, &operation, pages).unwrap();
    let tail = operation
        .allowance()
        .unwrap()
        .payload_bytes()
        .checked_sub(retained.payload_bytes())
        .unwrap();
    let (mut tail_values, tail_lease) = allocation_buffer::<u8>(facts, &operation, tail).unwrap();
    let initialized_rows = pages + tail;
    facts.visit(operation.limits(), initialized_rows).unwrap();
    page_values.resize(pages, [0; PAGE]);
    tail_values.resize(tail, 0);
    assert_eq!(
        operation.allowance().unwrap().payload_bytes(),
        retained.payload_bytes()
    );
    drop(operation);
    Descriptors {
        _pages: page_values,
        _page_lease: page_lease,
        _tail: tail_values,
        _tail_lease: tail_lease,
        initialized_rows,
    }
}

/// Count successful explicit one-unit work admissions, not private invocations
/// or an inspection of shared counters. The captured ceiling bounds this walk.
fn remaining_work<S: LayoutScalar>(
    facts: &ScopeAlignmentResources<S>,
    limits: text::ScopeAlignmentLimits,
) -> usize {
    for remaining in 0..=limits.structural_work() {
        if let Err(error) = facts.visit(limits, 1) {
            assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
            assert!(
                matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::Limit {
                    kind: text::ScopeAlignmentLimitKind::StructuralWork,
                    required, limit,
                },
            }) if *required == WORK + 1 && *limit == WORK)
            );
            return remaining;
        }
    }
    panic!("an immutable work ceiling must refuse the next work unit")
}

fn compare<S: LayoutScalar>(census_failure: bool) {
    let limits = text::ScopeAlignmentLimits::new(RECORDS, 100, WORK, BYTES);
    let case = Case::new_with_limits(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
        limits,
    );
    let tree = case.tree::<S>(100.0, VerticalAlignOf::Baseline);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![case.row(1, shifted::<S>(3.0))],
    )
    .unwrap();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout_with_flow(&flow, 0, request(100.0, axes)).unwrap();
    let prior = readback::continuous_custody(&batch, Phase::Final);
    let prior_source = placed(&batch).source().clone();
    let input = text::ScopeAlignmentCensusInput::Product(text::ScopeAlignmentProduct::Prepared(
        &case.binding.prepared,
    ));
    // A successful public census of these exact roots proves the actual input
    // custody and positive traversal without querying a font or fitting a line.
    let census = text::scope_alignment_census(
        input,
        text::ScopeAlignmentProducts::FinalizedAndInlineBoxFamilies {
            finalized: &[],
            families: &[],
        },
        text::ScopeAlignmentAllowance::new(RECORDS, BYTES),
    )
    .unwrap();
    assert!(census.usage().structural_work() > 0);
    assert_eq!(census.usage().font_queries(), 0);
    assert!(census.usage().peak_payload_bytes() > census.input_retention().payload_bytes());

    // Only immutable admitted input is shared. Each constructor creates its
    // own cumulative resource owner; no resource-state clone is a control.
    let spent = ScopeAlignmentResources::new(
        Arc::clone(&flow.scope_alignments),
        crate::text_layout::fragmentation::InlineBoxFamilies::default(),
    );
    let fresh = ScopeAlignmentResources::new(
        Arc::clone(&flow.scope_alignments),
        crate::text_layout::fragmentation::InlineBoxFamilies::default(),
    );
    let spent_descriptors =
        fund_descriptors(&spent, &case.binding.prepared, census.input_retention());
    let fresh_descriptors =
        fund_descriptors(&fresh, &case.binding.prepared, census.input_retention());
    assert_eq!(
        spent_descriptors.initialized_rows,
        fresh_descriptors.initialized_rows
    );

    if census_failure {
        // The retained union fits exactly. Its allocation-free prepass is
        // executed before the first seen-set row needs additional scratch.
        let error = match spent.begin(input, 0) {
            Err(error) => error,
            Ok(_) => panic!("the first census scratch row cannot fit the input-only allowance"),
        };
        assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
        assert!(
            matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
            reason: text::ScopeAlignmentError::Limit {
                kind: text::ScopeAlignmentLimitKind::PayloadBytes, required, limit,
            },
        }) if *limit == census.input_retention().payload_bytes() && required > limit)
        );
    } else {
        // Nearest positive control: an explicit real debit changes subsequent
        // admission by exactly one, with identical successful setup on both.
        spent.visit(limits, 1).unwrap();
    }
    let spent_remaining = remaining_work(&spent, limits);
    let fresh_remaining = remaining_work(&fresh, limits);
    // Successful setup includes descriptor initialization and, when admitted
    // by the owner, its census charges. Never assume the full ceiling remains.
    assert!(fresh_remaining <= WORK - fresh_descriptors.initialized_rows);
    assert!(fresh_remaining < WORK);
    assert_eq!(
        readback::continuous_custody(&batch, Phase::Final).as_slice(),
        prior
    );
    assert_eq!(placed(&batch).source(), &prior_source);
    if census_failure {
        assert!(
            spent_remaining < fresh_remaining,
            "funded failed census must reduce later admission: spent {spent_remaining}, identically funded fresh {fresh_remaining}"
        );
    } else {
        assert_eq!(spent_remaining + 1, fresh_remaining);
    }
}

#[test]
fn independent_owner_explicit_work_control_f32() {
    compare::<f32>(false);
}
#[test]
fn independent_owner_explicit_work_control_f64() {
    compare::<f64>(false);
}
#[test]
fn funded_census_failure_reduces_subsequent_admission_f32() {
    compare::<f32>(true);
}
#[test]
fn funded_census_failure_reduces_subsequent_admission_f64() {
    compare::<f64>(true);
}
