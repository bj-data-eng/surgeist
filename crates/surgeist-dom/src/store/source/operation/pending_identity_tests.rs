//! A real external acknowledgment survives isolated private payload refusal.
use super::*;

fn policy() -> DomSourceLimits {
    DomSourceLimits::new(256_000_000, 4_000_000, 100_000_000, 64, 4096, 4096, 1)
}

#[test]
fn acknowledged_cssom_receipt_survives_pending_identity_refusal_and_same_source_retry() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            100_000,
            128,
            0,
            0,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let source = setup.create_document();
    let document = setup.create_html_document(&source, None);
    let created = dom.apply(setup, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let snapshot = dom.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[1];
    let head = snapshot.node(html).unwrap().children()[0];
    let mut setup = DomBatch::new();
    let style = setup.create_element_ns(document, crate::Namespace::html(), "style");
    let text = setup.create_text(document, "a { color: red }");
    setup.insert_before(&text, &style, None);
    let created = dom.apply(setup, &cancel).unwrap();
    let style = created.created(&style).unwrap().unwrap();
    let text = created.created(&text).unwrap().unwrap();
    let historical = dom.snapshot();
    assert_eq!(historical.node(style).unwrap().parent(), None);
    assert!(historical.node(head).unwrap().children().is_empty());
    assert!(
        historical
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        historical.node(text).unwrap().data(),
        Some(&DocumentString::from("a { color: red }"))
    );

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, head, None),
            policy(),
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(csp)) = lease.advance(&cancel).unwrap()
    else {
        panic!("actual inline CSP request");
    };
    lease
        .accept_inline_csp(
            &csp,
            &DomStyleInlineCspObservation::complete_empty(csp.source(), 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual CSSOM CreateSheet request");
    };
    assert_eq!(create.action(), DomStyleAction::Update(Action::CreateSheet));
    assert_eq!(
        create.source().text().as_slice(),
        DocumentString::from("a { color: red }").code_units()
    );
    // This existing helper commits an actual CssomStore edit and transfers
    // that non-Clone publication through accept_cssom_publication exactly once.
    let root = accounting_tests::commit_actual_empty_sheet(&mut lease, &create, &cancel);
    let cssom = root.snapshot().unwrap();
    let actual_sheet = cssom.sheets()[0].clone();
    assert_eq!(cssom.sheets().len(), 1);
    let prefix = lease.snapshot();
    let (revision, receipt_sheet, frame, index, invocation, work) = {
        let op = lease.dom.source_operation.as_ref().unwrap();
        let Some(Receipt::Cssom { revision, sheet }) = &op.receipt else {
            panic!("genuine CSSOM acknowledgment retained before private tail");
        };
        assert_eq!(revision, cssom.revision());
        assert_eq!(sheet.as_ref(), Some(&actual_sheet));
        assert_eq!(op.request.as_ref().unwrap().action(), create.action());
        assert!(Arc::ptr_eq(
            &op.request.as_ref().unwrap().source,
            &create.source
        ));
        (
            revision.clone(),
            sheet.clone(),
            op.frame().unwrap(),
            op.index,
            op.draft.source_invocation,
            op.draft.source_work.source_arc().clone(),
        )
    };
    let spent = work.load(Ordering::Relaxed);
    let refusal = payload::identity_refusal::Refusal::begin();
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        DomError::RepresentationExhausted
    );
    assert!(
        refusal.reached(),
        "the actual new Pending identity reservation must execute"
    );
    drop(refusal);
    assert_eq!(lease.snapshot().stamp(), prefix.stamp());
    assert!(Arc::ptr_eq(&lease.snapshot().state, &prefix.state));
    assert_eq!(root.snapshot().unwrap().revision(), cssom.revision());
    assert_eq!(
        root.snapshot().unwrap().sheets(),
        std::slice::from_ref(&actual_sheet)
    );
    let failed_spent = work.load(Ordering::Relaxed);
    assert!(failed_spent >= spent);
    {
        let op = lease.dom.source_operation.as_ref().unwrap();
        assert_eq!(op.id, operation);
        assert_eq!(op.index, index);
        assert_eq!(op.draft.source_invocation, invocation);
        assert!(Arc::ptr_eq(op.draft.source_work.source_arc(), &work));
        let retained = op.frame().unwrap();
        assert_eq!(retained.key, frame.key);
        assert_eq!(retained.cursor.action(), frame.cursor.action());
        assert!(matches!((retained.invocation, frame.invocation),
            (Invocation::Exposed(actual), Invocation::Exposed(original)) if actual == original));
        let Some(Receipt::Cssom {
            revision: retained_revision,
            sheet,
        }) = &op.receipt
        else {
            panic!("private identity refusal must retain the original external receipt");
        };
        assert_eq!(retained_revision, &revision);
        assert_eq!(sheet, &receipt_sheet);
        assert!(Arc::ptr_eq(
            &op.request.as_ref().unwrap().source,
            &create.source
        ));
    }
    drop(lease);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    let mut lease = dom
        .resume_source_edit(operation, policy(), &cancel)
        .unwrap();
    assert!(lease.status().spent_work() >= failed_spent);
    assert!(Arc::ptr_eq(
        lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .source_arc(),
        &work
    ));
    let DomSourceProgress::Complete(completed) = lease.advance(&cancel).unwrap() else {
        panic!(
            "same Source must finish using its original acknowledgment, without another CSSOM request"
        );
    };
    assert_eq!(completed.operation(), operation);
    assert_eq!(root.snapshot().unwrap().revision(), cssom.revision());
    assert_eq!(
        root.snapshot().unwrap().sheets(),
        std::slice::from_ref(&actual_sheet)
    );
    let current = lease.snapshot();
    assert_eq!(current.node(style).unwrap().parent(), Some(head));
    assert_eq!(current.node(head).unwrap().children(), &[style]);
    assert_eq!(current.node(style).unwrap().children(), &[text]);
    let sheet = current
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(sheet.id(), &actual_sheet);
    assert_eq!(
        current.node(text).unwrap().data(),
        Some(&DocumentString::from("a { color: red }"))
    );
    assert_eq!(historical.node(style).unwrap().parent(), None);
    assert!(historical.node(head).unwrap().children().is_empty());
    assert!(
        historical
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        historical.node(text).unwrap().data(),
        Some(&DocumentString::from("a { color: red }"))
    );
    assert_eq!(
        historical.node(document.node()).unwrap().children(),
        snapshot.node(document.node()).unwrap().children()
    );
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
}
