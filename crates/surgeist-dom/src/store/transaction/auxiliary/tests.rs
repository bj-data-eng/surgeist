//! Actual raw Bank materializations exercise the consumed-output guard.
use super::*;
use crate::hosted::{DomHostedLimits, DomHostedRecordLimits, DomHostedSource, DomHostedSourceId};
use std::{collections::BTreeMap, sync::Mutex};

struct Row {
    origin: MaterializationId,
    value: u64,
}
struct Sample {
    identity: MaterializationId,
    rows: bank::Bank<Row>,
    ledger: custody::Ledger,
    dropped: Arc<Mutex<Vec<Usage>>>,
}
impl Drop for Sample {
    fn drop(&mut self) {
        self.dropped.lock().unwrap().push(self.ledger.total());
    }
}
struct Built {
    semantic: Arc<Sample>,
    incoming: Debit,
}
struct Children {
    rows: bank::BankCustody<Row>,
}
fn build(
    registry: &bank::CustodyRegistry,
    old: Option<&Sample>,
    index: usize,
    leftover: usize,
    dropped: &Arc<Mutex<Vec<Usage>>>,
) -> Built {
    let origin = MaterializationId::fresh().unwrap();
    let reserved = bank::Record::reserve().unwrap();
    let mut incoming = registry
        .ledger()
        .retain(
            Usage::new(size_of::<Sample>() + size_of::<Row>() + leftover, 0),
            |_| Ok(()),
        )
        .unwrap();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let source = DomHostedSource::new(
        DomHostedSourceId {
            owner: crate::store::DomOwnerId::allocate().unwrap(),
            serial: 0,
        },
        DomHostedLimits::new(
            DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 0),
            16_000_000,
            16_000_000,
            10_000_000,
        ),
        &cancel,
    )
    .unwrap();
    let mut rows = old.map_or_else(bank::Bank::default, |old| old.rows.clone());
    rows.set_hosted_admitted(
        index,
        reserved.materialize(Row {
            origin,
            value: index as u64,
        }),
        false,
        bank::Totals {
            bytes: size_of::<Row>(),
            units: 0,
            occurrences: 1,
        },
        origin,
        |charge| {
            incoming.grow(Usage::new(charge.bytes, 0), |_| Ok(()))?;
            source.retain(charge.bytes, &cancel)
        },
    )
    .unwrap();
    Built {
        semantic: Arc::new(Sample {
            identity: origin,
            rows,
            ledger: registry.ledger().clone(),
            dropped: dropped.clone(),
        }),
        incoming,
    }
}
fn capture(
    adopter: &Adopter<'_, '_, '_>,
    sample: &Sample,
) -> Result<bank::Attached<Children>, DomError> {
    let root = adopter.root(sample.identity, Usage::new(size_of::<Sample>(), 0))?;
    if let Some(existing) = root.attachment::<Children>() {
        return Ok(existing);
    }
    let rows = adopter.rows(&sample.rows, &|row| {
        Ok((row.origin, Usage::new(size_of::<Row>(), 0)))
    })?;
    adopter.attach(root, Children { rows })
}
fn prepare(
    built: Built,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<Prepared<Sample, bank::Attached<Children>>, DomError> {
    let candidate = Candidate {
        adopter: Adopter::new(
            Some(built.incoming),
            built.semantic.identity,
            registry,
            admit,
            step,
        ),
        semantic: built.semantic,
    };
    let custody = capture(&candidate.adopter, &candidate.semantic)?;
    candidate.adopter.finish()?;
    Ok(Prepared {
        semantic: candidate.semantic,
        custody,
    })
}
fn registry() -> bank::CustodyRegistry {
    bank::CustodyRegistry::new(&custody::Ledger::default(), |_| Ok(())).unwrap()
}
fn raw(sample: &Sample) -> BTreeMap<MaterializationId, Usage> {
    let mut expected = BTreeMap::from([(sample.identity, Usage::new(size_of::<Sample>(), 0))]);
    let physical = sample
        .rows
        .inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Row>(),
                units: 0,
                occurrences: 1,
            })
        })
        .unwrap();
    expected.extend(physical.materializations());
    expected
}
fn claimed(custody: &bank::Attached<Children>) -> BTreeMap<MaterializationId, Usage> {
    let mut actual = BTreeMap::from([custody.inspect_owner_payload()]);
    actual.extend(custody.rows.inspect_materializations());
    actual
}

