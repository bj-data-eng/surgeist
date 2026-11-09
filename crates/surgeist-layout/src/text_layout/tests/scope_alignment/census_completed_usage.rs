//! Completed census costs enter the existing owner once, including exhaustion.
use super::*;
use crate::text_layout::scope_alignment::{
    ScopeAlignmentFacts, ScopeAlignmentSnapshot, allocation_buffer,
};

const RECORDS: usize = 1_000_000;
const BYTES: usize = 16 << 20;
const PAGE: usize = 1024;

fn source() -> (
    text::PreparedText,
    Arc<ScopeAlignmentSnapshot<f64>>,
    text::ScopeAlignmentLimits,
) {
    let limits = text::ScopeAlignmentLimits::new(RECORDS, 100, WORK, BYTES);
    let case = Case::new_with_limits(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
        limits,
    );
    let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![case.row(1, shifted::<f64>(3.0))],
    )
    .unwrap();
    (
        case.binding.prepared.clone(),
        Arc::clone(&flow.scope_alignments),
        limits,
    )
}
fn input(prepared: &text::PreparedText) -> text::ScopeAlignmentCensusInput<'_> {
    text::ScopeAlignmentCensusInput::Product(text::ScopeAlignmentProduct::Prepared(prepared))
}
fn census(
    prepared: &text::PreparedText,
    limits: text::ScopeAlignmentLimits,
) -> text::ScopeAlignmentCensus {
    text::scope_alignment_census(
        input(prepared),
        text::ScopeAlignmentProducts::Mixed(&[]),
        text::ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes()),
    )
    .unwrap()
}
fn owner(snapshot: &Arc<ScopeAlignmentSnapshot<f64>>) -> ScopeAlignmentResources<f64> {
    ScopeAlignmentResources::new(
        Arc::clone(snapshot),
        crate::text_layout::fragmentation::InlineBoxFamilies::default(),
    )
}
fn remaining_records(
    facts: &ScopeAlignmentResources<f64>,
    limits: text::ScopeAlignmentLimits,
) -> usize {
    // Actual zero-byte logical descriptor rows exercise admission without
    // inventing a live-payload declaration or reading a private counter.
    let mut rows = Vec::<()>::new();
    for _ in 0..=limits.records() {
        match facts.materialize(limits, 0, 1) {
            Ok(()) => rows.push(()),
            Err(error) => {
                assert!(
                    matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
                    reason: text::ScopeAlignmentError::Limit {kind: text::ScopeAlignmentLimitKind::Records, required, limit},
                }) if *required == RECORDS + 1 && *limit == RECORDS)
                );
                return rows.len();
            }
        }
    }
    panic!("logical record admission must obey its captured ceiling")
}

#[test]
fn successful_census_charges_work_once_and_only_new_records_cumulatively() {
    let (prepared, snapshot, limits) = source();
    let observed = census(&prepared, limits);
    let spent = owner(&snapshot);
    let fresh = owner(&snapshot);
    drop(spent.begin(input(&prepared), 0).unwrap().unwrap());
    let work = observed.accounting().usage().structural_work();
    assert!(work > 0 && work < WORK);
    spent.visit(limits, WORK - work).unwrap();
    exact_refusal(spent.visit(limits, 1).unwrap_err(), WORK + 1, WORK);
    let spent_rows = remaining_records(&spent, limits);
    let fresh_rows = remaining_records(&fresh, limits);
    assert_eq!(
        fresh_rows - spent_rows,
        observed.accounting().newly_materialized_records()
    );
}

#[test]
fn completed_success_exhaustion_is_retained_before_later_admissions() {
    let (prepared, snapshot, limits) = source();
    let observed = census(&prepared, limits);
    let facts = owner(&snapshot);
    facts.visit(limits, WORK).unwrap();
    let required = WORK + observed.usage().structural_work();
    for _ in 0..2 {
        let error = match facts.begin(input(&prepared), 0) {
            Err(error) => error,
            Ok(_) => panic!("completed census work exceeds the cumulative ceiling"),
        };
        exact_refusal(error, required, WORK);
        exact_refusal(facts.visit(limits, 0).unwrap_err(), required, WORK);
    }
}

