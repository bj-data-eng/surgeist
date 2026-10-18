//! Explicit canonical history fixtures, without a fabricated active Window or
//! completed insertion/focus receipt. Runtime tests own actual hosted insertion.
use super::*;
use crate::Namespace;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
thread_local! {
    static CUT_RETIREMENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static RETIREMENT_CUT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}
pub(super) fn after_retirement_document(draft: &mut Draft) {
    if CUT_RETIREMENT.replace(false) {
        let spent = draft.source_work.load(Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = spent;
        RETIREMENT_CUT.set(Some(spent));
    }
}
struct RetirementCutGuard;
impl Drop for RetirementCutGuard {
    fn drop(&mut self) {
        CUT_RETIREMENT.set(false);
        RETIREMENT_CUT.set(None);
    }
}
fn source_limits(work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(64_000_000, 1_000_000, work, 64, 4096, 4096, 2)
}
fn fixture() -> (Dom, [DocumentId; 2], [NodeId; 3]) {
    let mut dom = Dom::new(
        DomLimits::new(64, 100_000, 256, 128, 128, 16, 1_000_000)
            .unwrap()
            .with_autofocus_candidates(16)
            .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    let a = b.create_document();
    let d = b.create_document();
    let first = b.create_element_ns(&a, Namespace::html(), "div");
    let middle = b.create_element_ns(&a, Namespace::html(), "span");
    let last = b.create_element_ns(&a, Namespace::html(), "div");
    let c = dom.apply(b, &cancel()).unwrap();
    (
        dom,
        [c.document(&a).unwrap(), c.document(&d).unwrap()],
        [
            c.created(&first).unwrap().unwrap(),
            c.created(&middle).unwrap().unwrap(),
            c.created(&last).unwrap().unwrap(),
        ],
    )
}
fn draft(dom: &Dom) -> Draft {
    let b = DomBatch::new();
    Draft::new_source(
        dom,
        b.owner.clone(),
        &b.commands,
        source_limits(20_000_000),
        &cancel(),
    )
    .unwrap()
    .0
}
fn publish(dom: &mut Dom, draft: &Draft) -> DomCommit {
    dom.publish(draft.finish(dom, &cancel()).unwrap(), &cancel())
        .unwrap()
}
fn history(dom: &mut Dom, docs: [DocumentId; 2], ids: [NodeId; 3]) {
    let mut d = draft(dom);
    for doc in docs {
        let key = d.resolve(doc.node().into()).unwrap();
        for id in ids {
            let subject = d.resolve(id.into()).unwrap();
            d.autofocus_append(key, subject, dom.limits, &cancel())
                .unwrap();
        }
    }
    let second = d.resolve(docs[1].node().into()).unwrap();
    d.autofocus_set_processed(second, &cancel()).unwrap();
    publish(dom, &d);
}
fn candidates(dom: &Dom, doc: DocumentId) -> Vec<NodeId> {
    dom.snapshot()
        .document(doc)
        .unwrap()
        .autofocus()
        .unwrap()
        .candidates()
        .to_vec()
}

#[test]
fn absent_autofocus_removal_does_not_walk_unrelated_candidates_or_other_roles() {
    for unrelated in [0, 256] {
        let cancel = cancel();
        let mut dom = Dom::new(
            DomLimits::new(1024, 1_000_000, 2048, 0, 0, 16, 1_000_000)
                .unwrap()
                .with_autofocus_candidates(1024)
                .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let absent = batch.create_element_ns(&document, Namespace::html(), "div");
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| batch.create_element_ns(&document, Namespace::html(), "span"))
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let document = setup.document(&document).unwrap();
        let absent = setup.created(&absent).unwrap().unwrap();
        let members: Vec<_> = tickets
            .iter()
            .map(|ticket| setup.created(ticket).unwrap().unwrap())
            .collect();
        let mut staged = draft(&dom);
        let owner = staged.resolve(document.into()).unwrap();
        for member in &members {
            let subject = staged.resolve((*member).into()).unwrap();
            staged
                .autofocus_append(owner, subject, dom.limits, &cancel)
                .unwrap();
        }
        // The subject is genuinely present in another role, not autofocus.
        let subject = staged.resolve(absent.into()).unwrap();
        staged.add_top_layer(subject, &cancel).unwrap();
        publish(&mut dom, &staged);
        let old = dom.snapshot();
        let mut staged = draft(&dom);
        let owner = staged.resolve(document.into()).unwrap();
        let subject = staged.resolve(absent.into()).unwrap();
        let before: Vec<_> = staged.autofocus_state(owner).unwrap().0.iter().collect();
        let spent = staged.source_work.load(Ordering::Relaxed);
        staged.source_limits = Some(source_limits(spent + 64));
        let result = staged.autofocus_remove(owner, subject, &cancel);
        assert_eq!(
            staged
                .autofocus_state(owner)
                .unwrap()
                .0
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        staged.source_limits = Some(source_limits(100_000_000));
        staged.autofocus_remove(owner, subject, &cancel).unwrap();
        assert_eq!(
            old.document(document)
                .unwrap()
                .autofocus()
                .unwrap()
                .candidates(),
            members
        );
        assert_eq!(
            old.document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            &[absent]
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        assert_eq!(
            result,
            Ok(()),
            "zero matching autofocus candidates, unrelated={unrelated}"
        );
    }
}

#[test]
fn native_initial_history_and_unavailable_history_have_distinct_readback() {
    let (mut dom, docs, _) = fixture();
    let original = dom.snapshot();
    let view = original.document(docs[0]).unwrap().autofocus().unwrap();
    assert_eq!(view.stamp(), original.document(docs[0]).unwrap().stamp());
    assert_eq!(view.candidates(), &[]);
    assert!(!view.processed());
    let mut d = draft(&dom);
    let key = d.resolve(docs[0].node().into()).unwrap();
    d.with_node_fields(key, &cancel(), |fields| {
        fields.autofocus_document = Some(State::Unavailable)
    })
    .unwrap();
    publish(&mut dom, &d);
    assert_eq!(
        dom.snapshot()
            .document(docs[0])
            .unwrap()
            .autofocus()
            .unwrap_err(),
        DomError::AutofocusDocumentStateUnavailable
    );
    assert!(original.document(docs[0]).unwrap().autofocus().is_ok());
    assert!(
        dom.snapshot()
            .document(docs[0])
            .unwrap()
            .top_layer()
            .is_ok()
    );
}

#[test]
fn source_ordered_remove_append_clear_and_processed_writes_preserve_other_documents() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let old = dom.snapshot();
    let mut d = draft(&dom);
    let doc = d.resolve(docs[0].node().into()).unwrap();
    let subject = d.resolve(ids[0].into()).unwrap();
    d.autofocus_remove(doc, subject, &cancel()).unwrap();
    d.autofocus_append(doc, subject, dom.limits, &cancel())
        .unwrap();
    publish(&mut dom, &d);
    assert_eq!(candidates(&dom, docs[0]), [ids[1], ids[2], ids[0]]);
    assert_eq!(candidates(&dom, docs[1]), ids);
    let mut d = draft(&dom);
    let doc = d.resolve(docs[0].node().into()).unwrap();
    d.autofocus_clear(doc, &cancel()).unwrap();
    d.autofocus_set_processed(doc, &cancel()).unwrap();
    let c = publish(&mut dom, &d);
    assert_eq!(
        c.changes().changes(),
        &[DomChange::DocumentAutofocusState(docs[0])]
    );
    assert_eq!(candidates(&dom, docs[0]), []);
    assert!(
        dom.snapshot()
            .document(docs[0])
            .unwrap()
            .autofocus()
            .unwrap()
            .processed()
    );
    assert_eq!(
        old.document(docs[0])
            .unwrap()
            .autofocus()
            .unwrap()
            .candidates(),
        ids
    );
    assert!(
        !old.document(docs[0])
            .unwrap()
            .autofocus()
            .unwrap()
            .processed()
    );
    assert_eq!(dom.usage().autofocus_candidates(), 3);
}

#[test]
fn detach_adopt_attribute_changes_and_inactive_insertion_retain_historical_membership() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let mut b = DomBatch::new();
    b.insert_before(ids[0], docs[0], None);
    b.set_attribute(ids[0], "autofocus", "");
    dom.apply(b, &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.detach(ids[0]);
    b.adopt(ids[0], docs[1]);
    b.remove_attribute(ids[0], "autofocus");
    dom.apply(b, &cancel()).unwrap();
    for doc in docs {
        assert_eq!(candidates(&dom, doc), ids);
    }
    assert!(
        dom.snapshot()
            .document(docs[1])
            .unwrap()
            .autofocus()
            .unwrap()
            .processed()
    );
}

#[test]
fn exact_retirement_cleans_every_document_stably_without_changing_processed_or_snapshots() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(ids[1], docs[1]);
    dom.apply(b, &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.destroy(ids[1]);
    let c = dom.apply(b, &cancel()).unwrap();
    for (i, doc) in docs.into_iter().enumerate() {
        assert_eq!(candidates(&dom, doc), [ids[0], ids[2]]);
        assert_eq!(
            dom.snapshot()
                .document(doc)
                .unwrap()
                .autofocus()
                .unwrap()
                .processed(),
            i == 1
        );
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::DocumentAutofocusState(doc))
        );
        assert_eq!(
            old.document(doc).unwrap().autofocus().unwrap().candidates(),
            ids
        );
    }
    assert_eq!(dom.usage().autofocus_candidates(), 4);
    assert_eq!(
        dom.snapshot().node(ids[1]).unwrap_err(),
        DomError::StaleNode
    );
    let mut b = DomBatch::new();
    let replacement = b.create_element_ns(docs[0], Namespace::html(), "div");
    let c = dom.apply(b, &cancel()).unwrap();
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, ids[1]);
    for doc in docs {
        assert!(!candidates(&dom, doc).contains(&replacement));
    }
}