#[test]
fn fresh_output_transfers_exact_components_and_external_aliases_keep_no_dom_custody() {
    let registry = registry();
    let dropped = Arc::new(Mutex::new(Vec::new()));
    let built = build(&registry, None, 17, 0, &dropped);
    let original = built.incoming.usage();
    let (sample, control) = prepare(built, &registry, &mut |_| Ok(()), &mut || Ok(()))
        .unwrap()
        .into_parts();
    assert_eq!(claimed(&control), raw(&sample));
    let alias = control.clone();
    let before = registry.total();
    drop(control);
    registry.reconcile_releases();
    assert_eq!(registry.total(), before);
    drop(alias);
    registry.reconcile_releases();
    assert_eq!(
        registry.total(),
        bank::CustodyRegistry::administration_usage()
    );
    assert_eq!(sample.rows.get(17).unwrap().value, 17);
    assert!(dropped.lock().unwrap().is_empty());
    drop(sample);
    assert_eq!(dropped.lock().unwrap().len(), 1);
    assert!(original.bytes() > size_of::<Sample>());
}

#[test]
fn each_reached_admission_refusal_drops_data_before_its_original_pool() {
    // Six materializations (root, record and four sparse path pages) each
    // need at most sixteen registry levels plus fixed control/staging work.
    // Fresh IDs can change prefix sharing, so use each execution's actual
    // reached cutoff rather than reusing another execution's callback count.
    for refused in 0..512 {
        let registry = registry();
        let dropped = Arc::new(Mutex::new(Vec::new()));
        let built = build(&registry, None, 4096, 0, &dropped);
        let original = built.incoming.usage();
        let mut reached = 0;
        let result = prepare(
            built,
            &registry,
            &mut |_| {
                let current = reached;
                reached += 1;
                if current == refused {
                    Err(DomError::RepresentationExhausted)
                } else {
                    Ok(())
                }
            },
            &mut || Ok(()),
        );
        let complete = result.is_ok();
        if complete {
            assert!(reached <= refused);
            assert!(refused > 10);
        } else {
            assert!(
                matches!(result, Err(DomError::RepresentationExhausted)),
                "admission {refused}"
            );
            assert_eq!(reached, refused + 1);
        }
        drop(result);
        let observed = dropped.lock().unwrap();
        assert_eq!(observed.len(), 1);
        assert!(
            observed[0].bytes() >= original.bytes(),
            "admission {refused}: {:?} < {original:?}",
            observed[0]
        );
        drop(observed);
        registry.reconcile_releases();
        assert_eq!(
            registry.total(),
            bank::CustodyRegistry::administration_usage(),
            "admission {refused}"
        );
        if complete {
            return;
        }
    }
    panic!("all admission cutoffs must finish within the finite graph bound");
}

#[test]
fn nonzero_producer_leftover_refuses_without_releasing_data_early() {
    let registry = registry();
    let dropped = Arc::new(Mutex::new(Vec::new()));
    let built = build(&registry, None, 0, 1, &dropped);
    let original = built.incoming.usage();
    assert!(matches!(
        prepare(built, &registry, &mut |_| Ok(()), &mut || Ok(())),
        Err(DomError::RepresentationExhausted)
    ));
    assert!(dropped.lock().unwrap()[0].bytes() >= original.bytes());
    registry.reconcile_releases();
    assert_eq!(
        registry.total(),
        bank::CustodyRegistry::administration_usage()
    );
}

