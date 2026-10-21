use super::super::rendering::tests::ready;
use super::*;
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
#[test]
fn completed_predecessor_is_same_entry_and_canonical_serial_membership_are_preserved() {
    let (mut dom, doc, id, q, element, cap) = ready("", 1024);
    dom.begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap();
    let before = dom.snapshot();
    let old_state = before
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .clone();
    let old_role = before
        .node(doc.node())
        .unwrap()
        .node
        .base_document()
        .unwrap()
        .render_blocking;
    let ledger = dom.custody_registry.ledger().clone();
    let mut hold = dom
        .begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
        .unwrap();
    let prep = hold.prepare_successor(Some(&cap), &flag()).unwrap();
    let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
    let entry = hold.meter.entry();
    let completed_guard = entry.completed_style.try_lock().unwrap();
    let completed = completed_guard.as_ref().unwrap();
    assert_eq!(completed.qualification, q);
    assert!(
        matches!(&completed.outcome, CompletedOutcome::Created { sheet,notification:DomParsedHtmlStyleNotificationHandoffStatus::Retained } if sheet==old_state.sheet.as_ref().unwrap())
    );
    assert_ne!(completion.qualification(), &q);
    let retained = ledger.total().bytes();
    drop(retirement);
    assert!(
        ledger.total().bytes() < retained,
        "old text/frame admission ends after real retirement"
    );
    drop(completed_guard);
    drop(hold);
    let after = dom.snapshot();
    let current = after
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .clone();
    assert!(current.sheet.is_none());
    assert_eq!(current.next_association, old_state.next_association);
    assert_eq!(
        after
            .node(doc.node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking,
        old_role
    );
    assert_eq!(
        current.availability,
        crate::DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert_eq!(completion.qualification().document(), doc);
}
#[test]
fn old_inline_decision_cannot_answer_new_occurrence() {
    let (mut dom, _doc, id, q, _element, cap) = ready("", 1024);
    dom.begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap();
    let mut old = {
        let entry = dom
            .pending_associations
            .entries
            .get(usize::try_from(id.stamp().revision().get()).unwrap())
            .unwrap();
        let graph = entry.style_prefix.try_lock().unwrap();
        let Graph::Rendered { request, .. } = graph.as_ref().unwrap().value().as_ref() else {
            unreachable!()
        };
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    };
    {
        let mut hold = dom
            .begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
            .unwrap();
        let (_output, retirement) = hold
            .prepare_successor(Some(&cap), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retirement);
        assert!(matches!(
            hold.accept_inline_policy(&mut old, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert!(old.is_some());
    }
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
}
#[test]
fn real_post_producer_work_refusal_preserves_old_invocation_and_raised_retry_removes_once() {
    let mut reached = false;
    for remaining in 1..=16 {
        let (mut dom, _doc, id, q, _element, cap) = ready("", 1024);
        dom.begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
            .unwrap()
            .process_rendering(
                &q,
                crate::DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag(),
            )
            .unwrap();
        let stamp = dom.snapshot().stamp();
        let snapshot = cap.snapshot().unwrap();
        let sheet = snapshot.sheets()[0].clone();
        let revision = snapshot.revision().clone();
        let mut hold = dom
            .begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
            .unwrap();
        let prep = hold.prepare_successor(Some(&cap), &flag()).unwrap();
        let left =
            prep.meter.entry().control.limits().work() - prep.meter.entry().control.spent_work();
        // Exercise the real original controller, leaving a bounded tail. These
        // are actual admitted owner steps, not a replacement/lowered policy.
        prep.meter.spend(left - remaining, &flag()).unwrap();
        let count = PREPARED.with(|value| value.get());
        let result = prep.commit(&flag());
        let post_prepare = PREPARED.with(|value| value.get()) > count;
        let failed = matches!(
            result,
            Err(Error::Style(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: crate::DomResource::ParsedAssociationProcessingWork,
                    ..
                })
            )))
        );
        drop(result);
        let spent = hold.spent_work();
        drop(hold);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(cap.snapshot().unwrap().revision(), &revision);
        assert!(
            cap.snapshot()
                .unwrap()
                .sheet(&sheet)
                .unwrap()
                .inputs()
                .owner
                .is_some()
        );
        assert_eq!(
            dom.parsed_html_style_prefix(id).unwrap().qualification(),
            Some(&q)
        );
        if failed && post_prepare {
            let raised =
                crate::DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 2_000_000, 1024)
                    .unwrap();
            let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
            assert!(hold.spent_work() >= spent);
            let (completion, retirement) = hold
                .prepare_successor(Some(&cap), &flag())
                .unwrap()
                .commit(&flag())
                .unwrap()
                .into_parts();
            drop(retirement);
            assert_ne!(completion.qualification(), &q);
            drop(hold);
            let snapshot = cap.snapshot().unwrap();
            assert!(snapshot.sheets().is_empty());
            assert!(snapshot.sheet(&sheet).unwrap().inputs().owner.is_none());
            reached = true;
            break;
        }
    }
    assert!(
        reached,
        "bounded genuine original work tail must reach CSSOM preparation before refusal"
    );
}
#[test]
fn authentic_unsupported_influence_diagnostic_is_admitted_before_identity_output() {
    use super::super::tests::{install, parse, setup};
    let (mut dom, env) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><head><meta http-equiv=default-style content=A></head><body>"),
        env,
    );
    let stamp = dom.snapshot().stamp();
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap();
    let spent = {
        let hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        let mut cursor = hold
            .meter
            .entry()
            .report
            .value()
            .value
            .pending_associations();
        loop {
            hold.step(&flag()).unwrap();
            match cursor.advance() {
                Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => {
                    assert!(matches!(
                        remaining(&hold.meter, &row, document, &flag()),
                        Err(DomError::ResourceLimit {
                            resource: crate::DomResource::ParsedAssociationProcessingOutputs,
                            limit: 0
                        })
                    ));
                    break;
                }
                Step::Complete => panic!("authentic Meta influence"),
                _ => (),
            }
        }
        hold.spent_work()
    };
    assert_eq!(dom.snapshot().stamp(), stamp);
    let raised = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1).unwrap();
    let hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() >= spent);
    let mut cursor = hold
        .meter
        .entry()
        .report
        .value()
        .value
        .pending_associations();
    loop {
        hold.step(&flag()).unwrap();
        match cursor.advance() {
            Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => {
                let identity = row.id();
                assert!(
                    matches!(remaining(&hold.meter, &row, document, &flag()).unwrap(), Error::Remaining(DomParsedHtmlStyleNotificationRequirement::Occurrence { contribution, occurrence, document: actual, kind: Kind::ChangePreferred }) if contribution==id && occurrence==identity && actual==document)
                );
                break;
            }
            Step::Complete => panic!("authentic Meta influence"),
            _ => (),
        }
    }
    assert!(
        hold.meter
            .entry()
            .style_prefix
            .try_lock()
            .unwrap()
            .is_none(),
        "diagnostic formation executes nothing"
    );
}
#[test]
fn different_real_same_store_ownerless_sheet_cannot_substitute_for_completed_invocation() {
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
    let (mut dom, _doc, id, q, element, cap) = ready("", 1024);
    dom.begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap();
    let old = dom
        .snapshot()
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .sheet
        .clone()
        .unwrap();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
            .unwrap();
        let (_completion, retirement) = hold
            .prepare_successor(Some(&cap), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retirement);
    }
    let other = {
        let mut owner = cap.lock().unwrap();
        let mut batch = owner
            .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
            .unwrap();
        let ticket = batch
            .create_parsed_sheet(
                "",
                CssomSheetInputs::constructed(
                    CssomInputVersion {
                        role: CssomInputRole::Document,
                        identity: "other".into(),
                        revision: 1,
                    },
                    "document".into(),
                    None,
                ),
            )
            .unwrap();
        let published = owner.commit(batch).unwrap();
        let sheet = published.sheet(&ticket).unwrap().clone();
        assert!(
            owner
                .snapshot()
                .sheet(&sheet)
                .unwrap()
                .inputs()
                .owner
                .is_none()
        );
        sheet
    };
    // A deliberately invalid claim containing a real, resolved same-owner ID,
    // not a fabricated execution record or a fake producer publication.
    assert_ne!(old.id, other);
    let mut input = None;
    let mut hold = dom
        .begin_parsed_html_style(id, super::super::tests::envelope(), &flag())
        .unwrap();
    hold.transfer_notification(&q, &mut input, &flag()).unwrap();
    assert_eq!(input.as_ref().unwrap().sheet(), &old);
    assert_eq!(
        input.as_ref().unwrap().sheet().association().element(),
        element
    );
    let prep = hold
        .prepare_notification(&mut input, "owner", &cap, &flag())
        .unwrap();
    assert!(
        prep.owner_association_removed(),
        "wrong resolved sheet issued no terminal disposition"
    );
}