#[test]
fn subtree_retirement_removes_descendant_candidates_and_failed_destruction_is_atomic() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let mut b = DomBatch::new();
    b.insert_before(ids[1], ids[0], None);
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.destroy(ids[1]);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut b = DomBatch::new();
    b.destroy(ids[0]);
    dom.apply(b, &cancel()).unwrap();
    for doc in docs {
        assert_eq!(candidates(&dom, doc), [ids[2]]);
    }
}

#[test]
fn aggregate_limit_reduction_and_refused_growth_preserve_actual_history() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let before = dom.snapshot();
    let limits = dom.limits;
    assert_eq!(
        dom.set_limits(limits.with_autofocus_candidates(5).unwrap()),
        Err(DomError::ResourceLimit {
            resource: DomResource::AutofocusCandidates,
            limit: 5
        })
    );
    assert_eq!(dom.limits.autofocus_candidates(), 16);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut d = draft(&dom);
    let doc = d.resolve(docs[0].node().into()).unwrap();
    let subject = d.resolve(ids[0].into()).unwrap();
    assert_eq!(
        d.autofocus_append(
            doc,
            subject,
            limits.with_autofocus_candidates(6).unwrap(),
            &cancel()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::AutofocusCandidates,
            limit: 6
        })
    );
    assert_eq!(d.autofocus_state(doc).unwrap().0.len(), 3);
    assert_eq!(dom.usage().autofocus_candidates(), 6);
}

