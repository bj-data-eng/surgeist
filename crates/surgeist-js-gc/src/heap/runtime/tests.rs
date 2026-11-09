use super::*;
use crate::heap::{EdgeVisitor, ObjectId};
use std::{cell::Cell, rc::Weak};

struct Record(RefCell<Option<ObjectId>>);

impl Trace for Record {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        if let Some(id) = *self
            .0
            .try_borrow()
            .map_err(|_| AccessError::PayloadBorrowed)?
        {
            edges.strong(id);
        }
        Ok(())
    }
}

#[test]
fn a_shared_read_checks_record_identity_and_releases_heap_access() {
    let runtime = RuntimeHeap::default();
    let record = runtime.allocate(17_u32).unwrap();
    let handle = record.record_ref().handle();
    let foreign = RuntimeHeap::default().allocate(23_u32).unwrap();

    runtime
        .read(|heap| {
            assert_eq!(heap.get(handle), Ok(&17));
            assert_eq!(heap.get(foreign.handle()), Err(AccessError::ForeignHeap));
            assert_eq!(runtime.collect(), Err(AccessError::HeapBorrowed));
        })
        .unwrap();

    drop(record);
    assert_eq!(runtime.collect(), Ok(1));
    assert_eq!(
        runtime.read(|heap| heap.get(handle).copied()),
        Ok(Err(AccessError::StaleHandle))
    );
}

#[test]
fn rooted_clones_keep_a_record_live_until_the_last_root_is_dropped() {
    let runtime = RuntimeHeap::default();
    let value = Rc::new(17_u32);
    let record = runtime.allocate(Rc::clone(&value)).unwrap();
    let edge = record.record_ref();
    let clone = record.clone();

    drop(record);
    assert_eq!(runtime.collect(), Ok(0));
    assert!(edge.is_live().unwrap());

    drop(clone);
    assert_eq!(runtime.collect(), Ok(1));
    assert!(!edge.is_live().unwrap());
    assert_eq!(*value, 17);
}

#[test]
fn independently_borrowed_records_share_one_heap() {
    let runtime = RuntimeHeap::default();
    let peer = runtime.clone();
    let left = runtime.allocate(Record(RefCell::new(None))).expect("left");
    let right = peer.allocate(Record(RefCell::new(None))).expect("right");

    let mut write = left.0.borrow_mut();
    assert!(right.0.borrow().is_none());
    assert!(matches!(
        runtime.collect(),
        Err(AccessError::PayloadBorrowed)
    ));
    *write = Some(right.handle().erase());
    drop(write);

    let right_id = right.handle();
    assert_eq!(peer.downcast::<Record>(right_id.erase()), Ok(right_id));
    assert_eq!(
        peer.downcast::<ReentrantDisposal>(right_id.erase()),
        Err(AccessError::WrongType)
    );
    drop(right);
    assert_eq!(runtime.collect().expect("trace left to right"), 0);
    assert!(peer.upgrade(right_id).expect("same heap").is_some());
    assert!(matches!(
        RuntimeHeap::default().upgrade(right_id),
        Err(AccessError::ForeignHeap)
    ));

    drop(left);
    assert_eq!(runtime.collect().expect("reclaim both records"), 2);
    assert!(peer.upgrade(right_id).expect("same heap").is_none());
}

#[test]
fn runtime_borrow_refusal_returns_input_and_roots_retain_the_runtime() {
    let runtime = RuntimeHeap::default();
    let anchor = runtime
        .allocate(Record(RefCell::new(None)))
        .expect("anchor");
    let id = anchor.handle().erase();
    let active_heap = runtime.0.borrow_mut();
    let rejected = runtime
        .allocate(Record(RefCell::new(Some(id))))
        .expect_err("heap access is active");
    assert_eq!(rejected.reason, AccessError::HeapBorrowed);
    assert_eq!(*rejected.value.0.borrow(), Some(id));
    drop(active_heap);

    let child = runtime.allocate(rejected.value).expect("retry allocation");
    drop(runtime);
    drop(anchor);
    assert_eq!(
        child.runtime().collect().expect("child keeps its runtime"),
        0
    );
    let handle = child
        .runtime()
        .downcast::<Record>(id)
        .expect("child keeps anchor");
    assert!(
        child
            .runtime()
            .upgrade(handle)
            .expect("same runtime")
            .is_some()
    );
}

struct ReentrantDisposal {
    runtime: Weak<RefCell<Heap>>,
    reentered: Rc<Cell<bool>>,
}

impl Trace for ReentrantDisposal {
    fn trace(&self, _: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        Ok(())
    }
}

impl Drop for ReentrantDisposal {
    fn drop(&mut self) {
        if let Some(heap) = self.runtime.upgrade() {
            self.reentered
                .set(matches!(RuntimeHeap(heap).collect(), Ok(0)));
        }
    }
}

#[test]
fn allocation_disposes_after_borrow_and_keeps_its_new_root_during_reentry() {
    let runtime = RuntimeHeap::default();
    let reentered = Rc::new(Cell::new(false));
    let canary = runtime
        .allocate(ReentrantDisposal {
            runtime: Rc::downgrade(&runtime.0),
            reentered: Rc::clone(&reentered),
        })
        .expect("canary");
    let canary_id = canary.handle();
    drop(canary);

    runtime
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .expect("enable forced collection");
    let fresh = runtime
        .allocate(Record(RefCell::new(None)))
        .expect("new root");
    assert!(reentered.get());
    assert!(runtime.upgrade(canary_id).expect("check canary").is_none());
    assert!(
        runtime
            .upgrade(fresh.handle())
            .expect("check new root")
            .is_some()
    );
}