#[test]
fn historical_frontier_adoption_prunes_active_rows_after_one_changed_path() {
    let registry = registry();
    let dropped = Arc::new(Mutex::new(Vec::new()));
    let (mut sample, mut control) = prepare(
        build(&registry, None, 0, 0, &dropped),
        &registry,
        &mut |_| Ok(()),
        &mut || Ok(()),
    )
    .unwrap()
    .into_parts();
    for index in 1..128 {
        let next = prepare(
            build(&registry, Some(&sample), index, 0, &dropped),
            &registry,
            &mut |_| Ok(()),
            &mut || Ok(()),
        )
        .unwrap()
        .into_parts();
        (sample, control) = next;
    }
    registry.reconcile_releases();
    let before = registry.total();
    let mut visited = 0;
    let alias = capture(
        &Adopter::new(
            None,
            sample.identity,
            &registry,
            &mut |_| Ok(()),
            &mut || {
                visited += 1;
                Ok(())
            },
        ),
        &sample,
    )
    .unwrap();
    assert_eq!(visited, 0);
    assert_eq!(registry.total(), before);
    drop(alias);
    let fresh = build(&registry, Some(&sample), 17, 0, &dropped);
    // Release the controlled formed holder while an external snapshot keeps
    // its immutable graph. Re-adoption borrows that external graph normally.
    let external = fresh.semantic.clone();
    drop(fresh);
    let mut visited = 0;
    let next_control = capture(
        &Adopter::new(
            None,
            external.identity,
            &registry,
            &mut |_| Ok(()),
            &mut || {
                visited += 1;
                Ok(())
            },
        ),
        &external,
    )
    .unwrap();
    assert_eq!(visited, 1);
    assert_eq!(claimed(&next_control), raw(&external));
    assert_eq!(sample.rows.get(127).unwrap().value, 127);
    drop((next_control, external, control, sample));
    registry.reconcile_releases();
    assert_eq!(
        registry.total(),
        bank::CustodyRegistry::administration_usage()
    );
}

fn wide_controls<T: Send + Sync + 'static>(
    custody: &index::U64Custody<T>,
    controls: &mut bank::PhysicalControlGraph,
    bodies: &mut BTreeMap<usize, usize>,
) {
    controls.merge(custody.branches.inspect_control_graph());
    controls.merge(custody.values.inspect_control_graph());
    for branch in custody
        .branches
        .inspect_dependencies::<index::U64Children<T>>()
    {
        bodies.insert(
            std::ptr::from_ref(&*branch) as usize,
            size_of::<index::U64Children<T>>() + size_of::<Debit>(),
        );
        wide_controls(&branch._next, controls, bodies);
    }
}
fn index_controls<T: Send + Sync + 'static>(
    custody: &index::IndexCustody<T>,
    controls: &mut bank::PhysicalControlGraph,
    bodies: &mut BTreeMap<usize, usize>,
) {
    wide_controls(&custody.sources, controls, bodies);
    for source in custody
        .sources
        .inspect_dependencies::<index::IssuerCustody<T>>()
    {
        bodies.insert(
            std::ptr::from_ref(&*source) as usize,
            size_of::<index::IssuerCustody<T>>() + size_of::<Debit>(),
        );
        controls.merge(source._slots.inspect_control_graph());
        for slot in source
            ._slots
            .inspect_dependencies::<index::GenerationsCustody<T>>()
        {
            bodies.insert(
                std::ptr::from_ref(&*slot) as usize,
                size_of::<index::GenerationsCustody<T>>() + size_of::<Debit>(),
            );
            wide_controls(&slot._values, controls, bodies);
        }
    }
}
fn history_peak(
    semantic: &hosted::history::Graph,
    custody: &HistoryCustody,
    adopter: &Adopter<'_, '_, '_>,
    before: Usage,
    original_registry: Usage,
) -> Usage {
    // Expected side enumerates physical storage and concrete owner layouts.
    // It deliberately never reads a Debit, cached payload/ledger total, or
    // recorded prospective admission to derive the new occurrence's price.
    let mut controls = custody.0.inspect_control_graph();
    controls.merge(custody.0._navigables.inspect_control_graph());
    controls.merge(custody.0._entries.inspect_control_graph());
    let mut bodies = BTreeMap::new();
    index_controls(&custody.0._navigable_ids, &mut controls, &mut bodies);
    index_controls(&custody.0._entry_ids, &mut controls, &mut bodies);
    controls.merge(custody.0._documents._slots.inspect_control_graph());
    for slot in custody
        .0
        ._documents
        ._slots
        .inspect_dependencies::<index::GenerationsCustody<hosted::history::documents::Members>>()
    {
        bodies.insert(
            std::ptr::from_ref(&*slot) as usize,
            size_of::<index::GenerationsCustody<hosted::history::documents::Members>>()
                + size_of::<Debit>(),
        );
        wide_controls(&slot._values, &mut controls, &mut bodies);
        for members in slot
            ._values
            .inspect_dependencies::<DocumentMembersCustody>()
        {
            bodies.insert(
                std::ptr::from_ref(&*members) as usize,
                size_of::<DocumentMembersCustody>() + size_of::<Debit>(),
            );
            controls.merge(members._rows.inspect_control_graph());
        }
    }
    assert!(!bodies.is_empty(), "real nested index capsules are present");
    let semantic_usage = semantic
        .inspect_materializations()
        .unwrap()
        .into_iter()
        .try_fold(Usage::ZERO, |sum, (_, usage)| sum.checked_add(usage))
        .unwrap();
    let staging = adopter.pool.borrow().aliases.capacity() * size_of::<SharedDebit>();
    let registry_growth = adopter
        .registry
        .inspect_storage()
        .checked_sub(original_registry)
        .unwrap();
    before
        .checked_add(semantic_usage)
        .unwrap()
        .checked_add(controls.usage())
        .unwrap()
        .checked_add(registry_growth)
        .unwrap()
        .checked_add(Usage::new(
            size_of::<HistoryChildren>() + staging + bodies.values().sum::<usize>(),
            0,
        ))
        .unwrap()
}

