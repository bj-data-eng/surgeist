use super::*;

fn limits(work: u64) -> DomLimits {
    DomLimits::new(
        128,
        4096,
        1024,
        0,
        0,
        0,
        0,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        DomDispatchCustodyLimits::new(64, 128, 4_000_000, 4_000_000, work).unwrap(),
    )
}

/// Independent dispatch-owner enumeration. Bank supplies its own structural
/// allocation facts; dispatch records and live row/release backing are walked.
pub(super) fn enumerated(bank: &CustodyBank) -> usize {
    let mut bytes = bank.scopes.index_charge().bytes + bank.targets.index_charge().bytes;
    for scope in bank.scopes.values() {
        bytes += size_of::<ScopeRecord>()
            + scope.targets.capacity() * size_of::<CleanupRow>()
            + size_of::<Scope>();
        if let Some(scope) = scope.scope.upgrade() {
            bytes += scope.rows.len() * size_of::<Row>();
            bytes += size_of::<Mutex<Ledger>>();
            bytes += scope.release.get().unwrap().node.len() * size_of::<ReleaseNode>();
        }
    }
    for bucket in bank.targets.values() {
        bytes += size_of::<TargetBucket>() + enumerated_generations(&bucket.generations);
    }
    if let Some(ledger) = &bank.ledger {
        bytes += size_of::<Mutex<Ledger>>();
        let ledger = ledger.lock().unwrap();
        let mut released = ledger.released.as_deref();
        while let Some(node) = released {
            bytes += std::mem::size_of_val(node);
            released = node[0].next.as_deref();
        }
    }
    bytes
}

fn enumerated_generations(generations: &bank::Bank<Generation>) -> usize {
    let mut bytes = generations.index_charge().bytes;
    for generation in generations.values() {
        bytes += size_of::<Generation>();
        match generation {
            Generation::Branch(children) => bytes += enumerated_generations(children),
            Generation::Rows(rows) => {
                bytes += rows.index_charge().bytes;
                for _ in rows.values() {
                    bytes += size_of::<IndexedRow>();
                }
            }
        }
    }
    bytes
}

fn fixture() -> (Dom, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "target");
    let commit = dom.apply(batch, &cancel).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&target).unwrap().unwrap(),
    )
}

#[test]
fn unchanged_custody_extent_admits_a_fixed_budget_with_many_live_scopes() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "target");
    let commit = dom.apply(batch, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scopes: Vec<_> = (0..64)
        .map(|_| dom.register_dispatch_custody(&[target], &cancel).unwrap())
        .collect();
    let retained = dom.dispatch_custody_retained_bytes(&cancel).unwrap();

    // No scope or row changed: an aggregate read and an unchanged limit check
    // have fixed work, independent of these sixty-four live registrations.
    assert_eq!(dom.set_limits(limits(4)), Ok(()));
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(retained));
    assert_eq!(scopes.len(), 64);
}

#[test]
fn scope_drop_debits_rows_before_any_prune_but_keeps_indexed_headers() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "target");
    let commit = dom.apply(batch, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scope = dom
        .register_dispatch_custody(&[target, target, target], &cancel)
        .unwrap();
    let before = dom.dispatch_custody_retained_bytes(&cancel).unwrap();
    std::thread::spawn(move || drop(scope)).join().unwrap();

    // Destruction releases the boxed rows and the scope's weak-ledger backing
    // occurrence. Reading does not prune its Scope header or cleanup index.
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(before - 3 * size_of::<Row>() - size_of::<Mutex<Ledger>>())
    );
    assert_eq!(
        enumerated(&dom.dispatch_custody),
        before - 3 * size_of::<Row>() - size_of::<Mutex<Ledger>>()
    );
}

