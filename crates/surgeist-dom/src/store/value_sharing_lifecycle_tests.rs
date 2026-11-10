//! Functional coverage of the transaction and session ownership seam of the attribute-bank sharing
//! owner: cold and active checkpoints, restore, and closing at the publication fence.
use super::*;
use crate::name::AttributeStorage;
use surgeist_html::values::{Namespace, QualifiedName};

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
    let element = setup.create_element(&document, "div");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
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
    (dom, element, draft)
}

/// Forms a one-attribute bank of `value` through the draft's sharing session, which offers it and
/// probes for an equal one.
fn form(draft: &Draft, value: &str) -> AttributeStorage {
    let cancel = AtomicBool::new(false);
    let raw = QualifiedName::new(Namespace::none(), None, "data-x");
    let name = draft.bind_name(&raw, None, &cancel).unwrap();
    let mut input = draft
        .attribute_input(1, None, &cancel, |entries| {
            entries
                .try_reserve_exact(1)
                .map_err(|_| DomError::RepresentationExhausted)
        })
        .unwrap();
    input.entries.push(crate::DomAttribute::from_parts(
        name,
        DocumentString::from(value),
    ));
    draft.form_attributes(input, None, &cancel).unwrap().value
}

#[test]
fn active_inner_restore_preserves_outer_offer_and_cold_outer_aborts_late_root() {
    let cancel = AtomicBool::new(false);
    let (_dom, _element, mut draft) = source_fixture();
    assert!(draft.sharing_session().unwrap().is_dormant());
    let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(
        outer.sharing().is_none(),
        "a cold boundary owns no pool mark"
    );
    let first = form(&draft, "outer-candidate");
    let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(inner.sharing().is_some());
    let second = form(&draft, "inner-candidate");
    inner.restore(&mut draft);
    assert!(
        form(&draft, "outer-candidate").same_storage(&first),
        "outer offer survives inner restore"
    );
    assert!(
        !form(&draft, "inner-candidate").same_storage(&second),
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
        !form(&draft, "outer-candidate").same_storage(&first),
        "cold restore aborted its late offer"
    );
}

#[test]
fn absent_checkpoint_removes_only_the_attachment_created_after_it() {
    let cancel = AtomicBool::new(false);
    let (dom, _element, mut draft) = source_fixture();
    drop(draft.sharing.take());
    let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(checkpoint.sharing().is_none());
    draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
    let late = form(&draft, "late-attachment");
    checkpoint.restore(&mut draft);
    assert!(draft.sharing_session().is_none());
    draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
    assert!(
        !form(&draft, "late-attachment").same_storage(&late),
        "restored late root owns no offer"
    );
}

#[test]
fn cold_checkpoint_can_close_late_root_at_the_actual_source_publication_fence() {
    let cancel = AtomicBool::new(false);
    let (mut dom, element, mut draft) = source_fixture();
    let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
    let mut edit = DomBatch::new();
    edit.set_attribute(element, "data-x", "published-late-root");
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
    let formed = form(&draft, "published-late-root");
    let DomNodeKind::Element(actual) = dom.state.node(dom.owner, element).unwrap().kind() else {
        panic!("published Element")
    };
    assert!(
        actual
            .attributes_backing()
            .is_some_and(|backing| backing.same_storage(&formed)),
        "the cold checkpoint's drop cannot abort the published offer"
    );
}
