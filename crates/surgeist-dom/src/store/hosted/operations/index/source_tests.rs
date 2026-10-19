//! Composed Source/Host storage proof for the new private projection owner.
use super::*;

fn fixture() -> (
    Dom,
    DomHostedSource,
    Arc<Work>,
    [DomHostedOperationId; 3],
    Draft,
) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            64,
            0,
            0,
            16,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let work = source.work(&cancel).unwrap();
    let ids = std::array::from_fn(|_| source.begin_operation(&work, 1, 0, &cancel).unwrap());
    let batch = DomBatch::new();
    let draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    (dom, source, work, ids, draft)
}

struct Pending {
    plan: Option<Plan>,
    charge: source::checkpoint::Charge,
}
fn prepare(
    index: &Index,
    key: KeyWords,
    id: DomHostedOperationId,
    draft: &Draft,
    source: &DomHostedSource,
    work: &Work,
) -> Result<Pending, DomHostedError> {
    let cancel = AtomicBool::new(false);
    let formation = source::formation::Formation::new(draft, &cancel)?.unwrap();
    let plan = index.prepare(
        key,
        id.slot,
        &mut |extent| {
            formation.retain_bytes(extent.bytes, &cancel)?;
            source.retain(extent.bytes, &cancel).map(Some)
        },
        Some(&formation),
        work,
        &cancel,
    )?;
    let vector = plan.paths.capacity() * size_of::<Option<EmptyPath<Node>>>();
    let raw_components = plan.inspect_storage_bytes() - vector;
    assert_eq!(plan.component_bytes(), raw_components);
    assert_eq!(formation.owned_payload().bytes, raw_components);
    Ok(Pending {
        plan: Some(plan),
        charge: formation.into_charge(),
    })
}

#[test]
fn source_reservation_exact_raw_boundary_and_one_byte_short_share_the_original_owner() {
    let (_dom, source, work, [id, _, _], mut draft) = fixture();
    let key = KeyWords::history(id);
    let row = key.row(id.slot);
    // An independent physical recipe: actual Vec capacity and full empty
    // native ChunkBank reservation, not Plan charge summaries or a prior peak.
    let mut raw_vector = Vec::<Option<EmptyPath<Node>>>::new();
    raw_vector.try_reserve_exact(row.len).unwrap();
    let bytes = raw_vector.capacity() * size_of::<Option<EmptyPath<Node>>>()
        + row.len * ChunkBank::<Node>::inspect_complete_reservation_bytes();
    drop(raw_vector);
    draft.custody_registry().reconcile_releases();
    let ledger = draft.custody_ledger().clone();
    let baseline = ledger.total();
    let external = draft.source_other_bytes;
    let hosted = source.retained_storage_bytes();
    let index = Index::default();
    let exact = baseline.bytes() + external + bytes;
    for short in [true, false] {
        draft.source_limits.as_mut().unwrap().retained_bytes = exact - usize::from(short);
        let work_before = draft.source_work.load(Ordering::Relaxed);
        let result = prepare(&index, key, id, &draft, &source, &work);
        if short {
            assert!(
                matches!(result,Err(DomHostedError::Dom(DomError::ResourceLimit {resource:DomResource::SourceRetainedBytes,limit})) if limit==exact-1)
            );
        } else {
            let pending = result.unwrap();
            assert_eq!(
                pending.plan.as_ref().unwrap().inspect_storage_bytes(),
                bytes
            );
            assert_eq!(
                ledger.total(),
                baseline.checked_add(custody::Usage::new(bytes, 0)).unwrap()
            );
            assert_eq!(source.retained_storage_bytes() - hosted, bytes);
            drop(pending);
        }
        assert_eq!(index.first(key), None);
        assert_eq!(index.inspect_storage_bytes(), 0);
        assert_eq!(ledger.total(), baseline);
        assert_eq!(source.retained_storage_bytes(), hosted);
        assert!(draft.source_work.load(Ordering::Relaxed) > work_before);
    }
}

#[test]
fn source_pending_storage_survives_prefix_prune_and_releases_unused_shared_spares() {
    let (_dom, source, work, [a, b, c], draft) = fixture();
    let key = KeyWords::history(a);
    draft.custody_registry().reconcile_releases();
    let ledger = draft.custody_ledger().clone();
    let baseline = ledger.total();
    let hosted = source.retained_storage_bytes();
    // The semantic owner precedes every moved/temporary Source receipt. A
    // panic during install therefore still drops installed data first.
    struct Owner {
        index: Index,
        pending: Option<Pending>,
        first: Option<source::checkpoint::Charge>,
        second: Option<source::checkpoint::Charge>,
    }
    impl Owner {
        fn install(&mut self, id: DomHostedOperationId) -> usize {
            let plan = self.pending.as_mut().unwrap().plan.take().unwrap();
            let moved = self.index.install(plan, id);
            // install has destroyed its unused spares and Plan Vec already.
            self.pending
                .as_mut()
                .unwrap()
                .charge
                .shrink_to_payload(source::storage::Payload {
                    bytes: moved.bytes,
                    units: 0,
                });
            moved.bytes
        }
    }
    let mut owner = Owner {
        index: Index::default(),
        pending: None,
        first: None,
        second: None,
    };
    owner.pending = Some(prepare(&owner.index, key, a, &draft, &source, &work).unwrap());
    let moved_a = owner.install(a);
    owner.first = Some(owner.pending.take().unwrap().charge);
    assert_eq!(
        ledger.total(),
        baseline
            .checked_add(custody::Usage::new(moved_a, 0))
            .unwrap()
    );
    owner.pending = Some(prepare(&owner.index, key, b, &draft, &source, &work).unwrap());
    let pending_raw = owner
        .pending
        .as_ref()
        .unwrap()
        .plan
        .as_ref()
        .unwrap()
        .inspect_storage_bytes();
    owner.index.retire(key, a);
    assert_eq!(owner.index.inspect_storage_bytes(), 0);
    // The old data is gone before its Source charge. Complete pending paths
    // remain independent of the now-pruned original prefix.
    drop(owner.first.take());
    assert_eq!(
        ledger.total(),
        baseline
            .checked_add(custody::Usage::new(pending_raw, 0))
            .unwrap()
    );
    assert_eq!(source.retained_storage_bytes() - hosted, pending_raw);
    let moved_b = owner.install(b);
    owner.first = Some(owner.pending.take().unwrap().charge);
    assert_eq!(owner.index.first(key), Some(b));
    let old = owner.index.inspect_storage_bytes();
    owner.pending = Some(prepare(&owner.index, key, c, &draft, &source, &work).unwrap());
    let reserved = owner
        .pending
        .as_ref()
        .unwrap()
        .plan
        .as_ref()
        .unwrap()
        .component_bytes();
    let moved_c = owner.install(c);
    owner.second = Some(owner.pending.take().unwrap().charge);
    assert!(moved_c < reserved);
    assert_eq!(owner.index.first(key), Some(b));
    assert_eq!(owner.index.inspect_storage_bytes(), old + moved_c);
    assert_eq!(
        ledger.total(),
        baseline
            .checked_add(custody::Usage::new(moved_b + moved_c, 0))
            .unwrap()
    );
    assert_eq!(
        source.retained_storage_bytes() - hosted,
        owner.index.inspect_storage_bytes()
    );
    owner.index.retire(key, b);
    assert_eq!(owner.index.first(key), Some(c));
    owner.index.retire(key, c);
    assert_eq!(owner.index.inspect_storage_bytes(), 0);
    assert_eq!(source.retained_storage_bytes(), hosted);
    drop(owner);
    assert_eq!(ledger.total(), baseline);
}
