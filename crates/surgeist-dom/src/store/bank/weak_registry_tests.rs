//! An actual borrowed materialization fits a finite registry allowance without
//! reserving an affine Debit field for every vacant weak-descriptor slot.
use super::*;
use crate::DomResource;
use std::mem::align_of;

struct BorrowedValue {
    identity: MaterializationId,
    words: [u64; 3],
}

fn one_object_allowance() -> usize {
    // An empty registry needs one leaf and the complete fixed radix path for
    // this first key. Enumerate the real target/header allocations independently
    // of the production receipt calculator. Opaque Arc/allocator bytes stay out.
    let branches = (u64::BITS / BITS - 1) as usize;
    let weak_descriptor = size_of::<u64>() + size_of::<WeakCustody>();
    [
        size_of::<BorrowedValue>(),
        size_of::<Debit>(), // shared payload receipt target
        size_of::<RecordCustody>(),
        size_of::<Mutex<RegistryState>>(), // release ticket's Weak owner target
        size_of::<RecordCustody>(),        // registry entry's Weak target allowance
        size_of::<ReleaseNode>(),
        (branches + 1) * size_of::<RegistryNode>(),
        branches * WIDTH * size_of::<Option<Box<RegistryNode>>>(),
        WIDTH * weak_descriptor, // nonce and Weak, including vacant positions
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
    let branches = (u64::BITS / BITS - 1) as usize;
    let state = registry.lock();
    assert!(state.pages.is_none());
    let mut node = state.records.as_deref().unwrap();
    for _ in 0..branches {
        let RegistryNode::Branch { children, .. } = node else {
            panic!("one-object registry path ended before its complete radix depth");
        };
        assert_eq!(children.len(), WIDTH);
        assert_eq!(children.capacity(), WIDTH, "actual branch reserve excess");
        let mut occupied = children.iter().flatten();
        node = occupied.next().unwrap();
        assert!(occupied.next().is_none());
    }
    let RegistryNode::Leaf { entries, .. } = node else {
        panic!("one-object registry path did not end in a leaf");
    };
    assert_eq!(entries.len(), WIDTH);
    assert_eq!(entries.capacity(), WIDTH, "actual leaf reserve excess");
    assert_eq!(entries.iter().flatten().count(), 1);
    let descriptor = size_of::<u64>() + size_of::<WeakCustody>();
    let alignment = align_of::<u64>().max(align_of::<WeakCustody>());
    assert_eq!(descriptor % alignment, 0, "descriptor alignment premise");
    eprintln!(
        "one-object registry inventory: branches={branches}, branch/leaf capacity={WIDTH}, \
         actual entry stride={}, nonce+Weak allowance={descriptor}, descriptor alignment={alignment}",
        size_of::<Option<RegistryEntry>>()
    );
    drop(state);
    drop(custody);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), before);
    assert_eq!(value.words, [11, 37, 97]);
}

#[test]
fn finite_weak_descriptor_allowance_adopts_aliases_and_releases_original_custody() {
    // This is genuine external immutable input, borrowed during adoption. Its
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
