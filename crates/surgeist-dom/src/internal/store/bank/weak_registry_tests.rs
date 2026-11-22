//! Exact weak custody and node backing for a borrowed materialization.
use super::*;
use crate::internal::store::DomResource;

struct BorrowedValue {
    identity: MaterializationId,
    words: [u64; 3],
}

fn one_object_allowance() -> usize {
    // Enumerate the first entry's allocations independently of the production
    // receipt calculator. Opaque Arc and allocator headers stay out.
    [
        size_of::<BorrowedValue>(),
        size_of::<Debit>(), // shared payload receipt target
        size_of::<RecordCustody>(),
        size_of::<Mutex<RegistryState>>(), // release ticket's Weak owner target
        size_of::<RecordCustody>(),        // registry entry's Weak target allowance
        size_of::<ReleaseNode>(),
        size_of::<RegistryNode>(),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .expect("bounded one-object allocation inventory")
}

fn qualify_registry_allocation_inventory(value: &BorrowedValue, payload: Usage) {
    let ledger = Ledger::default();
    let registry = CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let before = ledger.total();
    let custody = registry
        .adopt_object(value.identity, payload, |_| Ok(()))
        .unwrap();
    let state = registry.lock();
    assert!(state.pages.is_none());
    let node = state.records.as_deref().unwrap();
    assert_eq!(node.identity, value.identity);
    assert!(node.children.iter().all(Option::is_none));
    assert!(
        matches!(&node.entry.custody, WeakCustody::Record(weak) if weak.ptr_eq(&Arc::downgrade(&custody.0)))
    );
    drop(state);
    drop(custody);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), before);
    assert_eq!(value.words, [11, 37, 97]);
}

#[test]
fn finite_weak_descriptor_allowance_adopts_aliases_and_releases_original_custody() {
    // This is external immutable input, borrowed during adoption. Its
    // original identity is issued before its target allocation and never reset.
    let identity = MaterializationId::fresh().unwrap();
    let value = Arc::new(BorrowedValue {
        identity,
        words: [11, 37, 97],
    });
    let payload = Usage::new(size_of::<BorrowedValue>(), 0);
    // Qualify real allocation capacities independently, without pricing the
    // finite allowance from an observed ledger peak or requiring a new layout.
    qualify_registry_allocation_inventory(&value, payload);
    let ledger = Ledger::default();
    let registry = CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let before = ledger.total();
    let exact = before.bytes().checked_add(one_object_allowance()).unwrap();
    let admit = |limit| {
        move |usage: Usage| {
            super::super::admit(usage.bytes(), limit, DomResource::SourceRetainedBytes)
        }
    };

    let refused = registry.adopt_object(value.identity, payload, admit(exact - 1));
    let error = match refused {
        Err(error) => error,
        Ok(_) => panic!("one-short first-object allowance admitted a custodian"),
    };
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: exact - 1,
        }
    );
    registry.reconcile_releases();
    assert_eq!(ledger.total(), before);
    assert_eq!(value.identity, identity);
    assert_eq!(value.words, [11, 37, 97]);

    let original = registry
        .adopt_object(value.identity, payload, admit(exact))
        .unwrap_or_else(|error| {
            panic!("complete one-object weak-descriptor allowance {exact} refused: {error:?}")
        });
    let retained = ledger.total();
    assert_eq!(retained.bytes(), exact);
    assert_eq!(retained.units(), 0);
    assert_eq!(original.0.identity, identity);
    assert_eq!(value.words, [11, 37, 97]);
    let alias = registry
        .adopt_object(value.identity, payload, admit(exact))
        .unwrap();
    assert!(Arc::ptr_eq(&original.0, &alias.0));
    assert_eq!(ledger.total(), retained);
    drop(original);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), retained);
    assert_eq!(value.words, [11, 37, 97]);
    drop(alias);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), before);
    assert_eq!(value.identity, identity);
    assert_eq!(value.words, [11, 37, 97]);
}
