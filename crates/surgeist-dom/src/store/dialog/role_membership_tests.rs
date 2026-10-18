//! The genuine relation owner is exercised independently of native Show receipts.
//! Populating a canonical list here does not fabricate completed watcher/focus work.
use super::*;

fn policy(work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(256_000_000, 4_000_000, work, 64, 4096, 4096, 2)
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        policy(100_000_000),
        &AtomicBool::new(false),
    )
    .unwrap()
    .0
}
fn fixture(unrelated: usize) -> (Dom, DocumentId, NodeId, Vec<NodeId>) {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(1024, 1_000_000, 2048, 0, 0, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let absent = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let members: Vec<_> = (0..unrelated)
        .map(|_| {
            let member = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
            batch.set_attribute(&member, "open", "");
            member
        })
        .collect();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let absent = commit.created(&absent).unwrap().unwrap();
    let members: Vec<_> = members
        .iter()
        .map(|ticket| commit.created(ticket).unwrap().unwrap())
        .collect();
    let mut staged = draft(&dom);
    let owner = staged.resolve(document.into()).unwrap();
    // Use each real canonical relation primitive; keep roles distinct even
    // though this fixture deliberately gives them overlapping native Elements.
    for id in &members {
        let member = staged.resolve((*id).into()).unwrap();
        for list in [DialogList::Open, DialogList::Raw, DialogList::Pending] {
            staged
                .dialog_list_append(owner, member, list, &cancel)
                .unwrap();
        }
    }
    dom.publish(staged.finish(&dom, &cancel).unwrap(), &cancel)
        .unwrap();
    (dom, document, absent, members)
}
fn absent_role_is_local(list: DialogList) {
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256] {
        let (dom, document, absent, members) = fixture(unrelated);
        let old = dom.snapshot();
        let mut staged = draft(&dom);
        let owner = staged.resolve(document.into()).unwrap();
        let subject = staged.resolve(absent.into()).unwrap();
        let before: Vec<_> = staged.dialog_list(owner, list).unwrap().iter().collect();
        let spent = staged.source_work.load(Ordering::Relaxed);
        assert_eq!(
            staged.dialog_list_contains(owner, subject, list, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        );
        assert_eq!(staged.source_work.load(Ordering::Relaxed), spent);
        staged.source_limits = Some(policy(spent + 64));
        let result = staged.dialog_list_contains(owner, subject, list, &cancel);
        assert_eq!(
            staged
                .dialog_list(owner, list)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        let spent = staged.source_work.load(Ordering::Relaxed);
        staged.source_limits = Some(policy(spent + 64));
        let removal = staged.dialog_list_remove(owner, subject, list, &cancel);
        assert_eq!(
            staged
                .dialog_list(owner, list)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        staged.source_limits = Some(policy(100_000_000));
        assert!(
            !staged
                .dialog_list_contains(owner, subject, list, &cancel)
                .unwrap()
        );
        let checkpoint = staged
            .source_checkpoint(
                staged.source_other_bytes,
                staged.source_other_units,
                &cancel,
            )
            .unwrap();
        staged
            .dialog_list_append(owner, subject, list, &cancel)
            .unwrap();
        staged
            .dialog_list_append(owner, subject, list, &cancel)
            .unwrap();
        assert!(
            staged
                .dialog_list_contains(owner, subject, list, &cancel)
                .unwrap()
        );
        for other in [DialogList::Open, DialogList::Raw, DialogList::Pending] {
            if std::mem::discriminant(&other) != std::mem::discriminant(&list) {
                assert!(
                    !staged
                        .dialog_list_contains(owner, subject, other, &cancel)
                        .unwrap()
                );
                assert_eq!(staged.dialog_list(owner, other).unwrap().len(), unrelated);
            }
        }
        assert_eq!(
            staged.dialog_list(owner, list).unwrap().len(),
            unrelated + 2
        );
        staged
            .dialog_list_remove(owner, subject, list, &cancel)
            .unwrap();
        assert_eq!(
            staged
                .dialog_list(owner, list)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        checkpoint.restore(&mut staged);
        for other in [DialogList::Open, DialogList::Raw, DialogList::Pending] {
            assert_eq!(
                staged
                    .dialog_list(owner, other)
                    .unwrap()
                    .iter()
                    .collect::<Vec<_>>(),
                before
            );
        }
        assert_eq!(
            old.document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            members
        );
        assert_eq!(
            old.document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .pending_removals(),
            members
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        assert_eq!(
            result,
            Ok(false),
            "absent membership must not inspect {unrelated} unrelated rows"
        );
        assert_eq!(
            removal,
            Ok(()),
            "absent removal must not inspect {unrelated} unrelated rows"
        );
    }
}
#[test]
fn absent_open_dialog_membership_is_independent_of_other_open_dialogs() {
    absent_role_is_local(DialogList::Open);
}
#[test]
fn absent_raw_top_layer_membership_is_independent_of_other_raw_members() {
    absent_role_is_local(DialogList::Raw);
}
#[test]
fn absent_pending_removal_membership_is_independent_of_other_pending_members() {
    absent_role_is_local(DialogList::Pending);
}

#[test]
fn hidden_raw_members_do_not_make_the_showing_popover_fact_a_linear_search() {
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256] {
        let (dom, document, _, members) = fixture(unrelated);
        let before = dom.snapshot();
        let mut staged = draft(&dom);
        let owner = staged.resolve(document.into()).unwrap();
        let spent = staged.source_work.load(Ordering::Relaxed);
        staged.source_limits = Some(policy(spent + 64));
        let result = staged.known_empty_popover_lists(owner, &cancel);
        staged.source_limits = Some(policy(100_000_000));
        assert!(staged.known_empty_popover_lists(owner, &cancel).unwrap());
        assert_eq!(
            staged.dialog_list(owner, DialogList::Raw).unwrap().len(),
            unrelated
        );
        assert_eq!(
            before
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            members
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            result,
            Ok(true),
            "hidden raw members are not showing participants: {unrelated}"
        );
    }
}