#[test]
fn composed_history_transfer_peak_has_independent_exact_and_one_short_dom_boundary() {
    use crate::DomSourceLimits;
    use crate::store::{DomBatch, DomResource, Draft, source::formation::Formation};
    use std::sync::atomic::AtomicBool;
    let cancel = AtomicBool::new(false);
    let mut fixture = hosted::history::tests::Fixture::new();
    let (top, operation, work) = fixture.issue(0);
    let draft = Draft::new_hosted_source(
        &fixture.dom,
        DomBatch::new().owner,
        DomSourceLimits::new(128_000_000, 4096, 10_000_000, 0, 0, 0, 1),
        work.spent.clone(),
        work.spent_id,
        operation,
        &cancel,
    )
    .unwrap();
    let ledger = draft.custody_registry().ledger().clone();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let before = ledger.total();
    let original_registry = registry.inspect_storage();
    let context = hosted::history::Context::new(&fixture.source, &work, &cancel, 0)
        .with_formation(&formation);
    let semantic =
        hosted::history::Graph::initial(None, fixture.input(top, None), &context).unwrap();
    let incoming = formation.into_debit().unwrap();
    let payload = semantic
        .inspect_materializations()
        .unwrap()
        .into_iter()
        .try_fold(Usage::ZERO, |sum, (_, usage)| sum.checked_add(usage))
        .unwrap();
    assert_eq!(incoming.usage(), payload);
    let maximum = std::cell::Cell::new(Usage::ZERO);
    let mut observe = |next: Usage| {
        let old = maximum.get();
        maximum.set(Usage::new(
            old.bytes().max(next.bytes()),
            old.units().max(next.units()),
        ));
        Ok(())
    };
    let mut step = || Ok(());
    let origin = semantic.custody_origin();
    let (external, control, exact) = {
        let candidate = Candidate {
            semantic,
            adopter: Adopter::new(Some(incoming), origin, &registry, &mut observe, &mut step),
        };
        let control = candidate.adopter.history(&candidate.semantic).unwrap();
        candidate.adopter.finish().unwrap();
        let exact = history_peak(
            &candidate.semantic,
            &control,
            &candidate.adopter,
            before,
            original_registry,
        );
        assert_eq!(ledger.total(), exact);
        assert_eq!(
            maximum.get(),
            exact,
            "the actual reached staged peak equals independent physical storage"
        );
        (candidate.semantic, control, exact)
    };
    drop(control);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), before);
    for limit in [exact.bytes() - 1, exact.bytes()] {
        // These SAME immutable materializations survive only in the external
        // snapshot. Re-admit their independent payload before retaining the
        // consumed caller alias; registry path shape/IDs and issuer stay fixed.
        let incoming = ledger.retain(payload, |_| Ok(())).unwrap();
        let semantic = external.clone();
        let maximum = std::cell::Cell::new(0usize);
        let result = prepare_history(
            semantic,
            Some(incoming),
            &registry,
            &mut |next| {
                maximum.set(maximum.get().max(next.bytes()));
                if next.bytes() > limit {
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit,
                    })
                } else {
                    Ok(())
                }
            },
            &mut || Ok(()),
        );
        if limit < exact.bytes() {
            assert!(
                matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit: actual }) if actual == limit)
            );
            assert_eq!(maximum.get(), exact.bytes());
        } else {
            let (semantic, control) = result.unwrap().into_parts();
            assert_eq!(maximum.get(), exact.bytes());
            assert_eq!(
                control
                    .inspect_claimed_materializations()
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
                semantic
                    .inspect_materializations()
                    .unwrap()
                    .into_iter()
                    .collect()
            );
            assert_eq!(
                semantic.entry(top.entry).unwrap().document,
                Some(top.document)
            );
            drop((semantic, control));
        }
        registry.reconcile_releases();
        assert_eq!(ledger.total(), before);
        assert_eq!(
            external.entry(top.entry).unwrap().document,
            Some(top.document)
        );
    }
}

