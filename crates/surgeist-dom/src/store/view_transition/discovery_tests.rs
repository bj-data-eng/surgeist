// Included in the owning tests module; fixture() uses a real admitted CSSOM sheet.

fn discovery_limits() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(64_000_000, 1_000_000, 20_000, 8, 64, 64, 1)
}

struct DiscoveryFixture {
    dom: Dom,
    document: DocumentId,
    transition: DomViewTransitionId,
    chosen: DomViewTransitionRecordId,
    claimed: DomViewTransitionRecordId,
    target: NodeId,
    spare: NodeId,
}

fn discovery_fixture(unrelated: usize) -> DiscoveryFixture {
    let (mut dom, document) = fixture();
    dom.set_limits(
        DomLimits::new(16, 1_000_000, 4096, 0, 0, 16, 64_000_000)
            .unwrap()
            .with_view_transitions(4, 4096, 16_384)
            .unwrap(),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let root = batch.create_element(document, "root");
    let target = batch.create_element(document, "target");
    let spare = batch.create_element(document, "spare");
    batch.insert_before(&root, document, None);
    batch.insert_before(&target, &root, None);
    batch.insert_before(&spare, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let spare = commit.created(&spare).unwrap().unwrap();
    let mut batch = DomBatch::new();
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    let transition = batch.begin_view_transition(stamp);
    let ident = |value: &str| {
        CssViewTransitionIdent::try_new(CssCustomIdent::try_new(value).unwrap()).unwrap()
    };
    let chosen = batch.append_view_transition_record(stamp, &transition, ident("chosen__"));
    batch.set_view_transition_branches(stamp, &chosen, DomViewTransitionBranches::Old);
    for index in 0..unrelated {
        batch.append_view_transition_record(stamp, &transition, ident(&format!("r{index:07}")));
    }
    // A duplicate's one actual claimant is last, after unrelated records.
    let claimed = batch.append_view_transition_record(stamp, &transition, ident("claimed_"));
    batch.set_view_transition_branches(stamp, &claimed, DomViewTransitionBranches::New(target));
    let commit = dom.apply(batch, &cancel).unwrap();
    DiscoveryFixture {
        dom,
        document,
        transition: commit.view_transition(&transition).unwrap().unwrap(),
        chosen: commit.view_transition_record(&chosen).unwrap().unwrap(),
        claimed: commit.view_transition_record(&claimed).unwrap().unwrap(),
        target,
        spare,
    }
}

#[derive(Clone, Copy, Debug)]
enum ElementQuery {
    Absent,
    SelfClaim,
    OtherClaim,
}

fn measured_element_query(query: ElementQuery) {
    use crate::store::scale_metrics::{Domain, Hooks, Measurement};
    // This diagnostic must fail when explicitly selected without instrumentation.
    // The ordinary public tests separately assert the same product semantics.
    for unrelated in [0, 64, 2048] {
        let mut f = discovery_fixture(unrelated);
        if matches!(query, ElementQuery::SelfClaim) {
            let mut batch = DomBatch::new();
            batch.set_view_transition_branches(
                f.dom.snapshot().document(f.document).unwrap().stamp(),
                f.chosen,
                DomViewTransitionBranches::New(f.spare),
            );
            f.dom.apply(batch, &AtomicBool::new(false)).unwrap();
        }
        let before = f.dom.snapshot();
        let usage = f.dom.usage();
        let target = if matches!(query, ElementQuery::OtherClaim) {
            f.target
        } else {
            f.spare
        };
        let edit = crate::DomSourceEdit::set_view_transition_branches(
            before.document(f.document).unwrap().stamp(),
            f.chosen,
            DomViewTransitionBranches::New(target),
        );
        let cancel = AtomicBool::new(false);
        // Retire setup temporaries before measuring the next real operation.
        // Reclaiming those objects is distinct from selected lookup discovery.
        f.dom.custody_registry.reconcile_releases();
        let measurement = Measurement::begin()
            .expect("explicit diagnostic requires SURGEIST_DOM_STORAGE_COUNTERS=1");
        let result: Result<crate::DomSourceCompletion, DomError> = (|| {
            let mut lease = f.dom.begin_source_edit(edit, discovery_limits(), &cancel)?;
            match lease.advance(&cancel)? {
                crate::DomSourceProgress::Complete(done) => Ok(done),
                crate::DomSourceProgress::Request(request) => {
                    panic!("unexpected VT service {request:?}")
                }
            }
        })();
        let actual = measurement
            .finish(
                Hooks::one(Domain::Bank)
                    .union(Hooks::one(Domain::Accounting))
                    .union(Hooks::one(Domain::Source)),
            )
            .unwrap();
        match query {
            ElementQuery::OtherClaim => {
                assert_eq!(result.unwrap_err(), DomError::InvalidViewTransitionState);
                assert_eq!(f.dom.snapshot().stamp(), before.stamp());
                assert_eq!(f.dom.usage(), usage);
            }
            ElementQuery::SelfClaim => {
                assert!(result.unwrap().commit().changes().changes().is_empty());
                assert_eq!(f.dom.snapshot().stamp(), before.stamp());
                assert_eq!(f.dom.usage(), usage);
            }
            ElementQuery::Absent => {
                assert_eq!(
                    result.unwrap().commit().changes().changes(),
                    &[
                        DomChange::ViewTransition(f.transition),
                        DomChange::ViewTransitionRecord(f.chosen),
                    ]
                );
                assert_eq!(
                    f.dom
                        .snapshot()
                        .view_transition_record(f.chosen)
                        .unwrap()
                        .branches(),
                    DomViewTransitionBranches::New(target)
                );
            }
        }
        assert_eq!(
            before.view_transition_record(f.claimed).unwrap().branches(),
            DomViewTransitionBranches::New(f.target)
        );
        assert!(f.dom.pending_source_edit().is_none());
        assert!(
            actual.canonical_record_reads > 0 && actual.source_steps > 0,
            "missing actual execution: {actual:?}"
        );
        // One fixed-size full NodeId query, selected record and fixed output
        // through the actual Source execution. 1024 is generous for bounded
        // identity/index paths, and excludes visiting 2048 unrelated records.
        assert!(
            actual.canonical_record_reads <= 1024,
            "query={query:?}, unrelated={unrelated}: {actual:?}"
        );
        crate::store::oracle_tests::assert_full_canonical(&f.dom);
    }
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1 and explicit diagnostic selection"]
fn absent_new_element_lookup_excludes_unrelated_transition_records() {
    measured_element_query(ElementQuery::Absent);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1 and explicit diagnostic selection"]
fn self_new_element_noop_excludes_unrelated_transition_records() {
    measured_element_query(ElementQuery::SelfClaim);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1 and explicit diagnostic selection"]
fn duplicate_new_element_lookup_excludes_unrelated_transition_records() {
    measured_element_query(ElementQuery::OtherClaim);
}

#[test]
fn private_record_claims_restore_at_each_original_source_checkpoint() {
    let f = discovery_fixture(64);
    let before = f.dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let stamp = before.document(f.document).unwrap().stamp();
    let ident =
        || CssViewTransitionIdent::try_new(CssCustomIdent::try_new("private_").unwrap()).unwrap();
    let private = batch.append_view_transition_record(stamp, f.transition, ident());
    batch.set_view_transition_branches(stamp, &private, DomViewTransitionBranches::New(f.spare));
    batch.append_view_transition_record(stamp, f.transition, ident());
    let (mut draft, _) = Draft::new_source(
        &f.dom,
        batch.owner.clone(),
        &batch.commands,
        discovery_limits(),
        &cancel,
    )
    .unwrap();
    let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
    draft
        .apply(&batch.commands[0], f.dom.limits(), &cancel)
        .unwrap();
    let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
    draft
        .apply(&batch.commands[1], f.dom.limits(), &cancel)
        .unwrap();
    let (transition, key) = draft
        .view_transitions
        .record(&batch.owner, f.dom.owner(), (&private).into())
        .unwrap();
    assert_eq!(
        draft
            .view_transitions
            .get(transition)
            .unwrap()
            .record(key)
            .unwrap()
            .branches,
        DomViewTransitionBranches::New(f.spare)
    );
    assert_eq!(
        draft.apply(&batch.commands[2], f.dom.limits(), &cancel),
        Err(DomError::DuplicateViewTransitionName)
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    inner.restore(&mut draft);
    assert_eq!(
        draft
            .view_transitions
            .get(transition)
            .unwrap()
            .record(key)
            .unwrap()
            .branches,
        DomViewTransitionBranches::Pending
    );
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    draft
        .apply(&batch.commands[1], f.dom.limits(), &cancel)
        .unwrap();
    let spent = draft.source_work.load(Ordering::Relaxed);
    outer.restore(&mut draft);
    assert!(matches!(
        draft
            .view_transitions
            .record(&batch.owner, f.dom.owner(), (&private).into()),
        Err(DomError::UnavailableCreation)
    ));
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    // The exact same private name/identity operand is reusable after rollback.
    draft
        .apply(&batch.commands[0], f.dom.limits(), &cancel)
        .unwrap();
    draft
        .apply(&batch.commands[1], f.dom.limits(), &cancel)
        .unwrap();
    assert_eq!(*f.dom.state, *before.state);
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
}

#[test]
fn refused_same_owner_branch_storage_leaves_no_claim_and_retry_succeeds() {
    let f = discovery_fixture(64);
    let before = f.dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.set_view_transition_branches(
        before.document(f.document).unwrap().stamp(),
        f.chosen,
        DomViewTransitionBranches::New(f.spare),
    );
    batch.set_view_transition_branches(
        before.document(f.document).unwrap().stamp(),
        f.claimed,
        DomViewTransitionBranches::New(f.spare),
    );
    batch.set_view_transition_branches(
        before.document(f.document).unwrap().stamp(),
        f.claimed,
        DomViewTransitionBranches::New(f.target),
    );
    let (mut draft, _) = Draft::new_source(
        &f.dom,
        batch.owner.clone(),
        &batch.commands,
        discovery_limits(),
        &cancel,
    )
    .unwrap();
    let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
    draft.custody_registry().reconcile_releases();
    let live = draft.source_storage_bytes(&cancel).unwrap();
    draft.source_limits.as_mut().unwrap().retained_bytes = live;
    assert_eq!(
        draft.apply(&batch.commands[0], f.dom.limits(), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: live
        })
    );
    assert_eq!(
        draft
            .view_transitions
            .get(TransitionKey::Existing(f.transition))
            .unwrap()
            .record(RecordKey::Existing(f.chosen))
            .unwrap()
            .branches,
        DomViewTransitionBranches::Old
    );
    draft.source_limits = Some(discovery_limits());
    // A different record can claim the element before the original retry:
    // refusal must not leave an invisible claim owned by the chosen record.
    draft
        .apply(&batch.commands[1], f.dom.limits(), &cancel)
        .unwrap();
    draft
        .apply(&batch.commands[2], f.dom.limits(), &cancel)
        .unwrap();
    draft
        .apply(&batch.commands[0], f.dom.limits(), &cancel)
        .unwrap();
    assert_eq!(
        draft
            .view_transitions
            .get(TransitionKey::Existing(f.transition))
            .unwrap()
            .record(RecordKey::Existing(f.chosen))
            .unwrap()
            .branches,
        DomViewTransitionBranches::New(f.spare)
    );
    checkpoint.restore(&mut draft);
    assert_eq!(
        draft
            .view_transitions
            .get(TransitionKey::Existing(f.transition))
            .unwrap()
            .record(RecordKey::Existing(f.chosen))
            .unwrap()
            .branches,
        DomViewTransitionBranches::Old
    );
    assert_eq!(*f.dom.state, *before.state);
}

#[test]
fn reused_element_slot_keeps_historical_and_current_claims_distinct() {
    let mut f = discovery_fixture(64);
    let before = f.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(f.target);
    batch.destroy(f.target);
    let replacement = batch.create_element(f.document, "replacement");
    let replacement = f
        .dom
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&replacement)
        .unwrap()
        .unwrap();
    assert_eq!(replacement.slot, f.target.slot);
    assert_eq!(replacement.generation, f.target.generation + 1);
    let mut batch = DomBatch::new();
    batch.set_view_transition_branches(
        f.dom.snapshot().document(f.document).unwrap().stamp(),
        f.chosen,
        DomViewTransitionBranches::New(replacement),
    );
    f.dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        f.dom
            .snapshot()
            .view_transition_record(f.claimed)
            .unwrap()
            .branches(),
        DomViewTransitionBranches::New(f.target)
    );
    assert_eq!(
        before
            .view_transition_record(f.claimed)
            .unwrap()
            .resolve_new_element()
            .unwrap()
            .unwrap()
            .id(),
        f.target
    );
    crate::store::oracle_tests::assert_full_canonical(&f.dom);
}