#[test]
fn opportunistic_allocation_defers_a_borrowed_live_graph_and_retries() {
    use std::num::NonZeroUsize;

    let runtime = RuntimeHeap::default();
    let owner = runtime.allocate(Record(RefCell::new(None))).unwrap();
    let child = runtime.allocate(Record(RefCell::new(None))).unwrap();
    let child_ref = child.record_ref();
    let unreachable = runtime
        .allocate(Record(RefCell::new(None)))
        .unwrap()
        .record_ref();
    let mut write = owner.0.borrow_mut();
    *write = Some(child_ref.identity());
    drop(child);
    runtime
        .set_allocation_collection(AllocationCollection::Opportunistic(NonZeroUsize::MIN))
        .unwrap();

    let pending = runtime.allocate(Record(RefCell::new(None))).unwrap();
    assert!(child_ref.is_live().unwrap());
    assert!(unreachable.is_live().unwrap());
    drop(write);
    let retry = runtime.allocate(Record(RefCell::new(None))).unwrap();
    assert!(child_ref.is_live().unwrap());
    assert!(!unreachable.is_live().unwrap());
    assert!(pending.record_ref().is_live().unwrap());
    assert!(retry.record_ref().is_live().unwrap());
}

#[test]
fn adaptive_collection_waits_for_growth_after_a_sweep() {
    use std::num::NonZeroUsize;

    let runtime = RuntimeHeap::default();
    let allocate = || runtime.allocate(Record(RefCell::new(None))).unwrap();
    let owners = [allocate(), allocate(), allocate()];
    assert_eq!(runtime.collect().unwrap(), 0);
    runtime
        .set_allocation_collection(AllocationCollection::Adaptive(NonZeroUsize::MIN))
        .unwrap();

    let discarded = allocate().record_ref();
    let growth = [allocate(), allocate()];
    assert!(discarded.is_live().unwrap());

    let next = allocate();
    assert!(!discarded.is_live().unwrap());
    assert!(
        owners
            .iter()
            .chain(&growth)
            .chain([&next])
            .all(|root| root.record_ref().is_live().unwrap())
    );
}

#[test]
fn record_origin_heap_is_available_during_active_heap_access() {
    // The origin lookup is available independently of the payload's tracing API.
    fn origin<T>(record: &RecordRef<T>) -> Option<RuntimeHeap> {
        record.owning_heap()
    }

    let runtime = RuntimeHeap::default();
    let foreign = RuntimeHeap::default();
    let record = runtime.allocate(17_u32).unwrap();
    let reference = record.record_ref();
    runtime
        .read(|heap| {
            assert_eq!(heap.get(reference.handle()), Ok(&17));
            assert!(matches!(
                reference.upgrade(),
                Err(AccessError::HeapBorrowed)
            ));
            assert_eq!(reference.is_live(), Err(AccessError::HeapBorrowed));
            let owner = origin(&reference).expect("the origin heap is alive");
            assert!(owner.same_heap(&runtime));
            assert!(!owner.same_heap(&foreign));
        })
        .unwrap();
    assert!(reference.is_live().unwrap());
}

#[test]
fn reclaimed_record_keeps_its_origin_identity_after_real_slot_reuse() {
    let runtime = RuntimeHeap::default();
    let record = runtime.allocate(17_u32).unwrap();
    let reference = record.record_ref();
    let identity = reference.identity();
    drop(record);
    assert_eq!(runtime.collect(), Ok(1));
    let owner = reference
        .owning_heap()
        .expect("the heap outlives its record");
    assert!(owner.same_heap(&runtime));
    assert!(!reference.is_live().unwrap());
    assert!(reference.upgrade().unwrap().is_none());

    let replacement = owner.allocate(23_u32).unwrap();
    let replacement_identity = replacement.handle().erase();
    assert_eq!(
        replacement_identity.slot, identity.slot,
        "the actual reclaimed slot was reused"
    );
    assert_ne!(replacement_identity.generation, identity.generation);
    assert!(reference.owning_heap().unwrap().same_heap(&runtime));
    assert!(!reference.is_live().unwrap());
    assert!(reference.upgrade().unwrap().is_none());
    assert_eq!(
        owner.read(|heap| heap.get(reference.handle()).copied()),
        Ok(Err(AccessError::StaleHandle))
    );
    assert_eq!(*replacement, 23);
}

#[test]
fn recovered_heap_owner_extends_only_heap_lifetime() {
    let runtime = RuntimeHeap::default();
    let record = runtime.allocate(17_u32).unwrap();
    let reference = record.record_ref();
    let owner = reference.owning_heap().unwrap();
    drop(runtime);
    drop(record);

    assert_eq!(
        owner.collect(),
        Ok(1),
        "the recovered heap owner roots no record"
    );
    assert!(!reference.is_live().unwrap());
    assert!(reference.upgrade().unwrap().is_none());
    assert!(reference.owning_heap().unwrap().same_heap(&owner));

    drop(owner);
    assert!(reference.owning_heap().is_none());
    assert!(!reference.is_live().unwrap());
    assert!(reference.upgrade().unwrap().is_none());
}