#[test]
fn canceled_and_late_work_refusal_leave_published_history_then_retry_once() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.destroy(ids[1]);
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    let mut d = draft(&dom);
    let initial = d.source_work.load(Ordering::Relaxed);
    // Cut the original operation after its first real removal. Its next
    // mandatory read has one unit less than required; fresh Drafts can have
    // different custody paths and cannot calibrate this operation's work.
    RETIREMENT_CUT.set(None);
    CUT_RETIREMENT.set(true);
    let _cut_guard = RetirementCutGuard;
    let result = d.retire_autofocus_candidate(d.resolve(ids[1].into()).unwrap(), &cancel());
    let cut = RETIREMENT_CUT.take().expect("first real removal completed");
    assert_eq!(result, Err(DomError::SourceWorkLimit { limit: cut }));
    assert!(cut > initial);
    assert_eq!(d.source_work.load(Ordering::Relaxed), cut);
    let mut lengths = docs.map(|document| {
        let key = d.resolve(document.node().into()).unwrap();
        d.autofocus_state(key).unwrap().0.len()
    });
    lengths.sort();
    assert_eq!(lengths, [2, 3], "one private list changed before refusal");
    // The owning whole retirement transaction is unpublished; its private
    // partial scratch is discarded, preserving canonical lists and generations.
    drop(d);
    for doc in docs {
        assert_eq!(candidates(&dom, doc), ids);
    }
    let mut b = DomBatch::new();
    b.destroy(ids[1]);
    dom.apply(b, &cancel()).unwrap();
    for doc in docs {
        assert_eq!(candidates(&dom, doc), [ids[0], ids[2]]);
    }
}

#[test]
fn exact_unpublished_candidate_retirement_never_enters_the_public_mapping() {
    let (mut dom, docs, _) = fixture();
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(docs[0], Namespace::html(), "div");
    let mut d = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        source_limits(20_000_000),
        &cancel(),
    )
    .unwrap()
    .0;
    for command in &batch.commands {
        d.apply(command, dom.limits, &cancel()).unwrap();
    }
    let doc = d.resolve(docs[0].node().into()).unwrap();
    let key = d.resolve((&subject).into()).unwrap();
    d.autofocus_append(doc, key, dom.limits, &cancel()).unwrap();
    d.apply(&Command::Destroy((&subject).into()), dom.limits, &cancel())
        .unwrap();
    let c = publish(&mut dom, &d);
    assert_eq!(c.created(&subject).unwrap(), None);
    assert!(candidates(&dom, docs[0]).is_empty());
    assert_eq!(dom.usage().autofocus_candidates(), 0);
}