#[test]
fn partial_prune_refusal_keeps_original_roots_and_release_queue_for_retry() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let abandoned = dom
        .register_dispatch_custody(&[target, target, target], &cancel)
        .unwrap();
    let survivor = dom.register_dispatch_custody(&[target], &cancel).unwrap();
    drop(abandoned);
    let before = enumerated(&dom.dispatch_custody);
    let scopes = dom.dispatch_custody.scopes.clone();
    let targets = dom.dispatch_custody.targets.clone();

    // Refuse after private row backing was prepared, before sealing cleanup.
    let processing = Processing::new(limits(5).dispatch_custody, None, &cancel);
    assert_eq!(
        dom.dispatch_custody.prune(&processing),
        Err(DomError::DispatchCustodyWorkLimit { limit: 5 })
    );
    assert!(scopes.shares_root(&dom.dispatch_custody.scopes));
    assert!(targets.shares_root(&dom.dispatch_custody.targets));
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(before));
    assert_eq!(enumerated(&dom.dispatch_custody), before);
    assert!(
        dom.dispatch_custody
            .ledger
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .released
            .is_some()
    );

    dom.set_limits(limits(100_000)).unwrap();
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
    assert_eq!(dom.dispatch_custody.scopes.record_count(), 1);
    assert_eq!(
        dom.dispatch_custody_association(&survivor, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
    let replacement = dom
        .register_dispatch_custody(&[target, target], &cancel)
        .unwrap();
    assert_eq!(replacement.targets(), 2);
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
}

#[test]
fn target_lookup_and_retirement_ignore_unrelated_scopes() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, _) = fixture();
    let mut batch = DomBatch::new();
    let targets: Vec<_> = (0..32)
        .map(|_| batch.create_comment(document.node(), "unrelated"))
        .collect();
    let commit = dom.apply(batch, &cancel).unwrap();
    let targets: Vec<_> = targets
        .iter()
        .map(|target| commit.created(target).unwrap().unwrap())
        .collect();
    let scopes: Vec<_> = targets
        .iter()
        .map(|target| dom.register_dispatch_custody(&[*target], &cancel).unwrap())
        .collect();
    let target = *targets.last().unwrap();
    let mut steps = 0;
    assert!(
        dom.dispatch_custody
            .tracks(target, || {
                steps += 1;
                Ok(())
            })
            .unwrap()
    );
    assert_eq!(
        steps,
        3 + LAST_GENERATION_PART,
        "slot/generation lookups and one actual participant"
    );

    dom.set_limits(limits(16)).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &cancel).unwrap();
    for (index, (scope, target)) in scopes.iter().zip(targets).enumerate() {
        let association = dom.dispatch_custody_association(scope, 0, target).unwrap();
        if index == 31 {
            assert!(matches!(association, DomDispatchAssociation::Retired(_)));
        } else {
            assert_eq!(association, DomDispatchAssociation::Current(document));
        }
    }
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
}

#[test]
fn same_slot_generations_keep_their_original_retirement_cells() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let original = dom.register_dispatch_custody(&[target], &cancel).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &cancel).unwrap();
    let old = dom
        .dispatch_custody_association(&original, 0, target)
        .unwrap();
    let mut batch = DomBatch::new();
    let fresh = batch.create_comment(document.node(), "fresh generation");
    let commit = dom.apply(batch, &cancel).unwrap();
    let fresh = commit.created(&fresh).unwrap().unwrap();
    assert_eq!(fresh.slot, target.slot);
    assert_ne!(fresh, target);
    let current = dom
        .register_dispatch_custody(&[fresh, fresh], &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(fresh);
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        dom.dispatch_custody_association(&original, 0, target),
        Ok(old)
    );
    let first = dom
        .dispatch_custody_association(&current, 0, fresh)
        .unwrap();
    assert!(matches!(first, DomDispatchAssociation::Retired(_)));
    assert_ne!(first, old);
    assert_eq!(
        dom.dispatch_custody_association(&current, 1, fresh),
        Ok(first)
    );
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
}

