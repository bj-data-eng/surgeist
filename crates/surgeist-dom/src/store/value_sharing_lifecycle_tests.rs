//! Functional coverage of the new private transaction/session ownership seam.
use super::*;

#[test]
fn retained_prepared_root_bypasses_sibling_and_abort_releases_only_its_candidates() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            65_536,
            128,
            0,
            0,
            64,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let document = dom
        .apply(setup, &cancel)
        .unwrap()
        .document(&document)
        .unwrap();

    let mut units = Vec::with_capacity(19);
    units.extend_from_slice(&[120, 0xd800, 0]);
    let value = DocumentString::from_utf16(units);
    let unused = Arc::downgrade(value.shared_code_units());
    let mut first = DomBatch::new();
    let element = first.create_element_ns(document, crate::Namespace::html(), "div");
    first.set_attribute(&element, "data-first", value);
    let first = dom.prepare(first, &cancel).unwrap();
    assert!(unused.upgrade().is_some());

    // A retained first preparation owns the optional root. The independent
    // sibling still prepares against the same canonical tree by bypassing it.
    let mut sibling = DomBatch::new();
    let text = sibling.create_text(document, "sibling");
    let sibling = dom.prepare(sibling, &cancel).unwrap();
    drop(first);
    assert!(
        unused.upgrade().is_none(),
        "an aborted offer owns no strong field"
    );
    let commit = dom.publish(sibling, &cancel).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    assert!(matches!(&dom.state.node(dom.owner, text).unwrap().kind(),
        DomNodeKind::Text(value) if value.code_units() == [115, 105, 98, 108, 105, 110, 103]));

    let mut next = DomBatch::new();
    let a = next.create_text(document, DocumentString::from_utf16(vec![121, 0xd800]));
    let b = next.create_text(document, DocumentString::from_utf16(vec![121, 0xd800]));
    let next = dom.apply(next, &cancel).unwrap();
    let a = next.created(&a).unwrap().unwrap();
    let b = next.created(&b).unwrap().unwrap();
    let (DomNodeKind::Text(a), DomNodeKind::Text(b)) = (
        dom.state.node(dom.owner, a).unwrap().kind(),
        dom.state.node(dom.owner, b).unwrap().kind(),
    ) else {
        panic!("actual Text values")
    };
    assert!(
        Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()),
        "the aborted root leaves the next real transaction usable"
    );
}

fn source_fixture() -> (Dom, NodeId, Draft) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            65_536,
            128,
            0,
            0,
            64,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let text = setup.create_text(&document, "original");
    let text = dom
        .apply(setup, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    let commands = DomBatch::new();
    let draft = Draft::new_source(
        &dom,
        commands.owner,
        &commands.commands,
        crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 16, 64, 64, 4),
        &cancel,
    )
    .unwrap()
    .0;
    (dom, text, draft)
}

fn share(draft: &Draft, value: &DocumentString) -> Option<value_sharing::SelectedString> {
    draft
        .with_sharing_admission(&AtomicBool::new(false), |admission| {
            draft
                .sharing_session()
                .unwrap()
                .share_string(value, admission)
        })
        .unwrap()
}

#[test]
fn active_inner_restore_preserves_outer_offer_and_cold_outer_aborts_late_root() {
    let cancel = AtomicBool::new(false);
    let (_dom, _text, mut draft) = source_fixture();
    assert!(draft.sharing_session().unwrap().is_dormant());
    let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(
        outer.sharing().is_none(),
        "a cold boundary owns no pool mark"
    );
    let first = DocumentString::from("outer-candidate");
    assert!(share(&draft, &first).is_none());
    let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(inner.sharing().is_some());
    let second = DocumentString::from("inner-candidate");
    assert!(share(&draft, &second).is_none());
    inner.restore(&mut draft);
    assert!(
        share(&draft, &first).is_some(),
        "outer offer survives inner restore"
    );
    assert!(
        share(&draft, &second).is_none(),
        "inner offer was rolled back"
    );

    let spent = draft.source_work.load(Ordering::Relaxed);
    draft.source_limits.as_mut().unwrap().work = spent;
    cancel.store(true, Ordering::Relaxed);
    outer.restore(&mut draft);
    assert!(draft.sharing_session().unwrap().is_dormant());
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    // Restoration needs no allocation, work or cancellation permission. A
    // subsequent attempt can activate normally with the same attachment.
    draft.source_limits.as_mut().unwrap().work = 1_000_000;
    assert!(
        share(&draft, &first).is_none(),
        "cold restore aborted its late offer"
    );
}

#[test]
fn absent_checkpoint_removes_only_the_attachment_created_after_it() {
    let cancel = AtomicBool::new(false);
    let (dom, _text, mut draft) = source_fixture();
    drop(draft.sharing.take());
    let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(checkpoint.sharing().is_none());
    draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
    let value = DocumentString::from("late-attachment");
    assert!(share(&draft, &value).is_none());
    checkpoint.restore(&mut draft);
    assert!(draft.sharing_session().is_none());
    draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
    assert!(
        share(&draft, &value).is_none(),
        "restored late root owns no offer"
    );
}

#[test]
fn cold_checkpoint_can_close_late_root_at_the_actual_source_publication_fence() {
    let cancel = AtomicBool::new(false);
    let (mut dom, text, mut draft) = source_fixture();
    let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
    let mut edit = DomBatch::new();
    edit.replace_data(text, 0, usize::MAX, "published-late-root");
    draft.apply(&edit.commands[0], dom.limits, &cancel).unwrap();
    assert!(checkpoint.sharing().is_none());
    assert!(!draft.sharing_session().unwrap().is_dormant());
    let prepared = draft.finish_segment(&dom, &cancel).unwrap();
    let mut publication = dom.prepare_publication(prepared, &cancel).unwrap();
    let rebase = draft.prepare_rebase(&mut publication, &cancel).unwrap();
    let sharing = draft
        .prepare_sharing_publication(publication.changed, Some(&checkpoint), &cancel)
        .unwrap();
    draft.close_sharing_publication(&mut publication, sharing, Some(&checkpoint));
    let committed = dom.publish_prepared_with_proof(publication);
    draft.commit_publication(rebase, &committed.committed);
    drop(checkpoint);
    draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
    let selected = share(&draft, &DocumentString::from("published-late-root"))
        .expect("the cold checkpoint's drop cannot abort the published offer")
        .into_held();
    let DomNodeKind::Text(actual) = dom.state.node(dom.owner, text).unwrap().kind() else {
        panic!("published Text")
    };
    assert!(Arc::ptr_eq(
        actual.shared_code_units(),
        selected.value().shared_code_units()
    ));
}