#[test]
fn alias_replacement_admits_both_backings_and_preserves_full_old_pool_on_refusal() {
    use crate::store::{DomBatch, Draft};
    use crate::{Dom, DomLimits, DomSourceLimits};
    use std::cell::{Cell, RefCell};
    use std::sync::atomic::{AtomicBool, Ordering};
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(
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
    let batch = DomBatch::new();
    let draft = RefCell::new(
        Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4),
            &cancel,
        )
        .unwrap()
        .0,
    );
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let origin = MaterializationId::fresh().unwrap();
    // A real admitted byte backing supplies the affine payload. The Vec target
    // precedes its receipt in Candidate, including on every refusal below.
    let mut incoming = ledger
        .retain(Usage::new(size_of::<Vec<u8>>() + 4096, 0), |_| Ok(()))
        .unwrap();
    let mut data = Vec::new();
    data.try_reserve_exact(4096).unwrap();
    incoming
        .grow(Usage::new(data.capacity() - 4096, 0), |_| Ok(()))
        .unwrap();
    data.resize(4096, 0u8);
    let payload = size_of::<Vec<u8>>() + data.capacity();
    let limit = Cell::new(usize::MAX);
    let maximum = Cell::new(0);
    let mut admit = |next: Usage| {
        maximum.set(maximum.get().max(next.bytes()));
        if next.bytes() > limit.get() {
            Err(DomError::ResourceLimit {
                resource: crate::DomResource::SourceRetainedBytes,
                limit: limit.get(),
            })
        } else {
            Ok(())
        }
    };
    let mut step = || draft.borrow().source_step(&cancel);
    let candidate = Candidate {
        semantic: Arc::new(data),
        adopter: Adopter::new(Some(incoming), origin, &registry, &mut admit, &mut step),
    };
    candidate
        .adopter
        .retained(Some(origin), Usage::new(1, 0))
        .unwrap();
    let capacity = candidate.adopter.pool.borrow().aliases.capacity();
    assert!(capacity >= 4);
    for _ in 1..capacity {
        candidate
            .adopter
            .retained(Some(origin), Usage::new(1, 0))
            .unwrap();
    }
    let before = registry.inspect_storage().bytes()
        + payload
        + capacity * size_of::<Debit>()
        + capacity * size_of::<SharedDebit>();
    assert_eq!(ledger.total(), Usage::new(before, 0));
    let target = capacity * 2;
    // Independent raw layout witness for this exact replacement element type.
    // It never reads a Debit, cached usage, or observed prospective admission.
    let mut witness = Vec::<SharedDebit>::new();
    witness.try_reserve_exact(target).unwrap();
    let replacement_bytes = witness.capacity() * size_of::<SharedDebit>();
    drop(witness);
    let exact = before + replacement_bytes;
    maximum.set(0);
    limit.set(exact - 1);
    let result = candidate.adopter.retained(Some(origin), Usage::new(1, 0));
    assert!(
        matches!(
            &result,
            Err(DomError::ResourceLimit {
                resource: crate::DomResource::SourceRetainedBytes,
                limit: actual,
            }) if *actual == exact - 1
        ),
        "one-short replacement result: {:?}",
        result.as_ref().map(|value| value.is_some())
    );
    {
        let pool = candidate.adopter.pool.borrow();
        assert_eq!(pool.aliases.len(), capacity);
        assert_eq!(pool.aliases.capacity(), capacity);
        assert_eq!(
            pool.incoming.as_ref().unwrap().usage().bytes(),
            payload - capacity
        );
        assert!(pool.pending.is_none());
        assert!(pool.shared.is_none());
    }
    assert_eq!(ledger.total(), Usage::new(before, 0));
    assert_eq!(maximum.get(), exact);
    limit.set(exact);
    // Use the real Source tariff, with one successful step before refusal.
    // No alias has moved when prepayment fails, and spent work is not refunded.
    let work_before = draft.borrow().source_work.load(Ordering::Relaxed);
    let work_limit = work_before + 1;
    draft.borrow_mut().source_limits.as_mut().unwrap().work = work_limit;
    let old_aliases = candidate.adopter.pool.borrow().aliases.as_ptr();
    let result = candidate.adopter.retained(Some(origin), Usage::new(1, 0));
    assert!(
        matches!(
            &result,
            Err(DomError::SourceWorkLimit { limit }) if *limit == work_limit
        ),
        "replacement work refusal: {:?}",
        result.as_ref().map(|value| value.is_some())
    );
    assert_eq!(
        draft.borrow().source_work.load(Ordering::Relaxed),
        work_limit
    );
    {
        let pool = candidate.adopter.pool.borrow();
        assert_eq!(pool.aliases.as_ptr(), old_aliases);
        assert_eq!(pool.aliases.len(), capacity);
        assert_eq!(pool.aliases.capacity(), capacity);
        assert_eq!(
            pool.incoming.as_ref().unwrap().usage().bytes(),
            payload - capacity
        );
        assert!(pool.pending.is_none());
        assert!(pool.shared.is_none());
    }
    assert_eq!(ledger.total(), Usage::new(before, 0));
    assert_eq!(candidate.semantic.len(), 4096);
    assert!(candidate.semantic.iter().all(|byte| *byte == 0));
    // The retry must prepay the full unchanged pool, including the first move
    // charged by the refused attempt. The exact remaining budget admits it.
    let retry_limit = work_limit + u64::try_from(capacity).unwrap();
    draft.borrow_mut().source_limits.as_mut().unwrap().work = retry_limit;
    candidate
        .adopter
        .retained(Some(origin), Usage::new(1, 0))
        .unwrap();
    assert_eq!(
        draft.borrow().source_work.load(Ordering::Relaxed),
        retry_limit
    );
    let actual_capacity = candidate.adopter.pool.borrow().aliases.capacity();
    assert_eq!(
        actual_capacity * size_of::<SharedDebit>(),
        replacement_bytes
    );
    assert_eq!(maximum.get(), exact);
    assert_eq!(
        ledger.total(),
        Usage::new(
            registry.inspect_storage().bytes()
                + payload
                + (capacity + 1) * size_of::<Debit>()
                + actual_capacity * size_of::<SharedDebit>(),
            0,
        )
    );
    assert_eq!(candidate.semantic.len(), 4096);
    assert!(candidate.semantic.iter().all(|byte| *byte == 0));
    drop(candidate);
    assert_eq!(ledger.total(), registry.inspect_storage());
    drop(registry);
    assert_eq!(ledger.total(), Usage::ZERO);
}