#[test]
fn late_registration_refusal_preserves_extent_and_can_retry() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let original = dom.register_dispatch_custody(&[target], &cancel).unwrap();
    let before = enumerated(&dom.dispatch_custody);
    let scopes = dom.dispatch_custody.scopes.clone();
    let targets = dom.dispatch_custody.targets.clone();
    let processing = Processing::new(limits(12).dispatch_custody, None, &cancel);
    assert!(matches!(
        dom.dispatch_custody
            .register(dom.owner, &dom.state, &[target, target], &processing),
        Err(DomError::DispatchCustodyWorkLimit { .. })
    ));
    assert!(scopes.shares_root(&dom.dispatch_custody.scopes));
    assert!(targets.shares_root(&dom.dispatch_custody.targets));
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(before));
    assert_eq!(enumerated(&dom.dispatch_custody), before);
    let retry = dom
        .register_dispatch_custody(&[target, target], &cancel)
        .unwrap();
    assert_eq!(retry.targets(), 2);
    assert_eq!(
        dom.dispatch_custody_association(&original, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
}

#[test]
fn owner_loss_discards_queued_cleanup_and_keeps_remaining_affine_scope_valid() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, target) = fixture();
    let scopes: Vec<_> = (0..64)
        .map(|_| dom.register_dispatch_custody(&[target], &cancel).unwrap())
        .collect();
    let mut scopes = scopes.into_iter();
    let last = scopes.next().unwrap();
    drop(scopes);
    drop(dom);
    assert_eq!(last.targets(), 1);
    assert_eq!(
        last.retained_bytes(),
        DomDispatchCustody::maximum_retained_bytes(1).unwrap()
    );
    std::thread::spawn(move || drop(last)).join().unwrap();
}

#[test]
fn exact_retained_limit_admits_and_one_short_refuses_without_changing_custody() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let scope = dom
        .register_dispatch_custody(&[target, target], &cancel)
        .unwrap();
    let actual = enumerated(&dom.dispatch_custody);
    let admitted = |bytes| {
        limits(100_000).with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 128, bytes, 4_000_000, 100_000).unwrap(),
        )
    };
    assert_eq!(
        dom.set_limits(admitted(actual - 1)),
        Err(DomError::ResourceLimit {
            resource: DomResource::DispatchCustodyRetainedBytes,
            limit: actual - 1,
        })
    );
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(actual));
    assert_eq!(dom.set_limits(admitted(actual)), Ok(()));
    assert_eq!(
        dom.dispatch_custody_association(&scope, 1, target),
        Ok(DomDispatchAssociation::Current(document))
    );
}

#[test]
fn foreign_target_validation_precedes_registration_backing_admission() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut foreign = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = foreign.apply(batch, &cancel).unwrap();
    let target = commit.document(&document).unwrap().node();
    dom.set_limits(
        limits(2).with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 128, 0, 4_000_000, 2).unwrap(),
        ),
    )
    .unwrap();
    assert_eq!(
        dom.register_dispatch_custody(&[target], &cancel)
            .unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(0));
}

#[test]
fn current_generation_retirement_fits_budget_despite_retained_historical_generations() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let mut historical = Vec::new();
    let mut previous: Option<NodeId> = None;
    for _ in 0..32 {
        let mut batch = DomBatch::new();
        let target = batch.create_comment(document.node(), "retained generation");
        let commit = dom.apply(batch, &cancel).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        if let Some(previous) = previous {
            assert_eq!(target.slot, previous.slot, "exercise the same reused slot");
        }
        previous = Some(target);
        let scope = dom.register_dispatch_custody(&[target], &cancel).unwrap();
        let mut batch = DomBatch::new();
        batch.destroy(target);
        dom.apply(batch, &cancel).unwrap();
        let association = dom.dispatch_custody_association(&scope, 0, target).unwrap();
        assert!(matches!(association, DomDispatchAssociation::Retired(_)));
        historical.push((target, scope, association));
    }
    let mut batch = DomBatch::new();
    let current = batch.create_comment(document.node(), "current generation");
    let commit = dom.apply(batch, &cancel).unwrap();
    let current = commit.created(&current).unwrap().unwrap();
    assert_eq!(current.slot, previous.unwrap().slot);
    let scope = dom.register_dispatch_custody(&[current], &cancel).unwrap();

    // Even the original aggregate limit check fits this budget. Retirement
    // itself has one participating row; old generations keep their own cells.
    dom.set_limits(limits(66)).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(current);
    dom.apply(batch, &cancel).unwrap();
    let association = dom
        .dispatch_custody_association(&scope, 0, current)
        .unwrap();
    assert!(matches!(association, DomDispatchAssociation::Retired(_)));
    assert_eq!(association.document(), document);
    for (target, scope, original) in historical {
        assert_eq!(
            dom.dispatch_custody_association(&scope, 0, target),
            Ok(original)
        );
    }
}

