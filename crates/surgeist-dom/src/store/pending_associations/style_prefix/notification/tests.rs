//! Genuine owner locks, aborted producer identity and last-owner token custody.
use super::super::rendering::tests::ready;
use super::super::tests::{envelope, flag};
use super::*;
use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
fn rendered() -> (
    Dom,
    DomParsedHtmlAssociationId,
    DomParsedHtmlStyleQualification,
    DomStyleCssomRoot,
    DomStyleSheet,
    Option<DomParsedHtmlStyleNotificationContinuation>,
) {
    let (mut dom, _document, id, q, element, cap) = ready("", 1024);
    let mut input = None;
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Transfer(&mut input),
            &flag(),
        )
        .unwrap();
    let sheet = dom
        .snapshot()
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .sheet
        .clone()
        .unwrap();
    (dom, id, q, cap, sheet, input)
}
#[test]
fn real_busy_poisoned_and_missing_object_are_not_owner_removal() {
    let (mut dom, id, q, cap, sheet, mut input) = rendered();
    let root = q.document().node();
    {
        let locked = cap.lock().unwrap();
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(
            matches!(hold.prepare_notification(&mut input,"owner",&cap,&flag()),Err(Error::OwnerBusy{root:actual}) if actual==root)
        );
        drop(locked);
    }
    let ghost = {
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
                        identity: "ghost".into(),
                        revision: 1,
                    },
                    "document".into(),
                    None,
                ),
            )
            .unwrap();
        let prepared = owner.prepare_commit(batch).unwrap();
        prepared.sheet(&ticket).unwrap().id().clone()
    };
    assert!(matches!(
        cap.snapshot().unwrap().sheet(&ghost),
        Err(surgeist_cssom::CssomError::MissingObject)
    ));
    assert_eq!(input.as_ref().unwrap().sheet(), &sheet);
    assert_eq!(input.as_ref().unwrap().qualification(), &q);
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = cap.lock().unwrap();
        panic!("genuine owner failure");
    }));
    assert!(poisoned.is_err());
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_notification(&mut input,"owner",&cap,&flag()),Err(Error::OwnerPoisoned{root:actual}) if actual==root)
    );
}
#[test]
fn token_keeps_actual_original_entry_and_frame_debit_after_canonical_owner_drop() {
    let (mut dom, id, q, cap, sheet, mut input) = rendered();
    assert_eq!(input.as_ref().unwrap().qualification(), &q);
    assert_eq!(input.as_ref().unwrap().sheet(), &sheet);
    let token = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let prep = hold
            .prepare_notification(&mut input, "owner", &cap, &flag())
            .unwrap();
        let mut published = prep.commit(&flag()).unwrap();
        let DomParsedHtmlStyleNotificationDisposition::Token(token) =
            published.take_disposition().unwrap()
        else {
            panic!("actual owner")
        };
        token
    };
    let (weak, ledger) = {
        let custody = token.parsed.as_ref().unwrap();
        let meter = custody.meter.lock().unwrap();
        (
            Arc::downgrade(&meter.entry().report.value().value),
            dom.custody_registry.ledger().clone(),
        )
    };
    drop(dom);
    drop(cap);
    assert!(
        weak.upgrade().is_some(),
        "token owns authentic original entry, not caller completion"
    );
    let retained = ledger.total();
    drop(token);
    assert!(weak.upgrade().is_none());
    assert!(ledger.total().bytes() < retained.bytes());
}
#[test]
fn forbidden_cleanup_keeps_canonical_membership_and_token_phase_unchanged() {
    let (mut dom, id, q, cap, sheet, mut input) = rendered();
    let mut token = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut published = hold
            .prepare_notification(&mut input, "owner", &cap, &flag())
            .unwrap()
            .commit(&flag())
            .unwrap();
        let DomParsedHtmlStyleNotificationDisposition::Token(token) =
            published.take_disposition().unwrap()
        else {
            panic!("actual owner")
        };
        token
    };
    let stamp = dom.snapshot().stamp();
    assert!(
        dom.snapshot()
            .node(q.document().node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking
            .contains(&sheet.association().element())
    );
    assert!(matches!(
        dom.finish_style_notification(
            &mut token,
            crate::DomStyleNotificationDelivery::Native,
            1024,
            &flag()
        ),
        Err(DomError::StyleProtocol)
    ));
    assert!(!token.complete);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(
        dom.snapshot()
            .node(q.document().node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking
            .contains(&sheet.association().element())
    );
}

#[test]
fn explicit_continuation_and_token_drop_do_not_settle_rendered_work() {
    for token_drop in [false, true] {
        let (mut dom, id, _q, cap, _sheet, mut input) = rendered();
        assert_eq!(
            dom.pending_associations
                .controlled_entry(id)
                .unwrap()
                .value()
                .control
                .notification_count(),
            1
        );
        if token_drop {
            let mut hold = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            let mut published = hold
                .prepare_notification(&mut input, "owner", &cap, &flag())
                .unwrap()
                .commit(&flag())
                .unwrap();
            let token = published.take_disposition().unwrap();
            drop(published);
            drop(hold);
            drop(token);
        } else {
            drop(input.take());
        }
        assert_eq!(
            dom.pending_associations
                .controlled_entry(id)
                .unwrap()
                .value()
                .control
                .notification_count(),
            1,
            "storage release cannot acknowledge actual notification work"
        );
    }
}
#[test]
fn trusted_style_attestation_moves_once_and_its_drop_keeps_the_obligation() {
    let (mut dom, id, _q, cap, _sheet, mut input) = rendered();
    let mut token = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut published = hold
            .prepare_notification(&mut input, "owner", &cap, &flag())
            .unwrap()
            .commit(&flag())
            .unwrap();
        let DomParsedHtmlStyleNotificationDisposition::Token(token) =
            published.take_disposition().unwrap()
        else {
            panic!("actual owner")
        };
        token
    };
    let (mut foreign, _) = super::super::tests::setup();
    let foreign_executor = foreign.bind_source_event_executor(&flag()).unwrap();
    assert!(matches!(
        foreign_executor.seal_style_notification(&dom, &mut token, &flag()),
        Err(DomError::ForeignOwner)
    ));
    assert!(matches!(
        dom.finish_style_notification(
            &mut token,
            crate::DomStyleNotificationDelivery::Native,
            1024,
            &flag()
        ),
        Err(DomError::StyleProtocol)
    ));
    let mut missing = None;
    assert!(matches!(
        dom.finish_style_notification(
            &mut token,
            crate::DomStyleNotificationDelivery::Parsed(&mut missing),
            1024,
            &flag()
        ),
        Err(DomError::StyleProtocol)
    ));
    let executor = dom.bind_source_event_executor(&flag()).unwrap();
    // The external executor is the actual trusted attestation boundary. These
    // owning checks exercise its affine authority; Runtime tests use receipts.
    let publication = executor
        .seal_style_notification(&dom, &mut token, &flag())
        .unwrap();
    assert!(matches!(
        executor.seal_style_notification(&dom, &mut token, &flag()),
        Err(DomError::StyleProtocol)
    ));
    drop(publication);
    assert_eq!(
        dom.pending_associations
            .controlled_entry(id)
            .unwrap()
            .value()
            .control
            .notification_count(),
        1
    );
    assert!(matches!(
        dom.finish_style_notification(
            &mut token,
            crate::DomStyleNotificationDelivery::Parsed(&mut missing),
            1024,
            &flag()
        ),
        Err(DomError::StyleProtocol)
    ));
}
#[test]
fn actual_owner_removed_commit_settles_exactly_once() {
    let (mut dom, id, _q, cap, sheet, mut input) = rendered();
    let snapshot = cap.snapshot().unwrap();
    let mut batch = Some(cap.batch(CssomEditGuard::from_snapshot(&snapshot)).unwrap());
    assert!(batch.as_mut().unwrap().remove_sheet(sheet.id()).unwrap());
    cap.commit(&mut batch).unwrap();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let prep = hold
        .prepare_notification(&mut input, "owner", &cap, &flag())
        .unwrap();
    assert!(prep.owner_association_removed());
    let mut published = prep.commit(&flag()).unwrap();
    assert!(matches!(
        published.take_disposition(),
        Some(DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved { .. })
    ));
    drop(published);
    assert_eq!(hold.meter.entry().control.notification_count(), 0);
    assert!(matches!(
        hold.prepare_notification(&mut input, "owner", &cap, &flag()),
        Err(Error::MissingContinuation)
    ));
    assert_eq!(hold.meter.entry().control.notification_count(), 0);
}
