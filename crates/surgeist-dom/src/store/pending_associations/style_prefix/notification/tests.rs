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
) {
    let (mut dom, _document, id, q, element, cap) = ready("", 1024);
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .process_rendering(&q, &flag())
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
    (dom, id, q, cap, sheet)
}
#[test]
fn real_busy_poisoned_and_missing_object_are_not_owner_removal() {
    let (mut dom, id, q, cap, sheet) = rendered();
    let root = q.document().node();
    {
        let locked = cap.lock().unwrap();
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(
            matches!(hold.prepare_notification(&q,&sheet,"owner",&cap,&flag()),Err(Error::OwnerBusy{root:actual}) if actual==root)
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
    let mut wrong = sheet.clone();
    wrong.id = ghost;
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.prepare_notification(&q, &wrong, "owner", &cap, &flag()),
            Err(Error::Cssom(surgeist_cssom::CssomError::MissingObject))
        ));
    }
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = cap.lock().unwrap();
        panic!("genuine owner failure");
    }));
    assert!(poisoned.is_err());
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_notification(&q,&sheet,"owner",&cap,&flag()),Err(Error::OwnerPoisoned{root:actual}) if actual==root)
    );
}
#[test]
fn token_keeps_actual_original_entry_and_frame_debit_after_canonical_owner_drop() {
    let (mut dom, id, q, cap, sheet) = rendered();
    let token = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let prep = hold
            .prepare_notification(&q, &sheet, "owner", &cap, &flag())
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
    let (mut dom, id, q, cap, sheet) = rendered();
    let mut token = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut published = hold
            .prepare_notification(&q, &sheet, "owner", &cap, &flag())
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
    assert!(
        matches!(dom.finish_style_notification(&mut token, 1024, &flag()),Err(DomError::ParsedAssociationSynchronousPending{pending}) if pending==id)
    );
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