#[test]
fn many_distinct_rows_for_one_retirement_fit_nonquadratic_work_budget() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "many invocation rows");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scope = dom
        .register_dispatch_custody(&[target; 64], &cancel)
        .unwrap();
    dom.set_limits(limits(512)).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &cancel).unwrap();
    let first = dom.dispatch_custody_association(&scope, 0, target).unwrap();
    assert!(matches!(first, DomDispatchAssociation::Retired(_)));
    assert_eq!(first.document(), document);
    for row in 1..64 {
        assert_eq!(
            dom.dispatch_custody_association(&scope, row, target),
            Ok(first)
        );
    }
}

#[test]
fn source_work_refusal_keeps_cleanup_and_releases_temporary_debit_without_refunding_work() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, target) = fixture();
    let abandoned = dom
        .register_dispatch_custody(&[target, target, target], &cancel)
        .unwrap();
    let _survivor = dom.register_dispatch_custody(&[target], &cancel).unwrap();
    drop(abandoned);
    let actual = enumerated(&dom.dispatch_custody);
    let scopes = dom.dispatch_custody.scopes.clone();
    let targets = dom.dispatch_custody.targets.clone();
    let mut draft = Draft::new(&dom, Arc::new(crate::batch::BatchIdentity), &cancel).unwrap();
    let source_limits =
        |work| crate::DomSourceLimits::new(4_000_000, 4_000_000, work, 16, 4096, 4096, 1);
    draft.source_limits = Some(source_limits(128));
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    while draft.source_work.load(Ordering::Relaxed) < 123 {
        formation.step(&cancel).unwrap();
    }
    let processing = Processing::new(limits(100_000).dispatch_custody, Some(&formation), &cancel);
    assert_eq!(
        dom.dispatch_custody.prune(&processing),
        Err(DomError::SourceWorkLimit { limit: 128 })
    );
    assert!(scopes.shares_root(&dom.dispatch_custody.scopes));
    assert!(targets.shares_root(&dom.dispatch_custody.targets));
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(actual));
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), 128);
    assert_eq!(
        dom.dispatch_custody.prune(&processing),
        Err(DomError::SourceWorkLimit { limit: 128 })
    );
    drop(formation);

    draft.source_limits = Some(source_limits(1024));
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let processing = Processing::new(limits(100_000).dispatch_custody, Some(&formation), &cancel);
    dom.dispatch_custody.prune(&processing).unwrap();
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(enumerated(&dom.dispatch_custody))
    );
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert!(draft.source_work.load(Ordering::Relaxed) > 128);
}

#[test]
fn duplicate_retirement_packet_refuses_without_cells_and_real_retirement_can_retry() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let scope = dom
        .register_dispatch_custody(&[target, target], &cancel)
        .unwrap();
    let retirements = [
        Retirement { target, document },
        Retirement { target, document },
    ];
    let stamp = DomStamp {
        owner: dom.owner,
        revision: dom.revision,
    };
    assert!(matches!(
        dom.dispatch_custody.prepare_publication(
            &retirements,
            stamp,
            limits(100_000).dispatch_custody,
            None,
            &cancel,
        ),
        Err(DomError::InvalidDispatchCustody)
    ));
    for row in scope.scope.rows.iter() {
        assert!(row.retired.get().is_none());
    }
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &cancel).unwrap();
    let retired = dom.dispatch_custody_association(&scope, 0, target).unwrap();
    assert!(matches!(retired, DomDispatchAssociation::Retired(_)));
    assert_eq!(
        dom.dispatch_custody_association(&scope, 1, target),
        Ok(retired)
    );
}