#[test]
fn hosted_shadow_closure_retirement_cleans_exact_historical_candidates() {
    let (mut dom, docs, ids) = fixture();
    let mut b = DomBatch::new();
    let root = b.attach_shadow(
        ids[0],
        crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Closed),
    );
    b.insert_before(ids[1], &root, None);
    dom.apply(b, &cancel()).unwrap();
    history(&mut dom, docs, ids);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.destroy(ids[0]);
    dom.apply(b, &cancel()).unwrap();
    for doc in docs {
        assert_eq!(candidates(&dom, doc), [ids[2]]);
        assert_eq!(
            old.document(doc).unwrap().autofocus().unwrap().candidates(),
            ids
        );
    }
    assert_eq!(
        dom.snapshot().node(ids[1]).unwrap_err(),
        DomError::StaleNode
    );
}

#[test]
fn unpublished_source_retirement_refusal_drop_discards_private_work_then_retries() {
    let (mut dom, docs, ids) = fixture();
    history(&mut dom, docs, ids);
    let old = dom.snapshot();
    // Locate the first genuine initial-owner admission. With this exact Work
    // policy there is no credit for advance's next read; no padding/refund or
    // synthetic parked frame is introduced.
    let mut operation = None;
    for work in 1..16_384 {
        match dom.begin_source_edit(
            crate::DomSourceEdit::destroy(ids[1]),
            source_limits(work),
            &cancel(),
        ) {
            Err(DomError::SourceWorkLimit { .. }) => {}
            Err(error) => panic!("unexpected initial admission: {error:?}"),
            Ok(mut lease) => {
                operation = Some(lease.operation());
                assert!(matches!(
                    lease.advance(&cancel()),
                    Err(DomError::SourceWorkLimit { .. })
                ));
                drop(lease);
                break;
            }
        }
    }
    let operation = operation.expect("finite actual initial admission");
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    for doc in docs {
        assert_eq!(candidates(&dom, doc), ids);
    }
    // An unpublished operation has incurred no canonical obligations: dropping
    // its permission discards it. It cannot be resumed as a retained operation.
    assert!(dom.pending_source_edit().is_none());
    assert!(matches!(
        dom.resume_source_edit(operation, source_limits(20_000_000), &cancel()),
        Err(DomError::InvalidSourceOperation)
    ));
    let mut lease = dom
        .begin_source_edit(
            crate::DomSourceEdit::destroy(ids[1]),
            source_limits(20_000_000),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        crate::DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    for doc in docs {
        assert_eq!(candidates(&dom, doc), [ids[0], ids[2]]);
    }
}

#[test]
fn checkpoint_copies_candidate_backing_and_refused_capacity_growth_releases_formation() {
    let (dom, docs, ids) = fixture();
    let mut d = draft(&dom);
    let doc = d.resolve(docs[0].node().into()).unwrap();
    let subject = d.resolve(ids[0].into()).unwrap();
    let checkpoint = d.source_checkpoint(0, 0, &cancel()).unwrap();
    d.autofocus_append(doc, subject, dom.limits, &cancel())
        .unwrap();
    d.autofocus_set_processed(doc, &cancel()).unwrap();
    checkpoint.restore(&mut d);
    assert!(d.autofocus_state(doc).unwrap().0.is_empty());
    assert!(!d.autofocus_state(doc).unwrap().1);
    let before = d.source_storage_bytes(&cancel()).unwrap() + d.source_other_bytes;
    let bytes = before + size_of::<Key>() - 1;
    d.source_limits = Some(crate::DomSourceLimits::new(
        bytes, 1_000_000, 20_000_000, 64, 4096, 4096, 2,
    ));
    assert_eq!(
        d.autofocus_append(doc, subject, dom.limits, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: bytes
        })
    );
    assert!(d.autofocus_state(doc).unwrap().0.is_empty());
    assert!(!d.autofocus_state(doc).unwrap().1);
    assert_eq!(d.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(d.source_temporary_units.load(Ordering::Relaxed), 0);
    d.source_limits = Some(source_limits(20_000_000));
    d.autofocus_append(doc, subject, dom.limits, &cancel())
        .unwrap();
    assert_eq!(
        d.autofocus_state(doc).unwrap().0.iter().collect::<Vec<_>>(),
        [subject]
    );
}