#[test]
fn completed_new_record_exhaustion_survives_discarded_logical_rows() {
    let (prepared, snapshot, limits) = source();
    let observed = census(&prepared, limits);
    let facts = owner(&snapshot);
    let admitted = remaining_records(&facts, limits);
    assert!(admitted > 0);
    // Dropping the real zero-byte rows does not refund their materialization.
    let error = match facts.begin(input(&prepared), 0) {
        Err(error) => error,
        Ok(_) => panic!("completed scratch cannot refund earlier materialization"),
    };
    let expected = text::ScopeAlignmentError::Limit {
        kind: text::ScopeAlignmentLimitKind::Records,
        required: RECORDS + observed.usage().records(),
        limit: RECORDS,
    };
    assert!(
        matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment { reason }) if *reason == expected)
    );
    for _ in 0..2 {
        let error = facts.materialize(limits, 0, 0).unwrap_err();
        assert!(
            matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment { reason }) if *reason == expected)
        );
    }
}

#[test]
fn native_prepared_and_semantic_products_do_not_activate_a_scope_budget() {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    let style = crate::test_support::text::ahem_style(10.0, 10.0);
    let paragraph = crate::test_support::text::PreparedParagraph::new(
        vec![crate::test_support::text::ParagraphPart::text(
            10,
            text::Source::new("X"),
            style.clone(),
        )],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    assert!(paragraph.prepared.scope_alignment_retention().is_none());
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [10])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..Default::default()
            },
        )
        .input(10, paragraph.text_input(10));
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let facts = owner(&flow.scope_alignments);
    assert!(
        facts
            .begin(input(&paragraph.prepared), 1)
            .unwrap()
            .is_none()
    );
    let segment = paragraph.prepared.source_segments().next().unwrap();
    let proof = paragraph
        .prepared
        .semantic_window(&text::SemanticWindowRequest::new(segment))
        .unwrap();
    assert!(proof.scope_alignment_usage().is_none());
    assert!(
        facts
            .begin(
                text::ScopeAlignmentCensusInput::Product(
                    text::ScopeAlignmentProduct::SemanticWindow(&proof)
                ),
                1
            )
            .unwrap()
            .is_none()
    );
}

#[test]
fn failed_census_cause_wins_while_late_exhaustion_and_new_records_remain() {
    let (prepared, snapshot, limits) = source();
    let observed = census(&prepared, limits);
    let facts = owner(&snapshot);
    let operation = facts.begin(input(&prepared), 0).unwrap().unwrap();
    let retained = observed.input_retention().payload_bytes();
    let bytes = operation.allowance().unwrap().payload_bytes() - retained;
    let pages = bytes / PAGE;
    let (mut page_values, page_lease) =
        allocation_buffer::<[u8; PAGE]>(&facts, &operation, pages).unwrap();
    let tail = operation.allowance().unwrap().payload_bytes() - retained;
    let (mut tail_values, tail_lease) = allocation_buffer::<u8>(&facts, &operation, tail).unwrap();
    let initialized = pages + tail;
    facts.visit(limits, initialized).unwrap();
    page_values.resize(pages, [0; PAGE]);
    tail_values.resize(tail, 0);
    assert_eq!(operation.allowance().unwrap().payload_bytes(), retained);
    drop(operation);
    let refusal = text::scope_alignment_census(
        input(&prepared),
        text::ScopeAlignmentProducts::Mixed(&[]),
        text::ScopeAlignmentAllowance::new(limits.records(), retained),
    )
    .unwrap_err();
    let partial = refusal.accounting().unwrap();
    assert!(partial.usage().structural_work() > 0);
    assert!(partial.newly_materialized_records() > 0);
    facts
        .visit(
            limits,
            WORK - observed.usage().structural_work() - initialized,
        )
        .unwrap();
    let error = match facts.begin(input(&prepared), 0) {
        Err(error) => error,
        Ok(_) => panic!("scratch cannot fit the retained-input-only byte allowance"),
    };
    assert_eq!(error.code, refusal.error().code);
    assert_eq!(error.message, refusal.error().message);
    assert!(
        matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
        reason: text::ScopeAlignmentError::Limit {kind: text::ScopeAlignmentLimitKind::PayloadBytes, limit, ..},
    }) if *limit == retained)
    );
    let required = WORK + partial.usage().structural_work();
    exact_refusal(facts.visit(limits, 0).unwrap_err(), required, WORK);
    // Releasing actual live buffers cannot release completed structural work.
    drop((page_values, tail_values, page_lease, tail_lease));
    exact_refusal(facts.visit(limits, 1).unwrap_err(), required, WORK);
}