#[test]
fn generation_index_distinguishes_high_bits_and_maximum_generation() {
    let cancel = AtomicBool::new(false);
    let (dom, _, _) = fixture();
    let processing = Processing::new(limits(100_000).dispatch_custody, None, &cancel);
    let mut allocation = Allocation::new(&processing, 0);
    let mut bucket = TargetBucket {
        generations: bank::Bank::default(),
    };
    let original = bank::Bank::default();
    let generations = [1, (1_u64 << 32) | 1, (1_u64 << 63) | 1, u64::MAX];
    for (key, generation) in generations.into_iter().enumerate() {
        // These opaque identity values exercise the index primitive. They grant
        // no execution or canonical-node authority and are never registered.
        let target = NodeId {
            owner: dom.owner,
            slot: 17,
            generation,
        };
        let replaced = replaced_generation_bytes(
            &bucket.generations,
            &original,
            target,
            LAST_GENERATION_PART,
            key,
        )
        .unwrap();
        let row = allocation
            .record(IndexedRow {
                target,
                scope: 0,
                index: key,
            })
            .unwrap();
        edit_generation(
            &mut bucket.generations,
            target,
            LAST_GENERATION_PART,
            key,
            Some(row),
            &mut allocation,
        )
        .unwrap();
        allocation.release(replaced);
    }
    for (key, generation) in generations.into_iter().enumerate() {
        let target = NodeId {
            owner: dom.owner,
            slot: 17,
            generation,
        };
        let rows = bucket.rows(target, || Ok(())).unwrap().unwrap();
        assert_eq!(rows.record_count(), 1);
        assert_eq!(rows.get(key).unwrap().target, target);
    }
    assert!(
        bucket
            .rows(
                NodeId {
                    owner: dom.owner,
                    slot: 17,
                    generation: 2
                },
                || Ok(())
            )
            .unwrap()
            .is_none()
    );
    assert_eq!(allocation.held, enumerated_generations(&bucket.generations));
}

#[test]
fn actual_temporary_backing_capacity_is_measured_without_charging_empty_vectors() {
    if !scale_metrics::ENABLED {
        return;
    }
    let cancel = AtomicBool::new(false);
    let processing = Processing::new(limits(100_000).dispatch_custody, None, &cancel);
    let mut allocation = Allocation::new(&processing, 0);
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let values = reserve::<u64>(5, &mut allocation).unwrap();
    let counters = measurement.finish(scale_metrics::Hooks::default()).unwrap();
    assert!(values.capacity() >= 5);
    assert_eq!(counters.structural_allocations, 1);
    assert_eq!(
        counters.structural_bytes,
        values.capacity() * size_of::<u64>()
    );
    assert_eq!(counters.sequence_materialized_bytes, 0);
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let empty = reserve::<u64>(0, &mut allocation).unwrap();
    let counters = measurement.finish(scale_metrics::Hooks::default()).unwrap();
    assert_eq!(empty.capacity(), 0);
    assert_eq!(counters.structural_allocations, 0);
    assert_eq!(counters.sequence_materialized_bytes, 0);
}

#[test]
fn actual_registration_and_retirement_plan_measure_initialized_rows() {
    if !scale_metrics::ENABLED {
        return;
    }
    let cancel = AtomicBool::new(false);
    let (mut dom, document, target) = fixture();
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let scope = dom
        .register_dispatch_custody(&[target; 3], &cancel)
        .unwrap();
    let counters = measurement
        .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Bank))
        .unwrap();
    let initialized = 3 * (size_of::<CleanupRow>() + size_of::<Row>()) + size_of::<ReleaseNode>();
    // Boxing can relocate either exact initialized slice when an allocator
    // supplied spare capacity. Reusing the Vec backing contributes no copy.
    assert!(counters.sequence_materialized_bytes >= initialized);
    assert!(
        counters.sequence_materialized_bytes
            <= initialized + 3 * size_of::<Row>() + size_of::<ReleaseNode>()
    );
    assert!(
        counters.structural_bytes >= initialized + size_of::<Mutex<Ledger>>() + size_of::<Scope>()
    );
    for row in 0..3 {
        assert_eq!(
            dom.dispatch_custody_association(&scope, row, target),
            Ok(DomDispatchAssociation::Current(document))
        );
    }
    let stamp = dom.snapshot().stamp();
    let admitted = dom.limits.dispatch_custody;
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let plan = dom
        .dispatch_custody
        .prepare_publication(
            &[Retirement { target, document }],
            stamp,
            admitted,
            None,
            &cancel,
        )
        .unwrap();
    let counters = measurement
        .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Bank))
        .unwrap();
    assert_eq!(plan.rows.len(), 3);
    assert_eq!(
        counters.sequence_materialized_bytes,
        3 * size_of::<PlannedRow>()
    );
    assert!(counters.structural_bytes >= plan.rows.capacity() * size_of::<PlannedRow>());
    assert!(
        plan.rows
            .iter()
            .enumerate()
            .all(|(index, row)| row.index == index)
    );
    drop(plan);
    for row in scope.scope.rows.iter() {
        assert!(
            row.retired.get().is_none(),
            "preparation publishes no retirement"
        );
    }
}
