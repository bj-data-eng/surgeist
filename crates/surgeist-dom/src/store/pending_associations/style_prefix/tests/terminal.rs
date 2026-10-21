//! Real blocked invocations qualify zero obligations; no empty-policy denial.
use super::*;
use crate::{DomParsedHtmlContributionCompletion, DomParsedHtmlTerminalError};
use std::cell::Cell;
fn rejected(
    dom: &Dom,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    let view = dom.parsed_html_style_prefix(id).unwrap();
    let request = view.inline_policy_request().unwrap();
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let native = request
        .policy()
        .policy()
        .borrow_native_with::<NativePolicy, _>(
            request.policy().policy().profile().unwrap(),
            &mut allow,
        )
        .unwrap();
    let HtmlCspNativePolicy::External(native) = native else {
        panic!("actual external artifact")
    };
    // Clause 7 in this external fixture rejects every tested inline string.
    assert_eq!(native.clauses, [7; 8]);
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(request, !native.clauses.contains(&7))
}
fn blocked_ended(
    outputs: usize,
) -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    NodeId,
    Arc<AtomicUsize>,
    DomParsedHtmlSynchronousEnd,
) {
    let limits =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, outputs).unwrap();
    let (mut dom, env) = setup();
    let (doc, id) = install(
        &mut dom,
        parse("<!doctype html><style>rejected</style>"),
        env,
    );
    let (policy, drops, _) = native_policy(0);
    inline_with_policy_under(&mut dom, id, policy, limits);
    let q = qualifier(&dom, id);
    let node = dom
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap()
        .node_for(q.subject())
        .unwrap();
    let mut observation = Some(rejected(&dom, id));
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        let (next, retired) = hold
            .prepare_successor(None, &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
        drop(next);
    }
    let mut observation = Some(rejected(&dom, id));
    let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    hold.accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let end = hold.complete_synchronous_execution(&flag()).unwrap();
    drop(hold);
    (dom, doc, id, node, drops, end)
}
fn cause(error: DomParsedHtmlTerminalError) -> DomError {
    match error {
        DomParsedHtmlTerminalError::Frontier(DomParsedHtmlFrontierError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(error)),
        )) => error,
        other => panic!("actual original admission: {other:?}"),
    }
}
fn complete(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    limits: DomParsedHtmlAssociationLimits,
) -> Result<DomParsedHtmlContributionCompletion, DomParsedHtmlTerminalError> {
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .complete_contribution(&flag())
}
#[test]
fn whole_ack_publishes_readiness_and_retains_actual_bank_and_old_state_custody() {
    let (mut dom, doc, id, node, drops, end) = blocked_ended(1024);
    let old = dom.snapshot();
    let old_state = Arc::downgrade(&dom.state);
    let ledger = dom.custody_registry.ledger().clone();
    let report = {
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        Arc::downgrade(&hold.meter.entry().report.value().value)
    };
    let serial = old
        .node(node)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .next_association;
    let done = complete(&mut dom, id, envelope()).unwrap();
    assert_eq!(done.contribution(), id);
    assert_eq!(done.dom().changes().from(), old.stamp());
    assert_eq!(done.dom().changes().to(), dom.snapshot().stamp());
    assert!(
        done.dom()
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change,DomChange::ElementState(actual) if *actual==node))
    );
    assert!(dom.pending_parsed_html_association(doc).unwrap().is_none());
    let current = dom.snapshot();
    let current_style = current.node(node).unwrap().as_style().unwrap();
    assert_eq!(
        current_style.state().availability,
        DomStyleSheetAvailability::Established
    );
    assert_eq!(current_style.state().next_association, serial);
    assert!(current_style.sheet().unwrap().is_none());
    assert!(matches!(
        old.node(node).unwrap().as_style().unwrap().sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    drop(end);
    drop(current);
    drop(old);
    drop(dom);
    assert!(report.upgrade().is_some());
    assert!(old_state.upgrade().is_some());
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    assert_eq!(done.contribution(), id);
    drop(done);
    assert!(report.upgrade().is_none());
    assert!(old_state.upgrade().is_none());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}
#[test]
fn real_repeated_end_outputs_refuse_late_ack_and_exact_raised_output_retry_publishes_once() {
    let (mut dom, doc, id, node, _drops, end) = blocked_ended(16);
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 16).unwrap();
    let mut outputs = vec![end];
    loop {
        let result = dom
            .begin_parsed_html_style(id, limits, &flag())
            .unwrap()
            .complete_synchronous_execution(&flag());
        match result {
            Ok(end) => outputs.push(end),
            Err(DomParsedHtmlFrontierError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    ..
                }),
            ))) => break,
            other => panic!("actual End output admission: {other:?}"),
        }
    }
    let state = dom.state.clone();
    let before = super::super::terminal::PREPARED_ACK.with(Cell::get);
    assert!(matches!(
        cause(complete(&mut dom, id, limits).unwrap_err()),
        DomError::ResourceLimit {
            resource: DomResource::ParsedAssociationProcessingOutputs,
            ..
        }
    ));
    assert_eq!(
        super::super::terminal::PREPARED_ACK.with(Cell::get),
        before + 1,
        "actual bank and readiness candidate reached"
    );
    assert!(Arc::ptr_eq(&state, &dom.state));
    assert_eq!(dom.pending_parsed_html_association(doc).unwrap(), Some(id));
    assert_eq!(
        dom.snapshot()
            .node(node)
            .unwrap()
            .as_style()
            .unwrap()
            .state()
            .availability,
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    let raised = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 18).unwrap();
    let done = complete(&mut dom, id, raised).unwrap();
    assert_eq!(
        done.dom()
            .changes()
            .changes()
            .iter()
            .filter(|c| matches!(c, DomChange::ElementState(_)))
            .count(),
        1
    );
    assert!(
        matches!(dom.begin_parsed_html_style(id,raised,&flag()),Err(DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::NotPending{contribution})) if contribution==id)
    );
    assert_eq!(outputs[0].contribution(), id);
}
#[test]
fn cancelled_and_cumulative_work_refusal_preserve_same_end_pending_and_readiness() {
    let (mut dom, doc, id, node, _drops, _end) = blocked_ended(1024);
    let before = dom.state.clone();
    let original = dom.pending_associations.controlled_entry(id).unwrap();
    let spent = original.value().control.spent_work();
    {
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(std::ptr::eq(original.value().as_ref(), hold.meter.entry()));
        assert_eq!(
            cause(
                hold.complete_contribution(&AtomicBool::new(true))
                    .unwrap_err()
            ),
            DomError::Cancelled
        );
    }
    assert!(original.value().control.spent_work() >= spent);
    {
        // The refused consuming command released only its loan; this is the
        // same actual controller and cumulative work, not a replacement issuer.
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(std::ptr::eq(original.value().as_ref(), hold.meter.entry()));
        while hold.limits().work() - hold.spent_work() > 3 {
            hold.step(&flag()).unwrap();
        }
        assert!(matches!(
            cause(hold.complete_contribution(&flag()).unwrap_err()),
            DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                ..
            }
        ));
    }
    assert!(Arc::ptr_eq(&before, &dom.state));
    assert_eq!(dom.pending_parsed_html_association(doc).unwrap(), Some(id));
    assert_eq!(
        dom.snapshot()
            .node(node)
            .unwrap()
            .as_style()
            .unwrap()
            .state()
            .availability,
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    let exhausted = original.value().control.spent_work();
    let raised =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_100_000, 1024).unwrap();
    let hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(std::ptr::eq(original.value().as_ref(), hold.meter.entry()));
    assert!(hold.spent_work() >= exhausted);
    let done = hold.complete_contribution(&flag()).unwrap();
    assert_eq!(done.contribution(), id);
}

#[test]
fn fixed_obligation_arithmetic_refuses_overflow_and_underflow_without_wrapping() {
    // This is the private fixed-counter arithmetic boundary, not a fabricated
    // parser invocation or a claim that a practical report issues u64::MAX rights.
    let counter = crate::store::pending_associations::control::Control::default();
    assert!(matches!(
        counter.discharge_notification(),
        Err(DomError::StyleProtocol)
    ));
    assert_eq!(counter.notification_count(), 0);
    counter.publish_mint(u64::MAX, crate::DomParsedHtmlStyleResources::NoImports);
    assert!(matches!(
        counter.mint_notification(),
        Err(DomError::RepresentationExhausted)
    ));
    assert_eq!(counter.notification_count(), u64::MAX);
    let next = counter.discharge_notification().unwrap();
    assert_eq!(next, u64::MAX - 1);
    counter.publish_discharge(next);
    let next = counter.mint_notification().unwrap();
    counter.publish_mint(next, crate::DomParsedHtmlStyleResources::ImportsUnresolved);
    assert!(counter.has_imports());
    let next = counter.discharge_notification().unwrap();
    counter.publish_discharge(next);
    assert!(
        counter.has_imports(),
        "one settled right cannot erase actual imports"
    );
}
